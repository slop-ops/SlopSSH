# PROGRESS.md — Muon SSH Rust/Tauri Rewrite

Last updated: 2026-04-23 (Session 1)

## Session Summary

**Completed:** Phases 1-4 (partial)
**Total commits:** 6
**Lines of code:** ~3,500 Rust, ~500 TypeScript/Svelte

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

### What was built:
- **muon-core** crate: config (paths, settings), session (info, folder, store), credentials (cache), snippets (manager), logging
- **muon-tauri** crate: Tauri 2 app with IPC commands
- **frontend**: Svelte 5 + Vite + TypeScript with @tauri-apps/api integration

---

## Phase 2: SSH Engine

**Status: CORE COMPLETE**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 2.1 | Add russh + russh-sftp dependencies | DONE | `5d08492` |
| 2.2 | SSH connection struct (connect/disconnect) | DONE | `5d08492` |
| 2.3 | Authentication engine (password, pubkey) | DONE | `5d08492` |
| 2.4 | Host key verification (known_hosts) | STUB | `5d08492` |
| 2.5 | Shell channel (PTY, xterm-256color) | DONE | `5d08492` |
| 2.6 | Proxy support (HTTP CONNECT, SOCKS5) | DONE | `5d08492` |
| 2.7 | Jump host tunneling (multi-hop) | TODO | |
| 2.8 | Port forwarding (local -L, remote -R) | STUB | `5d08492` |
| 2.9 | X11 forwarding | TODO | |
| 2.10 | Keep-alive & compression | TODO | |
| 2.11 | Connection pool | TODO | |
| 2.12 | Unit tests | TODO | |

### What was built:
- `SshConnection` wrapping russh `Handle<ClientHandler>` with connect/disconnect
- `AuthMethod` enum: Password, PublicKey (with RSA hash), None
- `ShellChannel` with PTY allocation (xterm-256color), shell, resize, read/write
- `ProxyConfig` with HTTP CONNECT (Basic auth) and SOCKS5 support
- `SessionManager` managing multiple SSH connections and shell channels
- All Tauri IPC: ssh_connect, ssh_disconnect, ssh_open_shell, ssh_write_shell, ssh_resize_shell

---

## Phase 3: Session Management

**Status: CORE COMPLETE**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 3.1 | SessionInfo struct (complete) | DONE | `fc9c0af` |
| 3.2 | SessionFolder tree (recursive) | DONE | `fc9c0af` |
| 3.3 | SessionStore (JSON persistence, CRUD) | DONE | `fc9c0af` |
| 3.4 | Session import (SSH config, legacy) | TODO | |
| 3.5 | Credential store (keyring) | TODO | |
| 3.6 | Credential cache (in-memory) | DONE | `fc9c0af` |
| 3.7 | Tauri IPC commands (full CRUD) | DONE | `559681a` |

---

## Phase 4: Terminal Integration

**Status: PARTIAL**

| # | Task | Status | Commit |
|---|------|--------|--------|
| 4.1 | xterm.js setup | DONE | `8364883` |
| 4.2 | Terminal.svelte component | DONE | `8364883` |
| 4.3 | Terminal themes (dark/light) | DONE | `8364883` |
| 4.4 | PTY data bridge (Tauri events) | PARTIAL | `8364883` |
| 4.5 | Terminal session manager (Rust-side) | DONE | `559681a` |
| 4.6 | Terminal tabs UI | TODO | |
| 4.7 | Snippet panel | TODO | |
| 4.8 | Reconnection UI | TODO | |
| 4.9 | Local terminal (portable-pty) | TODO | |
| 4.10 | Copy/paste | TODO | |

### What was built:
- Terminal.svelte with xterm.js, WebGL addon, FitAddon
- Dark/light terminal themes (One Dark inspired)
- API layer (invoke.ts) wrapping all Tauri IPC commands
- AppShell layout with sidebar + main content
- Sidebar with session list, password input, connect action
- `$lib` path alias configured

---

## Phase 5: SFTP & File Browser

**Status: NOT STARTED**

All 13 tasks TODO.

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

1. **Terminal read loop:** Currently uses `sshReadShell` which doesn't exist. Need to implement Tauri event-based terminal output (Rust emits `terminal-output` events with base64 data)
2. **Host key verification:** Always returns `Unknown` — needs known_hosts file parsing
3. **Keyboard-interactive auth:** Not yet implemented
4. **Passphrase-protected keys:** `load_secret_key` is called with `None` for passphrase — needs UI integration
5. **No error handling UI:** SSH errors shown in console only

---

## Next Session

**Priority 1:** Fix terminal data bridge (event-based output from Rust to frontend)
**Priority 2:** Host key verification with known_hosts parsing
**Priority 3:** Phase 4.6 — Terminal tabs UI
**Priority 4:** Phase 5 — SFTP & File browser
**Prerequisites:** All Phase 1-3 core tasks complete. Phase 4 partially done.
**Estimated complexity:** Medium-High — terminal data bridge is critical for a working app.
