# Tauri Best Practices Guide

## 🎯 Mission: Build World-Class MacBook Apps with Tauri

**Target**: Production-quality MacBook desktop applications
**Philosophy**: Client-side privacy, Google-authenticated, no server touch
**Platform**: macOS primary (universal for later)

---

## 1. Architecture Patterns

### Frontend Framework Selection
```typescript
// Recommended for MacBook apps
• Svelte + TypeScript (lightweight, fast)
• React + TypeScript (ecosystem, familiar)
• Vue 3 + TypeScript (progressive)
```

**Best Practice**: Svelte for performance on macOS

### State Management
```typescript
// Tauri + Store Pattern
import { invoke } from '@tauri-apps/api/tauri';
import { writable } from 'svelte/store';

// Client-side state (no server)
export const userStore = writable(null);
export const appState = writable({
  isAuthenticated: false,
  theme: 'system',
  lastSync: null
});

// Secure storage for tokens
import { Store } from 'tauri-plugin-store-api';
const secureStore = new Store('.secure.dat');
```

### IPC Communication Design
```rust
// src-tauri/src/commands.rs
#[tauri::command]
async fn authenticate_google(code: String) -> Result<User, String> {
    // Validate OAuth code
    // Store token securely
    // Return user data
}

#[tauri::command]
async fn get_data() -> Result<Vec<Item>, String> {
    // Client-side data access
    // No server calls
}

#[tauri::command]
async fn save_data(data: Vec<Item>) -> Result<(), String> {
    // Local storage
}
```

---

## 2. MacBook/MacOS Optimization

### Native Menu Bar
```typescript
import { Menu, MenuItem, Submenu } from '@tauri-apps/api/menu';

const createMenuBar = () => {
  const menu = Menu.new();
  const file = Submenu.new("File", Menu.default());
  const edit = Submenu.new("Edit", Menu.default());
  
  // Add app-specific menu items
  const actions = Submenu.new("Actions", 
    Menu.withItems([
      MenuItem::with_id("sync", "Sync", false, None::<&str>),
      MenuItem::with_id("settings", "Settings", false, None::<&str>)
    ])
  );
  
  menu.append(&file)?;
  menu.append(&edit)?;
  menu.append(&actions)?;
  menu.set_as_app_menu()?;
};
```

### Native Notifications
```typescript
import { sendNotification } from '@tauri-apps/api/notification';

export const showNotification = (title: string, body: string) => {
  sendNotification({
    title,
    body,
    icon: 'path/to/icon.icns' // Native macOS icon
  });
};
```

### File System Access
```typescript
import { open, save } from '@tauri-apps/api/dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';

// Native file picker
const importData = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'JSON', extensions: ['json'] }]
  });
  
  if (selected) {
    const content = await readTextFile(selected);
    return JSON.parse(content);
  }
};

// Native save dialog
const exportData = async (data: any) => {
  const filePath = await save({
    filters: [{ name: 'JSON', extensions: ['json'] }]
  });
  
  if (filePath) {
    await writeTextFile(filePath, JSON.stringify(data, null, 2));
  }
};
```

### Dock Integration
```rust
// src-tauri/src/main.rs
// macOS-specific dock setup
#[cfg(target_os = "macos")]
use cocoa::appkit::NSApplication;

#[cfg(target_os = "macos")]
fn setup_dock_integration() {
    let app = unsafe { NSApplication::sharedApplication() };
    // Set dock icon behavior
    // Configure dock menu
}
```

---

## 3. Authentication Patterns (Google + Privacy)

### Google OAuth Flow
```typescript
// src/auth/google.ts
import { invoke } from '@tauri-apps/api/tauri';
import { Store } from 'tauri-plugin-store-api';

const secureStore = new Store('.secure.dat');

export const authenticateWithGoogle = async () => {
  // 1. Open Google OAuth popup (client-side)
  const authUrl = 'https://accounts.google.com/o/oauth2/v2/auth' +
    '?client_id=' + GOOGLE_CLIENT_ID +
    '&redirect_uri=' + REDIRECT_URI +
    '&response_type=code' +
    '&scope=email profile';
  
  const code = await openAuthPopup(authUrl);
  
  // 2. Exchange code for tokens (no server needed)
  const tokens = await invoke('exchange_code', { code });
  
  // 3. Store securely (client-side)
  await secureStore.set('access_token', tokens.access_token);
  await secureStore.set('refresh_token', tokens.refresh_token);
  await secureStore.set('user', tokens.user);
  
  return tokens.user;
};
```

