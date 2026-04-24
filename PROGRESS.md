# PROGRESS.md — Muon SSH Rust/Tauri Rewrite

Last updated: 2026-04-24 (Session 4)

## Session Summary

**Completed:** Phases 1-5 (core), Phase 6 (complete), Phase 7 (partial), Phase 9 (core)
**Total commits:** 6 (session 1) + 2 (session 2) + N (session 3) + pending (session 4)
**Lines of code:** ~8,200 Rust, ~5,800 TypeScript/Svelte, ~1,200 i18n JSON

---

## Phase 1: Project Scaffolding & Core Infrastructure

**Status: COMPLETE**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 1.1 | Initialize Cargo workspace with muon-core and muon-tauri crates | DONE | `fc9c0af` |
| 1.2 | Initialize Tauri 2 project | DONE | `c0c4488` |
| 1.3 | Set up Svelte 5 + Vite frontend | DONE | `c0c4488` |
| 1.4 | Add core dependencies (tokio, serde, anyhow, thiserror, tracing, dirs) | DONE | `fc9c0af` |
| 1.5 | Config directory management (~/.config/muon-ssh/) | DONE | `fc9c0af` |
| 1.6 | Settings module (Settings struct + TOML persistence) | DONE | `fc9c0af` |
| 1.7 | Logging setup (tracing-subscriber) | DONE | `8acac96` |
| 1.8 | App context / state (AppState) | DONE | `c0c4488` |
| 1.9 | Tauri state management (state.rs) | DONE | `c0c4488` |
| 1.10 | Basic IPC ping (greet command) | DONE | `c0c4488` |

---

## Phase 2: SSH Engine

**Status: CORE COMPLETE**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 2.1 | Add russh + russh-sftp dependencies | DONE | `5d08492` |
| 2.2 | SSH connection struct (connect/disconnect) | DONE | `5d08492` |
| 2.3 | Authentication engine (password, pubkey) | DONE | `5d08492` |
| 2.4 | Host key verification (known_hosts) | DONE | session 2 |
| 2.5 | Shell channel (PTY, xterm-256color) | DONE | `5d08492` |
| 2.6 | Proxy support (HTTP CONNECT, SOCKS5) | DONE | `5d08492` |
| 2.7 | Jump host tunneling (multi-hop) | TODO | |
| 2.8 | Port forwarding (local -L, remote -R) | STUB | `5d08492` |
| 2.9 | X11 forwarding | TODO | |
| 2.10 | Keep-alive & compression | TODO | |
| 2.11 | Connection pool | TODO | |
| 2.12 | Unit tests | TODO | |

---

## Phase 3: Session Management

**Status: COMPLETE**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 3.1 | SessionInfo struct (complete) | DONE | `fc9c0af` |
| 3.2 | SessionFolder tree (recursive) | DONE | `fc9c0af` |
| 3.3 | SessionStore (JSON persistence, CRUD) | DONE | `fc9c0af` |
| 3.4 | Session import (SSH config, legacy) | TODO | |
| 3.5 | Credential store (keyring) | TODO | |
| 3.6 | Credential cache (in-memory) | DONE | `fc9c0af` |
| 3.7 | Tauri IPC commands (full CRUD) | DONE | session 2 |

---

## Phase 4: Terminal Integration

**Status: COMPLETE (core)**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 4.1 | xterm.js setup | DONE | `8364883` |
| 4.2 | Terminal.svelte component | DONE | `8364883` |
| 4.3 | Terminal themes (dark/light) | DONE | `8364883` |
| 4.4 | PTY data bridge (Tauri events) | DONE | session 2 |
| 4.5 | Terminal session manager (Rust-side) | DONE | session 2 |
| 4.6 | Terminal tabs UI | DONE | session 2 |
| 4.7 | Snippet panel | DONE | session 3 |
| 4.8 | Reconnection UI | DONE | session 3 |
| 4.9 | Local terminal (portable-pty) | TODO | |
| 4.10 | Copy/paste | TODO | |

---

## Phase 5: SFTP & File Browser

**Status: COMPLETE**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 5.1 | SFTP filesystem (russh-sftp) | DONE | session 2 |
| 5.2 | Local filesystem adapter | DONE | session 2 |
| 5.3 | FileSystem trait | DONE | session 2 |
| 5.4 | File transfer engine | DONE | session 3 |
| 5.5 | Background transfers | DONE | session 3 |
| 5.6 | File browser UI | DONE | session 2 |
| 5.7 | Address bar | DONE | session 2 |
| 5.8 | Context menus | PARTIAL | session 2 |
| 5.9 | Drag and drop | DONE | session 4 |
| 5.10 | Transfer queue UI | DONE | session 3 |
| 5.11 | Archive operations | TODO | |
| 5.12 | Remote file editing | TODO | |
| 5.13 | Sudo fallback | TODO | |

### What was built (session 4):
- **Drag and drop**: HTML5 DnD for file uploads in FileBrowser, drag-over visual feedback, base64 file content transfer
- **FileList drag**: Files are draggable with metadata JSON payload

---

## Phase 6: Tools & Utilities

**Status: COMPLETE (9/9)**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 6.1 | Process viewer | DONE | session 3 |
| 6.2 | Log viewer | DONE | session 3 |
| 6.3 | Disk analyzer | DONE | session 3 |
| 6.4 | Search panel | DONE | session 3 |
| 6.5 | System info | DONE | session 3 |
| 6.6 | System load | DONE | session 3 |
| 6.7 | Port viewer | DONE | session 3 |
| 6.8 | SSH key manager | DONE | session 4 |
| 6.9 | Bundled scripts | DONE | session 4 |

