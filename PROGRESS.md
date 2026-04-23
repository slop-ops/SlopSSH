# PROGRESS.md — Muon SSH Rust/Tauri Rewrite

Last updated: 2026-04-24 (Session 2)

## Session Summary

**Completed:** Phases 1-4 (core), Phase 5 (core), partial Phase 2
**Total commits:** 6 (session 1) + 1 + pending
**Lines of code:** ~5,400 Rust, ~1,500 TypeScript/Svelte

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

### What was built (session 2):
- `HostKeyVerifier` with known_hosts parsing (plain, [host]:port, wildcard patterns)
- `ClientHandler` now carries host/port for server key verification
- Unknown hosts auto-accepted and added to known_hosts
- Changed hosts rejected

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

### What was built (session 2):
- `update_session`, `delete_session`, `create_folder` IPC commands
- Recursive session tree manipulation (remove from tree, re-add to folder)
- Frontend NewSessionDialog.svelte for creating sessions

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
| 4.7 | Snippet panel | TODO | |
| 4.8 | Reconnection UI | TODO | |
| 4.9 | Local terminal (portable-pty) | TODO | |
| 4.10 | Copy/paste | TODO | |

### What was built (session 2):
- **Terminal data bridge**: ShellChannel restructured with `Arc<Mutex<Channel>>` for concurrent read/write
- **Read loop**: `spawn_read_loop()` spawns tokio task with 100ms timeout polling and callback
- **Event emission**: `ssh_open_shell` emits `terminal-output-{channelId}` events with base64 data
- **TerminalHolder.svelte**: Closable tab container with active tab switching
- **SessionManager cleanup**: Read loop handles abort, shell close on disconnect

---

## Phase 5: SFTP & File Browser

**Status: CORE COMPLETE**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 5.1 | SFTP filesystem (russh-sftp) | DONE | session 2 |
| 5.2 | Local filesystem adapter | DONE | session 2 |
| 5.3 | FileSystem trait | DONE | session 2 |
| 5.4 | File transfer engine | TODO | |
| 5.5 | Background transfers | TODO | |
| 5.6 | File browser UI | DONE | session 2 |
| 5.7 | Address bar | DONE | session 2 |
| 5.8 | Context menus | PARTIAL | session 2 |
| 5.9 | Drag and drop | TODO | |
| 5.10 | Transfer queue UI | TODO | |
| 5.11 | Archive operations | TODO | |
| 5.12 | Remote file editing | TODO | |
| 5.13 | Sudo fallback | TODO | |

### What was built (session 2):
- **FileSystem trait** (`muon-core/src/filesystem/types.rs`): Common async interface for list_dir, stat, mkdir, remove, rename, read_file, write_file, exists
- **LocalFileSystem** (`local.rs`): `tokio::fs` wrapper implementing FileSystem trait, sorted directory listing
- **RemoteFileSystem** (`remote.rs`): russh-sftp wrapper implementing FileSystem trait, with session management
- **SFTP Tauri IPC** (`sftp_cmds.rs`): 10 commands - connect, disconnect, list_dir, mkdir, remove, rename, read_file, write_file, stat, home
- **FileBrowser.svelte**: Remote file browser with breadcrumb navigation, path editing, directory operations
- **FileList.svelte**: Table-based file listing with icon, name, size, modified, rename/delete actions
- **Frontend SFTP API**: Full set of invoke wrappers for all SFTP commands

---

## Phase 6: Tools & Utilities

**Status: NOT STARTED**

All 9 tasks TODO.

## Phase 7: Settings & Preferences

**Status: NOT STARTED**

All 5 tasks TODO.

## Phase 8: Plugin System

**Status: NOT STARTED**

All 7 tasks TODO.

## Phase 9: Internationalization

**Status: NOT STARTED**

All 5 tasks TODO.

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
3. **Keyboard-interactive auth:** Not yet implemented
4. **Passphrase-protected keys:** `load_secret_key` is called with `None` for passphrase — needs UI integration
5. **Read loop contention:** Uses `Arc<Mutex<Channel>>` with 100ms timeout polling — acceptable for terminal but could be improved
6. **SFTP channel:** Opens a new SSH session channel for SFTP — should reuse connection
7. **No file transfer progress:** SFTP read/write is all-or-nothing — needs streaming with progress events

---

## Next Session

**Priority 1:** Phase 4.7 — Snippet panel (send commands to terminal)
**Priority 2:** Phase 4.8 — Reconnection UI (disconnected overlay)
**Priority 3:** Phase 5.4-5.5 — File transfer engine with progress tracking
**Priority 4:** Phase 6.1-6.2 — Process viewer and log viewer tools
**Prerequisites:** Terminal data bridge, SFTP core, and session CRUD all working.
**Estimated complexity:** Medium — snippet panel and reconnection UI are straightforward; transfer engine needs design.
