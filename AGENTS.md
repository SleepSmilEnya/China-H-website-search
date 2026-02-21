# AGENTS.md - Project Guidelines

This document provides guidelines for AI agents working on this codebase.

## Project Overview

- **Project Name**: China H Website Search
- **Type**: Desktop application (Tauri v2 + Vue 3)
- **Purpose**: Scan .cc domains from aaaa.cc to zzzz.cc to find accessible websites
- **Tech Stack**: Vue 3, TypeScript, Vite, Tailwind CSS, Tauri v2, Rust

## Build Commands

### Frontend (Node.js)

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server (port 1420) |
| `npm run build` | Build frontend (runs `vue-tsc --noEmit` then `vite build`) |
| `npm run preview` | Preview built frontend |

### Tauri (Full App)

| Command | Description |
|---------|-------------|
| `npm run tauri dev` | Start Tauri in development mode |
| `npm run tauri build` | Build production executable (.exe, .msi, .dmg, .app, etc.) |

### Running Single Commands

```bash
# Type check only (no emit)
npx vue-tsc --noEmit

# Build frontend only
npx vite build

# Build Tauri only (requires frontend built first)
cd src-tauri && cargo build --release
```

## Code Style Guidelines

### General Principles

- Use **TypeScript** for all new code (strict mode enabled)
- Enable **strict type checking** (`strict: true` in tsconfig)
- Use **ES2020+** features
- Prefer **functional components** with Composition API (`<script setup>`)

### TypeScript Conventions

```typescript
// Interface naming: PascalCase, prefix with descriptive name
interface ScanProgress {
  current: number;
  total: number;
}

// Type for component refs
const count = ref<number>(0);

// Optional propertiesinterface FoundDomain {
 use ?
  domain: string;
  title?: string;  // optional
}
```

### Vue 3 Composition API

```vue
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";

// Define interfaces near usage
interface User {
  id: number;
  name: string;
}

// Use ref for primitives, reactive for objects
const isRunning = ref(false);
const userData = reactive<User>({ id: 0, name: "" });

// Computed properties for derived state
const progressPercent = computed(() => {
  return total.value > 0 ? (current.value / total.value) * 100 : 0;
});

// Always clean up listeners
let unlistenFn: UnlistenFn;
onMounted(async () => {
  unlistenFn = await listen<Event>("event-name", (event) => {
    // handle event
  });
});

onUnmounted(() => {
  unlistenFn?.();
});
</script>
```

### Imports Ordering

1. External libraries (Vue, Tauri APIs)
2. Internal components
3. Types/interfaces
4. Styles (if any)

```typescript
// 1. External
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// 2. Internal (if exists)
// import MyComponent from "./components/MyComponent.vue";

// 3. Types
interface MyType {
  key: string;
}
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Files | kebab-case | `scan-progress.ts`, `my-component.vue` |
| Components | PascalCase | `ScanProgress.vue`, `MyComponent.vue` |
| Functions | camelCase | `startScan()`, `formatNumber()` |
| Variables | camelCase | `current`, `foundDomains` |
| Constants | UPPER_SNAKE_CASE | `MAX_CONCURRENCY` |
| Interfaces | PascalCase | `ScanProgress`, `FoundDomain` |
| CSS Classes | kebab-case | `.progress-bar`, `.domain-list` |

### Error Handling

```typescript
// Always wrap async calls in try-catch
async function loadData() {
  try {
    const data = await invoke<MyType>("get_data");
    return data;
  } catch (e) {
    console.error(e);  // Log error
    // Handle gracefully - set error state or show user message
    return null;
  }
}

// Use optional chaining and nullish coalescing
const title = foundDomain?.title ?? "No Title";
```

### CSS/Styling

- Use **Tailwind CSS v4** for utility classes when possible
- Keep custom CSS minimal and scoped to components
- Follow existing color scheme (primary: `#0095f6`)
- Use BEM-like naming for custom classes: `.block-element--modifier`

### Rust Backend (src-tauri/)

- Use **Rust 2021 edition**
- Enable strict linting: `#![deny(warnings)]`
- Follow standard Rust naming (snake_case for functions/variables)
- Use `serde` for serialization/deserialization with Tauri commands

```rust
#[tauri::command]
async fn get_local_ip() -> Result<String, String> {
    // Return Result to allow error propagation to frontend
    Ok(ip_address)
}
```

### File Structure

```
src/                      # Frontend source
├── main.ts              # Entry point
├── App.vue              # Root component
├── components/          # Reusable components
├── types/               # TypeScript type definitions
└── utils/               # Utility functions

src-tauri/               # Backend source
├── src/
│   ├── main.rs          # Binary entry
│   └── lib.rs           # Library code
├── Cargo.toml           # Rust dependencies
└── tauri.conf.json      # Tauri configuration
```

### Testing

Currently **no test framework is configured** for this project. If adding tests:

- Use **Vitest** for Vue/TypeScript unit tests
- Use **rstest** or built-in Rust test framework for Rust code

### Linting & Formatting

- **TypeScript**: `vue-tsc --noEmit` (runs during `npm run build`)
- **No additional linter configured** - maintain consistent code style manually
- Consider adding ESLint + Prettier for stricter enforcement

### Important Configuration Files

- `tsconfig.json` - TypeScript configuration (strict mode enabled)
- `vite.config.ts` - Vite bundler configuration
- `src-tauri/tauri.conf.json` - Tauri app configuration
- `src-tauri/Cargo.toml` - Rust dependencies

## Common Tasks

### Add a new Tauri command

1. Add function in `src-tauri/src/lib.rs` with `#[tauri::command]` attribute
2. Import and call from frontend using `invoke()`:

```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke<ReturnType>("command_name", { param: value });
```

### Add a new frontend component

1. Create `.vue` file in appropriate directory
2. Use `<script setup lang="ts">` syntax
3. Export props with TypeScript interfaces

### Build for specific platform

```bash
# Windows (default)
npm run tauri build

# Note: Cross-compilation requires additional setup
# - macOS: requires Xcode
# - Linux: requires cross-compilation toolchain
```
