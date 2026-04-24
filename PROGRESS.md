# PROGRESS.md — Muon SSH Rust/Tauri Rewrite

Last updated: 2026-04-24 (Session 8)

## Session Summary

**Completed:** Phases 1-7 (all core), Phase 9 (complete), Phase 2.8 (remote port forwarding), Phase 4.9 (local terminal), Phase 10.1-10.4 (OS integration), Phase 11.1 (tests)
**Session 8 delivered:** Native menus, system tray, window management, streaming uploads, keyboard-interactive auth, 80 new unit tests
**Session 8 commits:** See git log
**Test count:** 109

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

**Status: COMPLETE (12/12)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | Add russh + russh-sftp dependencies | DONE | |
| 2.2 | SSH connection struct (connect/disconnect) | DONE | |
| 2.3 | Authentication engine (password, pubkey) | DONE | |
| 2.4 | Host key verification (known_hosts) | DONE | session 6: fixed bracketed port parsing, added tests |
| 2.5 | Shell channel (PTY, xterm-256color) | DONE | |
| 2.6 | Proxy support (HTTP CONNECT, SOCKS5) | DONE | |
| 2.7 | Jump host tunneling (multi-hop) | DONE | |
| 2.8 | Port forwarding (local -L, remote -R) | DONE | session 7: remote via tcpip_forward + cancel_tcpip_forward |
| 2.9 | X11 forwarding | TODO | |
| 2.10 | Keep-alive & compression | DONE | session 6: compression wired via Preferred.compression |
| 2.11 | Connection pool | DONE | session 6: integrated into AppState, cleanup on disconnect |
| 2.12 | Unit tests | DONE | session 6: 27 tests across 6 modules |

### What was built (session 8):

- **Keyboard-interactive auth** (`auth.rs`, `connection.rs`):
  - `AuthMethod::KeyboardInteractive { responses: Vec<String> }` variant added
  - Uses `authenticate_keyboard_interactive_start` + `authenticate_keyboard_interactive_respond` API from russh 0.60
  - Pre-configured responses used to answer server prompts
  - Full support in both direct and proxy connections
  - Jump host connections return clear error for unsupported keyboard-interactive
  - `ssh_cmds.rs`: Updated to use `session_info.auth_type` for routing auth method selection
  - `AuthType::KeyboardInteractive` already existed in `SessionInfo` from earlier sessions

- **Passphrase-protected keys** (`connection.rs`):
  - `load_key_pair_with_passphrase(path, passphrase)` added alongside `load_key_pair`
  - `PublicKey` auth now properly uses passphrase when provided
  - Applies to both direct and proxy connection paths

- **Streaming uploads** (`engine.rs`):
  - `perform_upload` rewritten: opens remote file via `SftpSession::create()`, writes chunks via `AsyncWrite`
  - No longer buffers entire file in memory — uses 32KB chunk streaming
  - Progress tracked per-chunk during write instead of after buffering
  - Proper `flush()` + `shutdown()` on remote file completion

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

---

## Phase 4: Terminal Integration

**Status: COMPLETE (10/10)**

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
| 4.9 | Local terminal (portable-pty) | DONE | session 7: full local PTY support |
| 4.10 | Copy/paste | DONE | |

---

## Phase 5: SFTP & File Browser

**Status: COMPLETE (13/13)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | SFTP filesystem (russh-sftp) | DONE | |
| 5.2 | Local filesystem adapter | DONE | |
| 5.3 | FileSystem trait | DONE | |
| 5.4 | File transfer engine | DONE | |
| 5.5 | Background transfers | DONE | |
| 5.6 | File browser UI | DONE | |
| 5.7 | Address bar | DONE | |
| 5.8 | Context menus | DONE | session 6 |
| 5.9 | Drag and drop | DONE | |
| 5.10 | Transfer queue UI | DONE | |
| 5.11 | Archive operations | DONE | session 6 |
| 5.12 | Remote file editing | DONE | session 6 |
| 5.13 | Sudo fallback | DONE | session 7 |

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
| 7.1 | Settings struct expansion | DONE | session 7: added external_editor field |
| 7.2 | Settings dialog | DONE | session 7: added Editor tab |
| 7.3 | Theme system | DONE | |
| 7.4 | Keyboard shortcuts | DONE | session 6 |
| 7.5 | External editors | DONE | session 7 |

---

## Phase 8: Plugin System

**Status: NOT STARTED**

All 7 tasks TODO.

## Phase 9: Internationalization

**Status: COMPLETE (5/5)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 9.1 | i18n framework (JSON-based, lazy loading) | DONE | |
| 9.2 | Extract all strings (~100 keys) | DONE | |
| 9.3 | Translate to 7 languages | DONE | |
| 9.4 | Language selector (in Settings dialog) | DONE | |
| 9.5 | RTL support | DONE | session 7 |

---

## Phase 10: OS Integration & Packaging

**Status: PARTIAL (4/8)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 10.1 | Native menus | DONE | session 8: File, Edit, Session, View, Tools, Help menus with shortcuts |
| 10.2 | System tray | DONE | session 8: tray icon with Show Window / Quit, left-click to show |
| 10.3 | File type associations | TODO | |
| 10.4 | Window management | DONE | session 8: save/restore position+size, close-to-tray (hide instead of quit) |
| 10.5 | Auto-updater | TODO | |
| 10.6 | Windows packaging | TODO | |
| 10.7 | Linux packaging | TODO | |
| 10.8 | GitHub Actions CI | TODO | |

### What was built (session 8):