### Client-Side Token Management
```typescript
// Token refresh without server
export const refreshTokens = async () => {
  const refreshToken = await secureStore.get('refresh_token');
  
  if (!refreshToken) {
    throw new Error('No refresh token');
  }
  
  const newTokens = await invoke('refresh_tokens', {
    refresh_token: refreshToken
  });
  
  // Update secure storage
  await secureStore.set('access_token', newTokens.access_token);
  await secureStore.set('user', newTokens.user);
  
  return newTokens;
};

// Logout - clear local storage only
export const logout = async () => {
  await secureStore.delete('access_token');
  await secureStore.delete('refresh_token');
  await secureStore.delete('user');
};
```

### Privacy-First Architecture
```
┌─────────────────────────────────────┐
│          MacBook App                │
│  (All data stays local)            │
├─────────────────────────────────────┤
│                                     │
│  ┌────────────┐                   │
│  │ Secure     │                   │
│  │ Storage    │                   │
│  │ (Tauri     │                   │
│  │ Store)     │                   │
│  └────────────┘                   │
│       ↑                            │
│       │                            │
│  ┌────────────┐                   │
│  │ Google     │                   │
│  │ Auth       │                   │
│  │ (OAuth)     │                   │
│  └────────────┘                   │
│       ↓                            │
│  No server calls ever               │
│  All operations local               │
└─────────────────────────────────────┘
```

---

## 4. Performance Best Practices

### Bundle Optimization
```json
// src-tauri/tauri.conf.json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "identifier": "com.yourapp.desktop",
    "icon": ["icons/32x32.png", "icons/icon.icns"]
  },
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  }
}
```

### Lazy Loading
```typescript
// Svelte component lazy loading
const DataEditor = lazy(() => import('./components/DataEditor.svelte'));
const Settings = lazy(() => import('./components/Settings.svelte'));

<svelte:component this={DataEditor} />
```

### Native vs Web Separation
```rust
// Keep Rust code minimal
// Do heavy work in TypeScript

// ✅ Good: IPC interface
#[tauri::command]
fn compute_heavy(input: Vec<Data>) -> Vec<Result> {
    // Just orchestrate
    input.par_iter().map(|d| d.process()).collect()
}

// ❌ Bad: All logic in Rust
// Keep UI and business logic in JS/TS
```

---

## 5. Security Patterns

### Secure Token Storage
```rust
// src-tauri/src/secure_store.rs
use std::path::PathBuf;
use tauri_plugin_store::StoreBuilder;

pub async fn get_secure_store() -> Result<Store, Error> {
    let path = get_secure_store_path()?;
    StoreBuilder::new(path).build()
}

fn get_secure_store_path() -> Result<PathBuf, Error> {
    // macOS Keychain integration
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join("Library/Application Support/YourApp/.secure.dat"))
}
```

### Client-Side Encryption
```typescript
import CryptoJS from 'crypto-js';

const encryptData = (data: any, secret: string) => {
  return CryptoJS.AES.encrypt(JSON.stringify(data), secret).toString();
};

const decryptData = (encrypted: string, secret: string) => {
  const bytes = CryptoJS.AES.decrypt(encrypted, secret);
  return JSON.parse(bytes.toString(CryptoJS.enc.Utf8));
};

// Use user's Google token as encryption key
const encryptUserData = async (data: any, userToken: string) => {
  const encrypted = encryptData(data, userToken);
  await writeTextFile('encrypted.dat', encrypted);
};
```

### CSP Configuration
```html
<!-- index.html -->
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self'; 
               script-src 'self'; 
               style-src 'self' 'unsafe-inline'; 
               connect-src 'self' https://accounts.google.com">
```

---

## 6. Development Workflow

### Hot Reload Setup
```bash
# Install dev dependencies
npm install -D @tauri-apps/cli svelte-check

# Development with hot reload
npm run tauri dev

# This will:
# • Start frontend dev server (localhost:1420)
# • Start Tauri dev watcher
# • Auto-reload on file changes
```