### What was built (session 4):
- **SSH Key Manager** (`muon-core/src/ssh/key_manager.rs`):
  - `list_local_keys()`: Scan ~/.ssh/ for private keys, detect type (OpenSSH, RSA PEM, EC, PKCS8), fingerprint
  - `list_remote_keys()`: Parse remote authorized_keys via SSH exec
  - `generate_key_pair()`: Generate Ed25519/RSA/ECDSA keys via ssh-keygen
  - `deploy_public_key()`: Upload public key to remote authorized_keys with dedup
  - `read_public_key()`: Read local .pub file
- **Key Manager IPC** (`key_cmds.rs`): 5 commands - list_local_keys, list_remote_keys, generate_key_pair, deploy_public_key, read_public_key
- **KeyManager.svelte**: Full key management UI with local/remote key listing, key generation dialog, deploy-to-remote action
- **Bundled Scripts** (`scripts/`):
  - `ps.sh`: Remote process listing (ps with user, pid, cpu, mem, vsz, rss)
  - `search.sh`: Remote file search (find + grep with name/content filters)
  - `linux-sysinfo.sh`: System info (OS, kernel, uptime, CPU, memory, disk, network)
- **ScriptLoader** (`muon-core/src/scripts/mod.rs`): `include_str!` bundled scripts with load_all() API

---

## Phase 7: Settings & Preferences

**Status: COMPLETE (3/5)**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 7.1 | Settings struct expansion | DONE | session 1 |
| 7.2 | Settings dialog | DONE | session 3 |
| 7.3 | Theme system | DONE | session 4 |
| 7.4 | Keyboard shortcuts | TODO | |
| 7.5 | External editors | TODO | |

### What was built (session 4):
- **Theme system** (CSS variables):
  - `app.css`: Complete dark/light theme with 30+ CSS custom properties (--bg-primary, --text-primary, --accent, --error, etc.)
  - `data-theme` attribute on `<html>` element for switching
  - All components migrated from hardcoded colors to CSS variables
  - Theme store (`lib/stores/theme.ts`): getTheme/setTheme/toggleTheme with localStorage persistence
  - Theme toggle button in toolbar (sun/moon icon)
  - Settings dialog applies theme on save
  - Scrollbar theming, selection colors, shadows all theme-aware

---

## Phase 8: Plugin System

**Status: NOT STARTED**

All 7 tasks TODO.

## Phase 9: Internationalization

**Status: COMPLETE (core)**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 9.1 | i18n framework (JSON-based, lazy loading) | DONE | session 4 |
| 9.2 | Extract all strings (~100 keys) | DONE | session 4 |
| 9.3 | Translate to 7 languages | DONE | session 4 |
| 9.4 | Language selector (in Settings dialog) | DONE | session 3 |
| 9.5 | RTL support | TODO | |

### What was built (session 4):
- **i18n framework** (`lib/utils/i18n.ts`):
  - `t(path, params)` function for dot-notation key lookup (e.g., `t('sidebar.sessions')`)
  - Lazy locale loading via dynamic `import()`
  - Fallback to English for missing keys
  - Parameter interpolation with `{key}` syntax
  - `useTranslations()` hook for components
- **Translation files** (7 languages):
  - `en.json` (English - base): ~100 keys across app/sidebar/toolbar/session/terminal/files/tools/settings/common
  - `es.json` (Spanish)
  - `ru.json` (Russian)
  - `fr.json` (French)
  - `de.json` (German)
  - `pt.json` (Portuguese)
  - `cn.json` (Chinese)

---

## Phase 10: OS Integration & Packaging

**Status: NOT STARTED**

All 8 tasks TODO.

## Phase 11: Polish & Testing

**Status: NOT STARTED**

All 6 tasks TODO.

---

## Technical Debt / Known Issues

1. ~~**Terminal read loop:** Not implemented~~ **FIXED** — Event-based terminal output with read loop
2. ~~**Host key verification:** Always returns `Unknown`~~ **FIXED** — Full known_hosts parsing and verification
3. ~~**Snippet panel:** Not implemented~~ **FIXED** — Full CRUD snippet panel with search
4. ~~**Reconnection UI:** Not implemented~~ **FIXED** — Disconnect overlay with reconnect button
5. ~~**File transfer progress:** All-or-nothing~~ **FIXED** — Chunked transfer with progress events
6. **Keyboard-interactive auth:** Not yet implemented
7. **Passphrase-protected keys:** `load_secret_key` is called with `None` for passphrase — needs UI integration
8. **Read loop contention:** Uses `Arc<Mutex<Channel>>` with 100ms timeout polling — acceptable for terminal but could be improved
9. **SFTP channel:** Opens a new SSH session channel for SFTP — should reuse connection
10. **Transfer upload buffering:** Upload reads entire file into memory before writing — needs streaming for large files
11. ~~**Remote exec uses shell channel:**~~ **FIXED** — Now uses proper exec channel via `channel.exec()`

---

## Next Session

**Priority 1:** Phase 2.7 — Jump host tunneling (multi-hop SSH)
**Priority 2:** Phase 2.8 — Port forwarding implementation (beyond stub)
**Priority 3:** Phase 2.10-2.11 — Keep-alive, compression, connection pool
**Priority 4:** Phase 3.4-3.5 — Session import (SSH config) and keyring credential store
**Prerequisites:** SSH engine core stable, tools fully functional, theme system working.
**Estimated complexity:** High — jump host and port forwarding require careful russh channel management.
