# PROGRESS.md — Muon SSH Rust/Tauri Rewrite

Last updated: 2026-04-24 (Session 6)

## Session Summary

**Completed:** Phases 1-6 (core), Phase 7 (complete), Phase 9 (core), Phase 2.12 (tests), Phase 5.8-5.12 (partial)
**Session 6 delivered:** Connection pool integration, compression wiring, OS keyring credentials, keyboard shortcuts, 27 unit tests, context menus, archive operations, remote file editor
**Session 6 commits:** 8 commits covering all session 5 leftovers + new features

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

**Status: 10/12 DONE, 1 PARTIAL, 1 TODO**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | Add russh + russh-sftp dependencies | DONE | |
| 2.2 | SSH connection struct (connect/disconnect) | DONE | |
| 2.3 | Authentication engine (password, pubkey) | DONE | |
| 2.4 | Host key verification (known_hosts) | DONE | session 6: fixed bracketed port parsing, added tests |
| 2.5 | Shell channel (PTY, xterm-256color) | DONE | |
| 2.6 | Proxy support (HTTP CONNECT, SOCKS5) | DONE | |
| 2.7 | Jump host tunneling (multi-hop) | DONE | |
| 2.8 | Port forwarding (local -L, remote -R) | PARTIAL | local done, remote is stub |
| 2.9 | X11 forwarding | TODO | |
| 2.10 | Keep-alive & compression | DONE | session 6: compression wired via Preferred.compression |
| 2.11 | Connection pool | DONE | session 6: integrated into AppState, cleanup on disconnect |
| 2.12 | Unit tests | DONE | session 6: 27 tests across 6 modules |

### What was built (session 6):

- **Compression wiring** (`connection.rs`):
  - When `enable_compression` is true, `Preferred.compression` lists zlib, zlib@openssh.com, then none
  - Wired in both direct and proxy connection paths
  - `enable_compression` added to Settings struct (default: false)
  - Passed through `SessionManager::connect()` from AppState settings

- **Connection pool integration** (`pool.rs`, `state.rs`, `ssh_cmds.rs`):
  - Pool redesigned to return `Arc<Handle<ClientHandler>>` for actual use
  - Added to AppState with max 3 connections per session
  - SSH disconnect cleans up pooled connections and SFTP sessions
  - `close_session()` and `close_all()` methods for cleanup

- **Unit tests** (27 total):
  - Settings: defaults, serialization roundtrip, missing field fallbacks
  - Host keys: simple/wildcard/bracketed-port/multi-pattern/revoked matching
  - Sessions: CRUD operations, nested folders, serialization
  - Snippets: serialization, optional fields, roundtrip
  - Auth: all three method variants
  - Connection: options defaults/clone, error display, client handler

---

## Phase 3: Session Management

**Status: COMPLETE**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | SessionInfo struct (complete) | DONE | |
| 3.2 | SessionFolder tree (recursive) | DONE | |
| 3.3 | SessionStore (JSON persistence, CRUD) | DONE | |
| 3.4 | Session import (SSH config) | DONE | |
| 3.5 | Credential store | DONE | session 6: OS keyring with file fallback |
| 3.6 | Credential cache (in-memory) | DONE | |
| 3.7 | Tauri IPC commands (full CRUD) | DONE | |

### What was built (session 6):

- **OS keyring credential store** (`credentials/store.rs`):
  - `CredentialBackend` trait with `save`, `get`, `delete` methods
  - `KeyringBackend`: Uses `keyring` crate for OS-native storage (macOS Keychain, Windows Credential Manager, Linux Secret Service)
  - `FileBackend`: JSON file fallback for systems without keyring support
  - `CredentialStore`: Auto-detects keyring availability at startup, falls back gracefully
  - Updated AppState to hold `CredentialStore` instance
  - IPC commands now route through CredentialStore instance

---

## Phase 4: Terminal Integration