### Testing Strategy
```typescript
// Unit tests
import { test, expect } from 'vitest';

test('authenticate with Google', async () => {
  const user = await authenticateWithGoogle();
  expect(user).toHaveProperty('email');
  expect(user).toHaveProperty('name');
});

// Integration tests
import { invoke } from '@tauri-apps/api/tauri';

test('save data locally', async () => {
  const data = [{ id: 1, text: 'test' }];
  await invoke('save_data', { data });
  
  const saved = await invoke('get_data');
  expect(saved).toEqual(data);
});
```

### Debugging Tools
```typescript
// Tauri debug mode
import { app } from '@tauri-apps/api';

// Enable debug logging
if (await app.appWindow.label() === 'dev') {
  console.log('Debug mode enabled');
  // Log all IPC calls
  invoke('enable_debug_logging');
}
```

---

## 7. Production Considerations

### Code Signing for macOS
```bash
# 1. Get developer certificate from Apple Developer account
# 2. Configure in tauri.conf.json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAMID)",
      "entitlements": null,
      "provisioningProfile": null
    }
  }
}

# 3. Build with signing
npm run tauri build
```

### Distribution Options

**DMG (Recommended)**
```bash
# Drag-and-drop installer
# User experience: Install like any Mac app
npm run tauri build --target dmg
```

**PKG (Enterprise)**
```bash
# Silent installation
# For enterprise deployment
npm run tauri build --target pkg
```

**App Store (MAS)**
```json
{
  "bundle": {
    "macOS": {
      "entitlements": "entitlements.mas.plist",
      "provisioningProfile": "profile.provisionprofile"
    }
  }
}
```

### Update Mechanism
```typescript
import { checkUpdate, installUpdate } from '@tauri-apps/api/updater';

export const checkForUpdates = async () => {
  const { shouldUpdate, manifest } = await checkUpdate();
  
  if (shouldUpdate) {
    await installUpdate();
    // Restart app after update
    await relaunch();
  }
};

// Check on startup (with permission)
if (userPreferences.checkUpdates) {
  checkForUpdates();
}
```

---

## 8. Tweet Scrolls Application Architecture

### Proposed Structure
```
tweet-scrolls-macbook/
├── src/
│   ├── components/
│   │   ├── DataViewer.svelte          # Thread display
│   │   ├── SearchFilter.svelte       # Search interface
│   │   ├── AnalyticsPanel.svelte     # Stats visualization
│   │   └── Settings.svelte         # User preferences
│   ├── stores/
│   │   ├── auth.ts                 # Google auth store
│   │   ├── data.ts                 # Thread data store
│   │   └── ui.ts                  # UI preferences
│   ├── lib/
│   │   ├── auth.ts                 # Google OAuth
│   │   ├── storage.ts              # Secure storage wrapper
│   │   └── encryption.ts          # Data encryption
│   └── main.ts
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs             # IPC commands
│   │   ├── secure_store.rs         # macOS Keychain
│   │   └── encryption.rs          # Rust encryption
│   └── tauri.conf.json
└── public/
    └── icons/
        └── icon.icns              # Native macOS icon
```

### Authentication Flow
```typescript
// 1. User opens app
// 2. App detects no auth token
// 3. "Sign in with Google" button
// 4. Opens browser for OAuth
// 5. Redirects back to app (custom URL scheme)
// 6. Exchanges code for tokens
// 7. Stores tokens in macOS Keychain
// 8. Unlocks app features
```

### Privacy Guarantee
```
All user data:
  ✓ Stored locally on MacBook
  ✓ Encrypted with user's Google token
  ✓ Never sent to any server
  ✓ Synced only if user explicitly enables iCloud
  ✓ Deleted on app uninstall
```

---

## 9. Quick Start Checklist

- [ ] Create Tauri app with `npm create tauri-app`
- [ ] Choose Svelte + TypeScript
- [ ] Configure Google OAuth client
- [ ] Implement secure storage (Tauri Store)
- [ ] Set up native menu bar
- [ ] Create data viewer component
- [ ] Implement file import/export
- [ ] Add native notifications
- [ ] Configure code signing
- [ ] Test on macOS (build .app)
- [ ] Verify no server calls
- [ ] Test authentication flow
- [ ] Validate data encryption
- [ ] Create DMG distribution
- [ ] Document for users

---

## 📚 Resources

- [Tauri Documentation](https://tauri.app/)
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)
- [Tauri Plugins](https://github.com/tauri-apps/plugins-workspace)
- [macOS Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/macos)

---

**Version**: 1.0  
**Created**: 2026-03-12  
**Status**: Ready for implementation
