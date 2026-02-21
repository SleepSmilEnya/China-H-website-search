use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

static DOMAINS: Lazy<Vec<String>> = Lazy::new(|| {
    let mut domains = Vec::new();
    for a in b'a'..=b'z' {
        for b in b'a'..=b'z' {
            for c in b'a'..=b'z' {
                for d in b'a'..=b'z' {
                    let domain = format!("{}{}{}{}.cc", a as char, b as char, c as char, d as char);
                    domains.push(domain);
                }
            }
        }
    }
    domains
});

static TITLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<title[^>]*>([^<]+)</title>").unwrap()
});

#[derive(Clone, Serialize, Deserialize)]
struct FoundDomain {
    domain: String,
    title: String,
}

struct ScannerState {
    running: AtomicBool,
    paused: AtomicBool,
    current_index: AtomicUsize,
    found_domains: Mutex<Vec<FoundDomain>>,
    total: usize,
    concurrency: AtomicUsize,
}

impl Default for ScannerState {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            current_index: AtomicUsize::new(0),
            found_domains: Mutex::new(Vec::new()),
            total: DOMAINS.len(),
            concurrency: AtomicUsize::new(20),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct ScanProgress {
    current: usize,
    total: usize,
    found: Vec<FoundDomain>,
    running: bool,
    concurrency: usize,
}

fn extract_title(html: &str) -> String {
    if let Some(caps) = TITLE_REGEX.captures(html) {
        if let Some(title) = caps.get(1) {
            return title.as_str().trim().to_string();
        }
    }
    String::from("No Title")
}

fn is_domain_seller(title: &str) -> bool {
    let title_lower = title.to_lowercase();
    let keywords = [
        "域名出售",
        "domain for sale",
        "domain name for sale",
        "buy domain",
        "sell domain",
        "domain sale",
        "域名交易",
        "域名转让",
        "出售域名",
        "域名挂售",
        "domain parking",
        "domain is for sale",
        "sedo",
        "godaddy",
        "domain marketplace",
        "premium domain",
        "expired domain",
        "domain auction",
    ];
    keywords.iter().any(|k| title_lower.contains(k))
}

fn is_error_page(html: &str, title: &str) -> bool {
    let html_lower = html.to_lowercase();
    let title_lower = title.to_lowercase();
    
    let error_patterns = [
        "当前无法使用此页面",
        "无法访问此网站",
        "此网站暂时无法访问",
        "connection refused",
        "connection reset",
        "connection timed out",
        "timeout",
        "server error",
        "500 error",
        "502 bad gateway",
        "503 service unavailable",
        "504 gateway timeout",
        "403 forbidden",
        "404 not found",
        "this site can't be reached",
        "unable to connect",
        "no internet",
        "dns_probe_finished_nxdomain",
        "your connection is not private",
        "ssl error",
        "certificate error",
        "domain is invalid",
        "domain not found",
        " parked ",
        "域名停用",
        "网站暂时关闭",
        "页面不存在",
        "error 404",
        "error 403",
        "error 500",
        "maintenance",
        "coming soon",
        "under construction",
        "site not ready",
    ];
    
    error_patterns.iter().any(|p| title_lower.contains(p) || html_lower.contains(p))
}

#[tauri::command]
async fn set_concurrency(state: State<'_, Arc<Mutex<ScannerState>>>, concurrency: usize) -> Result<(), String> {
    if concurrency < 1 {
        return Err("Concurrency must be at least 1".to_string());
    }
    let state = state.lock().await;
    state.concurrency.store(concurrency, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn start_scan(app: AppHandle, state: State<'_, Arc<Mutex<ScannerState>>>) -> Result<(), String> {
    let state_guard = state.lock().await;
    
    if state_guard.running.load(Ordering::SeqCst) {
        return Err("Already running".to_string());
    }

    state_guard.running.store(true, Ordering::SeqCst);
    state_guard.paused.store(false, Ordering::SeqCst);

    let state_clone = Arc::clone(&state);
    
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        loop {
            let concurrency = state_clone.lock().await.concurrency.load(Ordering::SeqCst);
            
            if !state_clone.lock().await.running.load(Ordering::SeqCst) {
                break;
            }

            while state_clone.lock().await.paused.load(Ordering::SeqCst) {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if !state_clone.lock().await.running.load(Ordering::SeqCst) {
                    return;
                }
            }

            let start_idx = state_clone.lock().await.current_index.load(Ordering::SeqCst);
            let end_idx = (start_idx + concurrency).min(DOMAINS.len());

            if start_idx >= DOMAINS.len() {
                state_clone.lock().await.running.store(false, Ordering::SeqCst);
                let _ = app.emit("scan-complete", ());
                break;
            }

            let mut handles = Vec::new();
            
            for i in start_idx..end_idx {
                let domain = DOMAINS[i].clone();
                let client = client.clone();
                let app = app.clone();
                let state_clone = Arc::clone(&state_clone);

                handles.push(tokio::spawn(async move {
                    let url = format!("https://{}", domain);
                    match client.get(&url).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(html) = resp.text().await {
                                let title = extract_title(&html);
                                let title_lower = title.to_lowercase();
                                if title.is_empty() || title_lower == "no title" || title_lower.trim().is_empty() {
                                    return;
                                }
                                if is_domain_seller(&title) {
                                    return;
                                }
                                if is_error_page(&html, &title) {
                                    return;
                                }
                                let found = FoundDomain {
                                    domain: domain.clone(),
                                    title,
                                };
                                let _ = app.emit("found-domain", found.clone());
                                let state = state_clone.lock().await;
                                state.found_domains.lock().await.push(found);
                            }
                        }
                        _ => {}
                    }
                }));
            }

            for handle in handles {
                let _ = handle.await;
            }

            state_clone.lock().await.current_index.store(end_idx, Ordering::SeqCst);

            let state = state_clone.lock().await;
            let progress = ScanProgress {
                current: end_idx,
                total: DOMAINS.len(),
                found: state.found_domains.lock().await.clone(),
                running: state.running.load(Ordering::SeqCst),
                concurrency: state.concurrency.load(Ordering::SeqCst),
            };
            drop(state);
            let _ = app.emit("scan-progress", progress);
        }
    });

    Ok(())
}