- **Native menus** (`menu.rs`):
  - `File`: New Session, Import Sessions, Close Tab, Quit
  - `Edit`: Copy, Paste, Select All, Settings
  - `Session`: Connect, Disconnect, Duplicate, Delete
  - `View`: Toggle Sidebar, Local Terminal, Zoom In/Out/Reset, Fullscreen
  - `Tools`: File Browser, Process Viewer, Log Viewer, Disk Analyzer, Search, Port Forwarding, Port Viewer, SSH Key Manager
  - `Help`: About, Check for Updates
  - Menu events forwarded to frontend via `menu-event` Tauri event
  - `AppShell.svelte`: Full `menu-event` listener that routes menu IDs to UI actions

- **System tray** (`menu.rs`):
  - `TrayIconBuilder` with app icon, tooltip "Muon SSH"
  - Right-click menu: Show Window, Quit
  - Left-click: shows and focuses main window
  - Enabled `tray-icon` and `image-png` features in tauri

- **Window management** (`main.rs`):
  - `WindowBounds` struct: x, y, width, height persisted to `~/.config/muon-ssh/window_bounds.json`
  - `load_window_bounds()` on startup: restores size and position
  - `save_window_bounds_on_close()` on close requested: saves current geometry
  - Close-to-tray: `api.prevent_close()` + `window.hide()` instead of quitting
  - Tray "Quit" item calls `app.exit(0)` for actual exit
  - Tauri capabilities updated with window permissions

---

## Phase 11: Polish & Testing

**Status: PARTIAL (2/6)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 11.1 | Unit tests | DONE | session 8: 109 tests (up from 29) |
| 11.2 | Frontend E2E tests | TODO | |
| 11.3 | Error handling audit | TODO | |
| 11.4 | Performance profiling | TODO | |
| 11.5 | Accessibility | TODO | |
| 11.6 | Documentation | TODO | |

### Test breakdown (109 tests):

- `config::settings::tests` (3)
- `config::paths::tests` (11) — NEW session 8
- `credentials::store::tests` (12) — NEW session 8
- `file_transfer::progress::tests` (13) — NEW session 8
- `file_transfer::engine::tests` (17) — NEW session 8
- `session::store::tests` (6)
- `session::import::tests` (3)
- `session::folder::tests` (9) — NEW session 8
- `snippets::tests` (3)
- `ssh::auth::tests` (4)
- `ssh::connection::tests` (5)
- `ssh::host_keys::tests` (5)
- `ssh::port_forward::tests` (10) — NEW session 8
- `ssh::proxy::tests` (5) — NEW session 8
- `ssh::channel::tests` (3) — NEW session 8

---

## Technical Debt / Known Issues

1. ~~**Terminal read loop:** Not implemented~~ **FIXED**
2. ~~**Host key verification:** Always returns `Unknown`~~ **FIXED**
3. ~~**Snippet panel:** Not implemented~~ **FIXED**
4. ~~**Reconnection UI:** Not implemented~~ **FIXED**
5. ~~**File transfer progress:** All-or-nothing~~ **FIXED**
6. ~~**Keyboard-interactive auth:** Not yet implemented~~ **FIXED** session 8
7. ~~**Passphrase-protected keys:** `load_secret_key` called with `None`~~ **FIXED** session 8: passphrase now passed through
8. **Read loop contention:** Uses `Arc<Mutex<Channel>>` with 100ms timeout polling — acceptable for terminal but could be improved
9. **SFTP channel:** Opens a new SSH session channel for SFTP — should reuse connection via pool (now possible)
10. ~~**Transfer upload buffering:** Upload reads entire file into memory~~ **FIXED** session 8: streaming via AsyncWrite
11. ~~**Remote exec uses shell channel:**~~ **FIXED**
12. ~~**Remote port forwarding:** Stub only~~ **FIXED** session 7
13. ~~**Credential store is file-based, not OS keyring:**~~ **FIXED** session 6
14. **Jump host auth:** Jump hosts only support password with empty string or pubkey — should integrate with credential store
15. **X11 forwarding:** Not implemented (Phase 2.9)
16. ~~**Connection pool not integrated:**~~ **FIXED** session 6
17. ~~**Compression not wired:**~~ **FIXED** session 6

---

## Next Session

### Remaining work by priority:

| # | Item | What's needed |
|---|------|---------------|
| A | **X11 forwarding** (2.9) | Request X11 channel, forward to Unix socket |
| B | **Phase 8: Plugin system** | WASM runtime via wasmtime |
| C | **Phase 10: OS integration** (remaining) | File associations, auto-updater, packaging, CI |
| D | **Phase 11: Polish** (remaining) | E2E tests, error audit, performance, accessibility |
| E | **Jump host credential integration** | Use credential store for jump host passwords |
| F | **GitHub Actions CI** | Build matrix: Linux, Windows |

**Estimated complexity:** High — remaining items are either new subsystems (X11, plugins, packaging) or require deeper integration work.

### Test Count: 109

- `config::settings::tests` (3)
- `config::paths::tests` (11)
- `credentials::store::tests` (12)
- `file_transfer::progress::tests` (13)
- `file_transfer::engine::tests` (17)
- `session::store::tests` (6)
- `session::import::tests` (3)
- `session::folder::tests` (9)
- `snippets::tests` (3)
- `ssh::auth::tests` (4)
- `ssh::connection::tests` (5)
- `ssh::host_keys::tests` (5)
- `ssh::port_forward::tests` (10)
- `ssh::proxy::tests` (5)
- `ssh::channel::tests` (3)
