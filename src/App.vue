<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

interface ScanProgress {
  current: number;
  total: number;
  found: FoundDomain[];
  running: boolean;
  concurrency: number;
}

interface FoundDomain {
  domain: string;
  title: string;
}

const current = ref(0);
const total = ref(456976);
const foundDomains = ref<FoundDomain[]>([]);
const isRunning = ref(false);
const isPaused = ref(false);
const concurrency = ref(20);
const searchKeyword = ref("");
const serverUrl = ref("");

async function loadServerInfo() {
  try {
    const ip = await invoke<string>("get_local_ip");
    serverUrl.value = `http://${ip}:8765`;
  } catch (e) {
    console.error(e);
  }
}

const filteredDomains = computed(() => {
  const validDomains = foundDomains.value.filter(item => 
    item.title && item.title.trim() !== "" && item.title.toLowerCase() !== "no title"
  );
  if (!searchKeyword.value.trim()) {
    return validDomains;
  }
  const keyword = searchKeyword.value.toLowerCase();
  return validDomains.filter(item => 
    item.domain.toLowerCase().includes(keyword) || 
    item.title.toLowerCase().includes(keyword)
  );
});

let unlistenProgress: UnlistenFn;
let unlistenFound: UnlistenFn;
let unlistenComplete: UnlistenFn;

async function loadStatus() {
  try {
    const status = await invoke<ScanProgress>("get_status");
    current.value = status.current;
    total.value = status.total;
    foundDomains.value = status.found;
    isRunning.value = status.running;
    concurrency.value = status.concurrency || 20;
  } catch (e) {
    console.error(e);
  }
}

async function setConcurrency() {
  if (concurrency.value < 1) concurrency.value = 1;
  if (concurrency.value > 200) concurrency.value = 200;
  try {
    await invoke("set_concurrency", { concurrency: concurrency.value });
  } catch (e) {
    console.error(e);
  }
}

async function startScan() {
  try {
    await invoke("start_scan");
    isRunning.value = true;
    isPaused.value = false;
  } catch (e) {
    console.error(e);
  }
}

async function pauseScan() {
  try {
    await invoke("pause_scan");
    isPaused.value = true;
  } catch (e) {
    console.error(e);
  }
}

async function resumeScan() {
  try {
    await invoke("resume_scan");
    isPaused.value = false;
  } catch (e) {
    console.error(e);
  }
}

async function resetScan() {
  try {
    await invoke("reset_scan");
    current.value = 0;
    foundDomains.value = [];
    isRunning.value = false;
    isPaused.value = false;
  } catch (e) {
    console.error(e);
  }
}

function formatNumber(num: number): string {
  return num.toLocaleString();
}

onMounted(async () => {
  loadServerInfo();
  await loadStatus();

  unlistenProgress = await listen<ScanProgress>("scan-progress", (event) => {
    current.value = event.payload.current;
    total.value = event.payload.total;
    foundDomains.value = event.payload.found;
    isRunning.value = event.payload.running;
    if (event.payload.concurrency) {
      concurrency.value = event.payload.concurrency;
    }
  });

  unlistenFound = await listen<FoundDomain>("found-domain", (event) => {
    foundDomains.value.unshift(event.payload);
  });

  unlistenComplete = await listen("scan-complete", () => {
    isRunning.value = false;
    isPaused.value = false;
  });
});

onUnmounted(() => {
  unlistenProgress?.();
  unlistenFound?.();
  unlistenComplete?.();
});
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>China H Website Search</h1>
      <p class="subtitle">Scan .cc domains from aaaa.cc to zzzz.cc</p>
      <p v-if="serverUrl" class="server-url">
        手机访问: <a :href="serverUrl" target="_blank">{{ serverUrl }}</a>
      </p>
    </header>

    <div class="control-panel">
      <div class="progress-info">
        <span class="progress-text">Progress: {{ formatNumber(current) }} / {{ formatNumber(total) }}</span>
        <span class="progress-percent">{{ ((current / total) * 100).toFixed(2) }}%</span>
      </div>
      <div class="progress-bar-container">
        <div class="progress-bar" :style="{ width: (current / total * 100) + '%' }"></div>
      </div>
      <div class="concurrency-control">
        <label for="concurrency">并发数:</label>
        <input 
          type="number" 
          id="concurrency" 
          v-model.number="concurrency" 
          @change="setConcurrency"
          :disabled="isRunning"
          min="1" 
          max="200"
        />
      </div>
    </div>

    <div class="button-group">
      <button v-if="!isRunning" @click="startScan" class="btn btn-primary">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z"/>
        </svg>
        Start
      </button>
      <button v-if="isRunning && !isPaused" @click="pauseScan" class="btn btn-warning">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/>
        </svg>
        Pause
      </button>
      <button v-if="isRunning && isPaused" @click="resumeScan" class="btn btn-success">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z"/>
        </svg>
        Resume
      </button>
      <button @click="resetScan" class="btn btn-secondary">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M17.65 6.35A7.958 7.958 0 0012 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08A5.99 5.99 0 0112 18c-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
        Reset
      </button>
    </div>

    <div class="stats">
      <div class="stat-card">
        <span class="stat-value">{{ formatNumber(foundDomains.length) }}</span>
        <span class="stat-label">Found Domains</span>
      </div>
    </div>

    <div class="domain-list-container">
      <div class="domain-list-header">
        <h2>Accessible Domains</h2>
        <div class="search-box">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <path d="M21 21l-4.35-4.35"/>
          </svg>
          <input 
            type="text" 
            v-model="searchKeyword" 
            placeholder="Search domains or titles..." 
          />
        </div>
      </div>
      <div class="domain-list">
        <div v-for="item in filteredDomains" :key="item.domain" class="domain-item">
          <div class="domain-info">
            <span class="domain-name">{{ item.domain }}</span>
            <span class="domain-title">{{ item.title }}</span>
          </div>
          <a :href="'https://' + item.domain" target="_blank" class="domain-link">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/>
              <polyline points="15,3 21,3 21,9"/>
              <line x1="10" y1="14" x2="21" y2="3"/>
            </svg>
          </a>
        </div>
        <div v-if="foundDomains.length === 0" class="empty-state">
          <p>No accessible domains found yet</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background-color: #ffffff;
  color: #262626;
  line-height: 1.5;
}