**Status: COMPLETE (core, 9/10)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 4.1 | xterm.js setup | DONE | |
| 4.2 | Terminal.svelte component | DONE | |
| 4.3 | Terminal themes (dark/light) | DONE | |
| 4.4 | PTY data bridge (Tauri events) | DONE | |
| 4.5 | Terminal session manager (Rust-side) | DONE | |
| 4.6 | Terminal tabs UI | DONE | |
| 4.7 | Snippet panel | DONE | |
| 4.8 | Reconnection UI | DONE | |
| 4.9 | Local terminal (portable-pty) | TODO | |
| 4.10 | Copy/paste | DONE | |

---

## Phase 5: SFTP & File Browser

**Status: 12/13 DONE**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | SFTP filesystem (russh-sftp) | DONE | |
| 5.2 | Local filesystem adapter | DONE | |
| 5.3 | FileSystem trait | DONE | |
| 5.4 | File transfer engine | DONE | |
| 5.5 | Background transfers | DONE | |
| 5.6 | File browser UI | DONE | |
| 5.7 | Address bar | DONE | |
| 5.8 | Context menus | DONE | session 6: ContextMenu component + right-click actions |
| 5.9 | Drag and drop | DONE | |
| 5.10 | Transfer queue UI | DONE | |
| 5.11 | Archive operations | DONE | session 6: tar.gz/bz2/zip create + extract |
| 5.12 | Remote file editing | DONE | session 6: inline text editor with save |
| 5.13 | Sudo fallback | TODO | |

### What was built (session 6):

- **Context menus** (`ContextMenu.svelte`, `FileBrowser.svelte`):
  - Reusable ContextMenu component with auto-positioning, click-outside/Escape close
  - File context: Open (dirs), Edit (files), Rename, Delete, Extract (archives), Archive (dirs)
  - Empty space context: New Folder, Refresh
  - FileList rows support `onContextMenu` callback

- **Archive operations** (`tools_cmds.rs`):
  - `archive_create`: Creates archives via remote `tar`/`zip` commands
  - `archive_extract`: Extracts based on file extension, auto-creates target dir
  - Supports: tar.gz, tar.bz2, tar, zip formats
  - Shell escaping for safe command construction
  - Frontend API functions + context menu integration

- **Remote file editor** (`FileEditor.svelte`):
  - Full-screen modal editor for remote files
  - Reads content via SFTP, displays in monospace textarea
  - Save with Ctrl+S or button, re-uploads via SFTP write
  - Unsaved changes detection with close confirmation
  - Accessible via right-click "Edit" on files

---

## Phase 6: Tools & Utilities

**Status: COMPLETE (10/10)**

| # | Task | Status |
|---|------|--------|
| 6.1 | Process viewer | DONE |
| 6.2 | Log viewer | DONE |
| 6.3 | Disk analyzer | DONE |
| 6.4 | Search panel | DONE |
| 6.5 | System info | DONE |
| 6.6 | System load | DONE |
| 6.7 | Port viewer | DONE |
| 6.8 | SSH key manager | DONE |
| 6.9 | Bundled scripts | DONE |
| 6.10 | Port forwarding UI | DONE |

---

## Phase 7: Settings & Preferences

**Status: COMPLETE (5/5)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 7.1 | Settings struct expansion | DONE | session 6: added enable_compression, keyboard_shortcuts |
| 7.2 | Settings dialog | DONE | |
| 7.3 | Theme system | DONE | |
| 7.4 | Keyboard shortcuts | DONE | session 6 |
| 7.5 | External editors | TODO | not started |

### What was built (session 6):

- **Keyboard shortcuts** (`shortcuts.ts`, `AppShell.svelte`):
  - 15 default key bindings: new/close/next/prev tab, toggle sidebar, open settings, new session, toggle files/tools, font size, fullscreen, escape
  - `registerHandler()` for component-level shortcut handling
  - Action routing in AppShell handles all shortcut actions
  - Shortcuts disabled when dialogs are open
  - `keyboard_shortcuts` field in Settings for persistence
  - `serializeShortcuts`/`deserializeShortcuts` for custom shortcuts

---