#[tauri::command]
async fn pause_scan(state: State<'_, Arc<Mutex<ScannerState>>>) -> Result<(), String> {
    let state = state.lock().await;
    state.paused.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn resume_scan(state: State<'_, Arc<Mutex<ScannerState>>>) -> Result<(), String> {
    let state = state.lock().await;
    state.paused.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn stop_scan(state: State<'_, Arc<Mutex<ScannerState>>>) -> Result<(), String> {
    let state = state.lock().await;
    state.running.store(false, Ordering::SeqCst);
    state.paused.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
async fn reset_scan(state: State<'_, Arc<Mutex<ScannerState>>>) -> Result<(), String> {
    let state = state.lock().await;
    state.running.store(false, Ordering::SeqCst);
    state.paused.store(false, Ordering::SeqCst);
    state.current_index.store(0, Ordering::SeqCst);
    state.found_domains.lock().await.clear();
    Ok(())
}

#[tauri::command]
async fn get_status(state: State<'_, Arc<Mutex<ScannerState>>>) -> Result<ScanProgress, String> {
    let state = state.lock().await;
    let found = state.found_domains.lock().await.clone();
    let result = ScanProgress {
        current: state.current_index.load(Ordering::SeqCst),
        total: state.total,
        found,
        running: state.running.load(Ordering::SeqCst),
        concurrency: state.concurrency.load(Ordering::SeqCst),
    };
    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scanner_state = Arc::new(Mutex::new(ScannerState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(scanner_state)
        .invoke_handler(tauri::generate_handler![
            start_scan,
            pause_scan,
            resume_scan,
            stop_scan,
            reset_scan,
            get_status,
            set_concurrency
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