.app {
  max-width: 800px;
  margin: 0 auto;
  padding: 40px 24px;
}

.header {
  text-align: center;
  margin-bottom: 32px;
}

.header h1 {
  font-size: 28px;
  font-weight: 600;
  color: #262626;
  margin-bottom: 8px;
}

.subtitle {
  font-size: 14px;
  color: #8e8e8e;
}

.server-url {
  font-size: 12px;
  color: #0095f6;
  margin-top: 8px;
}

.server-url a {
  color: #0095f6;
  text-decoration: none;
}

.server-url a:hover {
  text-decoration: underline;
}

.control-panel {
  background: #ffffff;
  border: 1px solid #f5f5f5;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 20px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.progress-text {
  font-size: 14px;
  color: #262626;
}

.progress-percent {
  font-size: 14px;
  color: #8e8e8e;
}

.concurrency-control {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #f5f5f5;
}

.concurrency-control label {
  font-size: 14px;
  color: #262626;
}

.concurrency-control input {
  width: 80px;
  padding: 6px 10px;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  font-size: 14px;
  text-align: center;
}

.concurrency-control input:disabled {
  background: #f5f5f5;
  color: #999;
}

.concurrency-control input:focus {
  outline: none;
  border-color: #0095f6;
}

.progress-bar-container {
  width: 100%;
  height: 8px;
  background: #f5f5f5;
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #0095f6, #00c8ff);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.button-group {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn svg {
  width: 16px;
  height: 16px;
}

.btn-primary {
  background: #0095f6;
  color: #ffffff;
}

.btn-primary:hover {
  background: #0084dd;
}

.btn-success {
  background: #4caf50;
  color: #ffffff;
}

.btn-success:hover {
  background: #43a047;
}

.btn-warning {
  background: #ff9800;
  color: #ffffff;
}

.btn-warning:hover {
  background: #f57c00;
}

.btn-danger {
  background: #f44336;
  color: #ffffff;
}

.btn-danger:hover {
  background: #e53935;
}

.btn-secondary {
  background: #f5f5f5;
  color: #262626;
}

.btn-secondary:hover {
  background: #e8e8e8;
}

.stats {
  display: flex;
  justify-content: center;
  margin-bottom: 24px;
}

.stat-card {
  background: #ffffff;
  border: 1px solid #f5f5f5;
  border-radius: 12px;
  padding: 20px 40px;
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 32px;
  font-weight: 600;
  color: #0095f6;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 12px;
  color: #8e8e8e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.domain-list-container {
  background: #ffffff;
  border: 1px solid #f5f5f5;
  border-radius: 12px;
  overflow: hidden;
}

.domain-list-header {
  padding: 16px 20px;
  border-bottom: 1px solid #f5f5f5;
}

.domain-list-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: #262626;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #f5f5f5;
  border-radius: 8px;
  margin-top: 12px;
}

.search-box svg {
  color: #8e8e8e;
  flex-shrink: 0;
}

.search-box input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 14px;
  color: #262626;
  outline: none;
}

.search-box input::placeholder {
  color: #8e8e8e;
}

.domain-list {
  max-height: 400px;
  overflow-y: auto;
}

.domain-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-bottom: 1px solid #f5f5f5;
  transition: background 0.2s ease;
}

.domain-item:hover {
  background: #fafafa;
}

.domain-item:last-child {
  border-bottom: none;
}

.domain-info {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  min-width: 0;
}

.domain-name {
  font-size: 14px;
  color: #262626;
}

.domain-title {
  font-size: 12px;
  color: #8e8e8e;
  margin-left: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.domain-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  color: #8e8e8e;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.domain-link:hover {
  background: #f5f5f5;
  color: #0095f6;
}

.empty-state {
  padding: 40px 20px;
  text-align: center;
}

.empty-state p {
  font-size: 14px;
  color: #8e8e8e;
}

.domain-list::-webkit-scrollbar {
  width: 6px;
}

.domain-list::-webkit-scrollbar-track {
  background: #f5f5f5;
}

.domain-list::-webkit-scrollbar-thumb {
  background: #d1d1d1;
  border-radius: 3px;
}

.domain-list::-webkit-scrollbar-thumb:hover {
  background: #b0b0b0;
}
</style>