## Phase 8: Plugin System

**Status: NOT STARTED**

All 7 tasks TODO.

## Phase 9: Internationalization

**Status: COMPLETE (core, 4/5)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 9.1 | i18n framework (JSON-based, lazy loading) | DONE | |
| 9.2 | Extract all strings (~100 keys) | DONE | |
| 9.3 | Translate to 7 languages | DONE | |
| 9.4 | Language selector (in Settings dialog) | DONE | |
| 9.5 | RTL support | TODO | |

---

## Phase 10: OS Integration & Packaging

**Status: NOT STARTED**

All 8 tasks TODO.

## Phase 11: Polish & Testing

**Status: PARTIAL (11.1 started)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 11.1 | Integration tests | PARTIAL | session 6: 27 unit tests |
| 11.2 | Frontend E2E tests | TODO | |
| 11.3 | Error handling audit | TODO | |
| 11.4 | Performance profiling | TODO | |
| 11.5 | Accessibility | TODO | |
| 11.6 | Documentation | TODO | |

---

## Technical Debt / Known Issues

1. ~~**Terminal read loop:** Not implemented~~ **FIXED**
2. ~~**Host key verification:** Always returns `Unknown`~~ **FIXED**
3. ~~**Snippet panel:** Not implemented~~ **FIXED**
4. ~~**Reconnection UI:** Not implemented~~ **FIXED**
5. ~~**File transfer progress:** All-or-nothing~~ **FIXED**
6. **Keyboard-interactive auth:** Not yet implemented
7. **Passphrase-protected keys:** `load_secret_key` is called with `None` for passphrase — needs UI integration
8. **Read loop contention:** Uses `Arc<Mutex<Channel>>` with 100ms timeout polling — acceptable for terminal but could be improved
9. **SFTP channel:** Opens a new SSH session channel for SFTP — should reuse connection via pool (blocked by ~~#16~~ now possible)
10. **Transfer upload buffering:** Upload reads entire file into memory before writing — needs streaming for large files
11. ~~**Remote exec uses shell channel:**~~ **FIXED**
12. **Remote port forwarding:** Stub only — `start_remote()` spawns an idle task. Needs russh `tcpip_forward` request implementation
13. ~~**Credential store is file-based, not OS keyring:**~~ **FIXED** session 6 — OS keyring with file fallback
14. **Jump host auth:** Jump hosts only support password with empty string or pubkey — should integrate with credential store for real passwords
15. **X11 forwarding:** Not implemented (Phase 2.9)
16. ~~**Connection pool not integrated:**~~ **FIXED** session 6 — in AppState, cleanup on disconnect
17. ~~**Compression not wired:**~~ **FIXED** session 6 — wired via Preferred.compression

---

## Next Session

### Remaining work by priority:

| # | Item | What's needed |
|---|------|---------------|
| A | **Remote port forwarding** (2.8) | Implement russh `tcpip_forward` request |
| B | **X11 forwarding** (2.9) | Request X11 channel, forward to Unix socket |
| C | **Local terminal** (4.9) | Add `portable-pty` crate, local PTY data bridge |
| D | **Sudo fallback** (5.13) | Transfer to /tmp then `sudo cp/mv` |
| E | **External editors** (7.5) | Auto-detect installed editors, manual path config |
| F | **RTL support** (9.5) | CSS direction: rtl for applicable locales |
| G | **Phase 8: Plugin system** | WASM runtime via wasmtime |
| H | **Phase 10: OS integration** | Native menus, system tray, auto-updater, packaging |
| I | **Phase 11: Polish** | E2E tests, error audit, performance, accessibility |

**Estimated complexity:** Medium-high — remaining items are either new subsystems (X11, plugins, packaging) or require deeper integration work.

### Test Count: 27

- `config::settings::tests` (3)
- `session::store::tests` (6)
- `session::import::tests` (3)
- `snippets::tests` (3)
- `ssh::auth::tests` (3)
- `ssh::connection::tests` (4)
- `ssh::host_keys::tests` (5)
