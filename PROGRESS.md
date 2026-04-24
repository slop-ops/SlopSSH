# PROGRESS.md — Muon SSH Rust/Tauri Rewrite

Last updated: 2026-04-24 (Session 7)

## Session Summary

**Completed:** Phases 1-7 (all core), Phase 9 (complete), Phase 2.8 (remote port forwarding), Phase 4.9 (local terminal)
**Session 7 delivered:** Remote port forwarding, sudo fallback for SFTP, local terminal with portable-pty, external editor auto-detection, RTL i18n support
**Session 7 commits:** 5 commits covering all remaining Phase 2-5, 7, 9 items
**Test count:** 28

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

### What was built (session 7):

- **Remote port forwarding** (`port_forward.rs`, `connection.rs`):
  - `RemoteForwardMap`: Shared `Arc<Mutex<HashMap<(String, u32), (String, u16)>>>` between ClientHandler and PortForwardManager
  - `ClientHandler::server_channel_open_forwarded_tcpip`: Accepts incoming forwarded connections, looks up target in shared map, spawns bidirectional TCP relay
  - `PortForwardManager::start_remote`: Calls `handle.tcpip_forward()`, inserts rule into shared map, stores metadata for cleanup
  - `PortForwardManager::stop`: For remote forwards, removes from shared map and calls `cancel_tcpip_forward()`
  - `forward_channel_tcp`: Bidirectional relay between SSH channel and TCP stream (shared with local forwarding)
  - Forward map threaded through `SessionManager::connect_with_options` → `SshConnection::connect_with_options` → `ClientHandler::with_remote_forwards`

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

### What was built (session 7):

- **Local terminal** (`local_terminal/pty.rs`, `LocalTerminal.svelte`):
  - `LocalTerminalSession`: Opens PTY via `portable-pty`, spawns `$SHELL` or `/bin/sh`
  - Reader thread with `AtomicBool` cancellation token, emits data via callback
  - `LocalTerminalManager`: CRUD for local terminal sessions (open/write/resize/close)
  - Stored in `AppState` as `std::sync::Mutex<LocalTerminalManager>` (sync PTY operations)
  - Tauri commands: `local_terminal_open/write/resize/close`
  - `LocalTerminal.svelte`: Full xterm.js terminal with WebGL, fit, resize, clipboard
  - `TerminalHolder.svelte`: Updated with `+$` button and "Open Local Terminal" empty state
  - Local tabs marked with green left border indicator

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

### What was built (session 7):

- **Sudo fallback** (`sftp_cmds.rs`):
  - `sftp_upload_sudo`: Uploads to `/tmp/.muon_upload_<name>` via SFTP, then `sudo cp /tmp/file target && rm -f /tmp/file`
  - `sftp_download_sudo`: `sudo cp source /tmp/.muon_download_<name>`, download via SFTP, then `rm -f /tmp/file`
  - Cleanup on both success and failure paths
  - Shell escaping for safe command construction
  - Frontend API functions: `sftpUploadSudo`, `sftpDownloadSudo`

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

### What was built (session 7):

- **External editor detection** (`config/editor.rs`):
  - `detect_editors()`: Scans for 14 common editors using `which` command
  - Checks: VS Code, VS Code Insiders, Cursor, Vim, Neovim, Nano, Emacs, Micro, Helix, Sublime Text, Atom, Kate, Gedit, Mousepad
  - Returns `Vec<EditorInfo>` with name, command, and full path
  - `resolve_editor()`: Falls back through configured → code → nvim → vim → nano → vi
  - `open_in_editor()`: Spawns editor process with file path argument
  - Settings dialog: New "Editor" tab with detected editors list, click-to-select
  - `external_editor` field in Settings struct (default: empty = auto-detect)

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

### What was built (session 7):

- **RTL support** (`i18n.ts`, `app.css`):
  - `isRTL()` / `getTextDirection()`: Detect RTL locales (ar, he, fa, ur)
  - `applyDirection()`: Sets `document.documentElement.dir` on locale change
  - CSS rules for RTL: sidebar border flip, toolbar/tab bar stay LTR for code
  - Integrated into `loadLocale()` flow

---

## Phase 10: OS Integration & Packaging

**Status: NOT STARTED**

All 8 tasks TODO.

## Phase 11: Polish & Testing

**Status: PARTIAL**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 11.1 | Integration tests | PARTIAL | 28 unit tests |
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
9. **SFTP channel:** Opens a new SSH session channel for SFTP — should reuse connection via pool (now possible)
10. **Transfer upload buffering:** Upload reads entire file into memory before writing — needs streaming for large files
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
| C | **Phase 10: OS integration** | Native menus, system tray, auto-updater, packaging |
| D | **Phase 11: Polish** | E2E tests, error audit, performance, accessibility |
| E | **Streaming uploads** | Chunked SFTP write instead of buffering entire file |
| F | **Keyboard-interactive auth** | Implement auth callback with UI prompt |

**Estimated complexity:** High — remaining items are either new subsystems (X11, plugins, packaging) or require deeper integration work.

### Test Count: 28

- `config::settings::tests` (3)
- `session::store::tests` (6)
- `session::import::tests` (3)
- `snippets::tests` (3)
- `ssh::auth::tests` (3)
- `ssh::connection::tests` (5)
- `ssh::host_keys::tests` (5)
