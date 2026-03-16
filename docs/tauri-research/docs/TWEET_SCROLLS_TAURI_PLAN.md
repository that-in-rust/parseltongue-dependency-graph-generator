# Tweet Scrolls MacBook App - Tauri Implementation Plan

## 🎯 Vision

**Build**: World-class MacBook desktop application for tweet archive management  
**Philosophy**: Private, authenticated, no server touch  
**Target**: Professional knowledge workers who want offline access to Twitter archives  

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│         Tweet Scrolls MacBook App            │
│   (All data stays on user's MacBook)         │
├─────────────────────────────────────────────────┤
│                                            │
│  ┌──────────────────┐                      │
│  │   Svelte UI      │                      │
│  │  (Frontend)     │                      │
│  └────────┬─────────┘                      │
│           │                                │
│           ↓                                │
│  ┌──────────────────┐                      │
│  │  Tauri Commands │                      │
│  │  (IPC Layer)     │                      │
│  └────────┬─────────┘                      │
│           │                                │
│           ↓                                │
│  ┌──────────────────┐                      │
│  │  Rust Backend    │                      │
│  │  (File System)   │                      │
│  └────────┬─────────┘                      │
│           │                                │
│           ↓                                │
│  ┌──────────────────┐                      │
│  │  Local Files     │                      │
│  │  (JSON/CSV)      │                      │
│  └──────────────────┘                      │
│                                            │
│  ┌──────────────────┐                      │
│  │  Google Auth     │                      │
│  │  (OAuth 2.0)     │                      │
│  └──────────────────┘                      │
│                                            │
│  ┌──────────────────┐                      │
│  │  macOS Keychain  │                      │
│  │  (Secure Storage)│                      │
│  └──────────────────┘                      │
└─────────────────────────────────────────────────┘
```

---

## User Journey

### Phase 1: First Launch
```
1. User downloads DMG from website
2. User installs app (drag to Applications)
3. User launches "Tweet Scrolls"
4. Welcome screen appears:
   "Analyze your Twitter archive privately"
5. User clicks "Sign in with Google"
6. Browser opens → Google OAuth
7. User authorizes → Redirects to app
8. Token stored in macOS Keychain
9. App unlocks features
10. Onboarding tutorial starts
```

### Phase 2: Import Data
```
1. User clicks "Import Archive"
2. Native file picker opens (macOS dialog)
3. User selects Twitter archive folder
4. App parses threads.json automatically
5. Progress bar shows parsing
6. Native notification: "3,847 threads imported"
7. Threads ready for exploration
```

### Phase 3: Explore & Analyze
```
1. User sees thread list
2. Filter by score (PMF 80+)
3. Search by keyword
4. Click thread → Detailed view
5. See scoring breakdown
6. Export high-value threads to CSV
7. All operations happen locally
```

---

## Core Features

### 1. Authentication Module

**Purpose**: Unlock app using Google identity  
**Privacy**: No server - token stored locally  

```typescript
// src/lib/auth/google.ts
export const authenticate = async () => {
  // 1. Generate OAuth URL
  const authUrl = generateGoogleAuthURL();
  
  // 2. Open browser for user consent
  const authCode = await openAuthPopup(authUrl);
  
  // 3. Exchange code for tokens
  const tokens = await invoke('exchange_code', { 
    code: authCode 
  });
  
  // 4. Store in macOS Keychain
  await invoke('store_tokens', { 
    access: tokens.access_token,
    refresh: tokens.refresh_token 
  });
  
  // 5. Return user info
  return await invoke('get_user_profile');
};

export const checkAuthStatus = async () => {
  const hasToken = await invoke('has_stored_token');
  return hasToken;
};

export const logout = async () => {
  await invoke('delete_stored_tokens');
};
```

### 2. Data Import Module

**Purpose**: Import Twitter archive JSON files  
**Performance**: Fast parsing for large archives  

```typescript
// src/lib/data/importer.ts
export const importArchive = async (folderPath: string) => {
  // 1. Detect archive structure
  const structure = await invoke('detect_archive_structure', {
    path: folderPath
  });
  
  // 2. Parse threads.json (main data)
  const threads = await invoke('parse_threads_file', {
    path: folderPath + '/threads.js'
  });
  
  // 3. Validate thread scores
  const scored = await invoke('validate_thread_scores', {
    threads
  });
  
  // 4. Store in local database
  await invoke('store_threads', { 
    threads: scored,
    dbPath: getDatabasePath()
  });
  
  // 5. Show completion
  showNotification('Import Complete', `${scored.length} threads ready`);
  
  return scored.length;
};
```

### 3. Thread Viewer Module

**Purpose**: Display and explore threads  
**Features**: Filtering, searching, sorting  

```svelte
<!-- src/components/ThreadViewer.svelte -->
<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  
  let threads = [];
  let filtered = [];
  let searchQuery = '';
  let minScore = 80;
  let sortBy = 'score'; // score, date, length
  
  onMount(async () => {
    threads = await invoke('load_threads');
    filterThreads();
  });
  
  function filterThreads() {
    filtered = threads
      .filter(t => t.pmf_score >= minScore)
      .filter(t => {
        if (!searchQuery) return true;
        const q = searchQuery.toLowerCase();
        return t.theme.includes(q) || 
               t.thread_text.toLowerCase().includes(q);
      })
      .sort((a, b) => {
        if (sortBy === 'score') return b.pmf_score - a.pmf_score;
        if (sortBy === 'date') return new Date(b.date) - new Date(a.date);
        if (sortBy === 'length') return b.thread_text.length - a.thread_text.length;
      });
  }
  
  $: if (searchQuery || minScore) filterThreads();
</script>

<div class="thread-viewer">
  <div class="controls">
    <input 
      type="text" 
      placeholder="Search threads..." 
      bind:value={searchQuery}
    />
    
    <label>
      Min Score: {minScore}
      <input 
        type="range" 
        min="0" 
        max="100" 
        bind:value={minScore} 
      />
    </label>
    
    <select bind:value={sortBy}>
      <option value="score">Sort by Score</option>
      <option value="date">Sort by Date</option>
      <option value="length">Sort by Length</option>
    </select>
  </div>
  
  <div class="threads">
    {#each filtered as thread (i)}
      <div class="thread-card score-{thread.pmf_score}">
        <div class="score">{thread.pmf_score}</div>
        <div class="content">
          <div class="theme">{thread.theme}</div>
          <div class="text">{thread.thread_text}</div>
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .thread-card {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    margin-bottom: 0.5rem;
  }
  
  .score-90 { border-left: 4px solid #00c853; }
  .score-80 { border-left: 4px solid #64dd17; }
  .score-70 { border-left: 4px solid #00e676; }
  .score-60 { border-left: 4px solid #00b0ff; }
</style>
```

### 4. Export Module

**Purpose**: Export filtered threads to CSV/JSON  
**Features**: Selective export, formatting options  

```typescript
// src/lib/data/exporter.ts
export const exportThreads = async (
  threads: Thread[],
  format: 'csv' | 'json',
  includeReasoning: boolean
) => {
  let content;
  
  if (format === 'csv') {
    const headers = ['PMF Score', 'Theme', 'Reasoning', 'Thread Text'];
    const rows = threads.map(t => [
      t.pmf_score,
      t.theme,
      includeReasoning ? t.reasoning : '',
      `"${t.thread_text.replace(/"/g, '""')}"` // Escape quotes
    ]);
    
    content = [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
  } else {
    content = JSON.stringify(threads, null, 2);
  }
  
  // 1. Native file save dialog
  const filePath = await invoke('save_file_dialog', {
    defaultPath: `tweet-scrolls-export.${format}`,
    filters: [{ name: format.toUpperCase(), extensions: [format] }]
  });
  
  if (filePath) {
    // 2. Write to local filesystem
    await invoke('write_file', {
      path: filePath,
      content
    });
    
    showNotification('Export Complete', `Saved to ${filePath}`);
  }
};
```

### 5. Analytics Module

**Purpose**: Visualize thread statistics  
**Features**: Score distribution, themes, timeline  

```svelte
<!-- src/components/AnalyticsPanel.svelte -->
<script>
  import { onMount } from 'svelte';
  import Chart from 'chart.js/auto';
  
  let threads = [];
  let scoreDistribution = {};
  let topThemes = {};
  
  onMount(async () => {
    threads = await invoke('load_threads');
    calculateStatistics();
  });
  
  function calculateStatistics() {
    // Score distribution
    threads.forEach(t => {
      const range = Math.floor(t.pmf_score / 10) * 10;
      scoreDistribution[range] = (scoreDistribution[range] || 0) + 1;
    });
    
    // Top themes
    threads.forEach(t => {
      const theme = t.theme.split('-')[0];
      topThemes[theme] = (topThemes[theme] || 0) + 1;
    });
  }
</script>

<div class="analytics">
  <h2>Thread Statistics</h2>
  
  <div class="stats">
    <div class="stat-card">
      <h3>Total Threads</h3>
      <p>{threads.length}</p>
    </div>
    
    <div class="stat-card">
      <h3>Average Score</h3>
      <p>{(threads.reduce((a, b) => a + b.pmf_score, 0) / threads.length).toFixed(1)}</p>
    </div>
    
    <div class="stat-card">
      <h3>High Value (80+)</h3>
      <p>{threads.filter(t => t.pmf_score >= 80).length}</p>
    </div>
  </div>
  
  <div class="charts">
    <canvas id="scoreChart"></canvas>
    <canvas id="themeChart"></canvas>
  </div>
</div>
```

---

## Privacy Guarantee

### Data Flow
```
User's MacBook (All Data)
    ↓
Encrypted Local Storage
    ↓
Accessed Only By User
    ↓
No Network Calls
    ↓
No Server Storage
    ↓
Complete Privacy
```

### Security Measures

1. **Token Storage**
   - macOS Keychain integration
   - Encrypted at rest
   - App-specific access
   - Revocable at logout

2. **Data Storage**
   - Local SQLite database
   - Encrypted with user's token
   - Filesystem permissions restricted
   - No cloud sync (user choice)

3. **Network Usage**
   - Only for OAuth flow (Google)
   - No telemetry
   - No analytics
   - No crash reporting

---

## Implementation Checklist

### Phase 1: Foundation (Week 1)
- [ ] Initialize Tauri app with Svelte + TypeScript
- [ ] Configure Google OAuth client
- [ ] Set up macOS Keychain storage
- [ ] Create basic UI shell
- [ ] Implement authentication flow

### Phase 2: Core Features (Week 2-3)
- [ ] Build data importer for threads.json
- [ ] Create thread database (SQLite)
- [ ] Implement thread viewer component
- [ ] Add filtering and search
- [ ] Create thread detail view

### Phase 3: Advanced Features (Week 4)
- [ ] Build export module (CSV/JSON)
- [ ] Create analytics dashboard
- [ ] Add native notifications
- [ ] Implement settings panel
- [ ] Add keyboard shortcuts

### Phase 4: Polish (Week 5)
- [ ] Design icon and branding
- [ ] Create native menu bar
- [ ] Add dock integration
- [ ] Optimize performance
- [ ] Code signing for macOS

### Phase 5: Launch (Week 6)
- [ ] Build DMG installer
- [ ] Create website landing page
- [ ] Write user documentation
- [ ] Test distribution
- [ ] Launch public beta

---

## Technical Specifications

### Stack
- **Frontend**: Svelte 4 + TypeScript
- **Backend**: Tauri 2.x + Rust
- **Database**: SQLite (via tauri-plugin-sql)
- **Storage**: Tauri Store (macOS Keychain)
- **UI Framework**: TailwindCSS + shadcn/svelte
- **Charts**: Chart.js
- **Icons**: Lucide Svelte

### Dependencies
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-store": "^2.0.0",
    "@tauri-apps/plugin-sql": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-notification": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "svelte": "^4.0.0",
    "chart.js": "^4.0.0",
    "lucide-svelte": "^0.300.0"
  }
}
```

### Build Configuration
```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "bundle": {
    "identifier": "com.tweetscrolls.desktop",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns"],
    "targets": ["dmg", "app"],
    "category": "Productivity",
    "shortDescription": "Private Twitter archive manager",
    "longDescription": "Analyze your Twitter archive privately on your Mac. Import, explore, and export your most valuable threads without ever sending data to a server."
  },
  "plugins": {
    "sql": {
      "preload": ["dist/db.js"]
    }
  }
}
```

---

## Success Metrics

### User Adoption
- [ ] 1,000 downloads in first month
- [ ] 50% conversion from trial to full use
- [ ] 4.5+ star rating on Mac App Store

### Engagement
- [ ] Average session: 15+ minutes
- [ ] 70% of users import their archive
- [ ] 40% of users export filtered threads

### Performance
- [ ] App launch < 2 seconds
- [ ] Import 10,000 threads < 5 seconds
- [ ] Search 50,000 threads < 100ms
- [ ] Memory usage < 200MB

### Privacy
- [ ] 0 server-side data collection
- [ ] 100% client-side operations
- [ ] External security audit passed

---

## Roadmap

### v1.0 (MVP)
- Google authentication
- Archive import
- Thread viewer
- Basic search/filter
- CSV export

### v1.1
- Analytics dashboard
- Custom scoring rules
- Tagging system
- Keyboard shortcuts

### v1.2
- iCloud sync (optional)
- Multiple archive management
- Thread comparison
- Advanced search (regex)

### v2.0
- Multi-device support
- AI-powered insights
- Network analysis
- Integration with other tools

---

## 🚀 Getting Started

```bash
# 1. Create Tauri app
npm create tauri-app@latest tweet-scrolls-macbook

# 2. Select options
# • Svelte
# • TypeScript
# • ESLint
# • Prettier

# 3. Navigate to project
cd tweet-scrolls-macbook

# 4. Install additional dependencies
npm install chart.js lucide-svelte
npm install -D @types/chart.js

# 5. Set up Google OAuth
# Create OAuth 2.0 client at console.cloud.google.com
# Use custom URL scheme: com.tweetscrolls.desktop://auth

# 6. Start development
npm run tauri dev

# 7. Build for macOS
npm run tauri build
```

---

**Version**: 1.0  
**Created**: 2026-03-12  
**Status**: Ready for development  
**Estimated Timeline**: 6 weeks to launch
