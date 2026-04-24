# PROGRESS.md — Muon SSH Rust/Tauri Rewrite

Last updated: 2026-04-24 (Session 5)

## Session Summary

**Completed:** Phases 1-6 (core), Phase 7 (partial), Phase 9 (core)
**Session 5 delivered:** Jump host tunneling, local port forwarding, keep-alive, SSH config import, file-based credential store, terminal copy/paste
**Session 5 incomplete:** Compression wiring, connection pool integration into app, OS keyring, keyboard shortcuts

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

**Status: 7/12 DONE, 2 PARTIAL, 3 TODO**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | Add russh + russh-sftp dependencies | DONE | `5d08492` |
| 2.2 | SSH connection struct (connect/disconnect) | DONE | |
| 2.3 | Authentication engine (password, pubkey) | DONE | |
| 2.4 | Host key verification (known_hosts) | DONE | |
| 2.5 | Shell channel (PTY, xterm-256color) | DONE | |
| 2.6 | Proxy support (HTTP CONNECT, SOCKS5) | DONE | |
| 2.7 | Jump host tunneling (multi-hop) | DONE | session 5 — see below |
| 2.8 | Port forwarding (local -L, remote -R) | PARTIAL | session 5 — local done, remote is stub |
| 2.9 | X11 forwarding | TODO | |
| 2.10 | Keep-alive & compression | PARTIAL | session 5 — keep-alive done, compression NOT wired into russh Config |
| 2.11 | Connection pool | PARTIAL | session 5 — module written but NOT integrated into SessionManager/AppState |
| 2.12 | Unit tests | TODO | |

### What was built (session 5):

- **Jump host tunneling** (`muon-core/src/ssh/jump_host.rs`):
  - `JumpHost` struct: host, port, username, auth_type, key path
  - `JumpHostTunnel::connect_via_jumps()`: Chains SSH connections through intermediate hosts via `channel_open_direct_tcpip()`
  - Authenticates each jump host independently (password/pubkey/none)
  - Integrated into `SessionManager::connect_with_options()` — auto-detects `jump_hosts` field in SessionInfo

- **Port forwarding** (`muon-core/src/ssh/port_forward.rs`):
  - `PortForwardRule`: Local/Remote direction, bind/target host/port
  - `PortForwardManager`: Start/stop/list active forwards
  - Local forwarding: Binds TCP listener, spawns `channel_open_direct_tcpip` per connection, bidirectional relay via `tokio::select!`
  - Remote forwarding: **STUB** — spawns an idle task. russh requires server-side `tcpip_forward` which needs additional implementation
  - `Arc<Handle<ClientHandler>>` used to share SSH handle across spawned tasks
  - IPC: `port_forward_start`, `port_forward_stop`, `port_forward_list`
  - UI: `PortForwarding.svelte` with direction selector, bind/target fields, active forwards list

- **Connection options & keep-alive** (`muon-core/src/ssh/connection.rs`):
  - `ConnectionOptions` struct with keep_alive_interval_secs, keep_alive_max_count, enable_compression, connection_timeout_secs
  - Keep-alive wired into russh `Config { keepalive_interval, keepalive_max }` — **WORKING**
  - `enable_compression` field exists on `ConnectionOptions` but is **NOT wired** into russh `Config` — russh doesn't expose a compression toggle in Config, would need custom negotiation or different library support
  - `connect_with_options()` and `connect_via_proxy()` accept configurable options
  - `SshConnection.handle` changed to `Arc<Handle>` for safe sharing
  - `ClientHandler` now derives `Clone`

- **Connection pool** (`muon-core/src/session/pool.rs`):
  - `ConnectionPool` struct with per-session pool, configurable max size (default 3)
  - Methods: `get_or_create()`, `release()`, `cleanup()`, `close_all()`, `active_count()`, `total_count()`
  - `PooledConnectionGuard` return type for RAII-style usage
  - **NOT INTEGRATED**: The pool is not wired into `SessionManager`, `AppState`, or any IPC command. It is currently dead code. Needs: add to AppState, use for SFTP session reuse, expose via IPC

### Still TODO from session 5 plan:
- Wire `enable_compression` into actual SSH negotiation (may require russh feature flag or custom implementation)
- Integrate `ConnectionPool` into `SessionManager` for SFTP/background op reuse
- Implement remote port forwarding (requires russh `tcpip_forward` request)

---

## Phase 3: Session Management

**Status: COMPLETE (7/7 tasks done, but 3.5 is simplified)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | SessionInfo struct (complete) | DONE | |
| 3.2 | SessionFolder tree (recursive) | DONE | |
| 3.3 | SessionStore (JSON persistence, CRUD) | DONE | |
| 3.4 | Session import (SSH config) | DONE | session 5 |
| 3.5 | Credential store | DONE (simplified) | session 5 — file-based JSON, NOT OS keyring |
| 3.6 | Credential cache (in-memory) | DONE | |
| 3.7 | Tauri IPC commands (full CRUD) | DONE | |

### What was built (session 5):

- **SSH config importer** (`muon-core/src/session/import.rs`):
  - `SshConfigImporter::parse()`: Full parser — Host, HostName, Port, User, IdentityFile, ProxyCommand, ProxyJump, ForwardAgent, ForwardX11, RemoteCommand, arbitrary extra options
  - `parse_default()`: Auto-loads `~/.ssh/config`
  - `to_session_info()`: Converts parsed host to SessionInfo (skips wildcard patterns like `Host *`)
  - `import_to_folder()`: Batch-import into a new SessionFolder
  - `~` path expansion for IdentityFile
  - 3 unit tests: basic parsing, wildcard exclusion, folder import

- **Credential store** (`muon-core/src/credentials/store.rs`):
  - File-based JSON store in `~/.config/muon-ssh/credentials.json`
  - `save_credential()`, `get_credential()`, `delete_credential()`, `delete_all_for_session()`
  - Key format: `muon-ssh:<session_id>:<field>`
  - **NOT OS keyring**: Original plan called for `keyring` crate for OS-native credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service). This is a simplified fallback. See Technical Debt #13.

- **ImportDialog.svelte**: Two-step UI — scan SSH config to preview hosts, then import all into a folder
- **Sidebar.svelte**: Added import button in header, opens ImportDialog
- **IPC commands** (`import_cmds.rs`): `import_ssh_config`, `import_ssh_config_to_folder`, `credential_save`, `credential_get`, `credential_delete`
- **Frontend API** (`invoke.ts`): 9 new API functions for port forwarding, SSH config import, credential management
- **Vite/TS config**: Added `$components` path alias

### Still TODO from session 5 plan:
- Replace file-based credential store with OS keyring (`keyring` crate) — or make it optional/fallback
- Wire credential store into session connect flow (auto-fill password from store)

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
| 4.10 | Copy/paste | DONE | session 5 |

### What was built (session 5):
- **Terminal copy/paste** (`Terminal.svelte`):
  - `onSelectionChange`: Auto-copies selected text to system clipboard via `navigator.clipboard`
  - Right-click: Reads from clipboard and pastes to terminal via `sshWriteShell`

---

## Phase 5: SFTP & File Browser

**Status: 10/13 DONE**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | SFTP filesystem (russh-sftp) | DONE | |
| 5.2 | Local filesystem adapter | DONE | |
| 5.3 | FileSystem trait | DONE | |
| 5.4 | File transfer engine | DONE | |
| 5.5 | Background transfers | DONE | |
| 5.6 | File browser UI | DONE | |
| 5.7 | Address bar | DONE | |
| 5.8 | Context menus | PARTIAL | |
| 5.9 | Drag and drop | DONE | |
| 5.10 | Transfer queue UI | DONE | |
| 5.11 | Archive operations | TODO | |
| 5.12 | Remote file editing | TODO | |
| 5.13 | Sudo fallback | TODO | |

---

## Phase 6: Tools & Utilities

**Status: COMPLETE (10/10)**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 6.1 | Process viewer | DONE | |
| 6.2 | Log viewer | DONE | |
| 6.3 | Disk analyzer | DONE | |
| 6.4 | Search panel | DONE | |
| 6.5 | System info | DONE | |
| 6.6 | System load | DONE | |
| 6.7 | Port viewer | DONE | |
| 6.8 | SSH key manager | DONE | |
| 6.9 | Bundled scripts | DONE | |
| 6.10 | Port forwarding UI | DONE | session 5 |

---

## Phase 7: Settings & Preferences

**Status: 3/5 DONE**

| # | Task | Status | Notes |
|---|------|--------|-------|
| 7.1 | Settings struct expansion | DONE | |
| 7.2 | Settings dialog | DONE | |
| 7.3 | Theme system | DONE | |
| 7.4 | Keyboard shortcuts | TODO | Was planned for session 5, NOT done |
| 7.5 | External editors | TODO | |

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

**Status: NOT STARTED**

All 6 tasks TODO.

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
9. **SFTP channel:** Opens a new SSH session channel for SFTP — should reuse connection via pool (blocked by #16)
10. **Transfer upload buffering:** Upload reads entire file into memory before writing — needs streaming for large files
11. ~~**Remote exec uses shell channel:**~~ **FIXED**
12. **Remote port forwarding:** Stub only — `start_remote()` spawns an idle task. Needs russh `tcpip_forward` request implementation
13. **Credential store is file-based, not OS keyring:** `keyring` crate was planned for Phase 3.5 but not integrated. Current JSON file store is a simplified fallback — credentials stored in plaintext in config dir
14. **Jump host auth:** Jump hosts only support password with empty string or pubkey — should integrate with credential store for real passwords
15. **X11 forwarding:** Not implemented (Phase 2.9)
16. **Connection pool not integrated:** `ConnectionPool` module exists in `session/pool.rs` but is dead code — not added to `AppState`, not used by `SessionManager`, not exposed via IPC. Needs wiring into SFTP session reuse and background operations
17. **Compression not wired:** `ConnectionOptions.enable_compression` field exists but is never read. russh doesn't expose a simple compression toggle in `Config` — may need feature flag or custom negotiation

---

## Next Session

### Incomplete work from session 5 (finish first):

| # | Item | What's needed |
|---|------|---------------|
| A | **Connection pool integration** (2.11) | Add `ConnectionPool` to `AppState`, wire into `SessionManager` for SFTP reuse, add IPC commands |
| B | **Compression** (2.10) | Investigate russh compression support, wire `enable_compression` into Config if possible |
| C | **OS keyring** (3.5 upgrade) | Add `keyring` crate to Cargo.toml, implement `KeyringCredentialStore` alongside file fallback |
| D | **Keyboard shortcuts** (7.4) | Configurable shortcut system, global and per-component bindings |

### New work (after session 5 leftovers):

**Priority 1:** Phase 2.12 — Unit tests for SSH engine (auth, host keys, config import)
**Priority 2:** Phase 2.9 — X11 forwarding
**Priority 3:** Phase 5.8 — Context menus (delete, rename, permissions, archive)
**Priority 4:** Phase 5.11-5.13 — Archive operations, remote file editing, sudo fallback
**Priority 5:** Phase 7.5 — External editor detection
**Priority 6:** Phase 8 — Plugin system (WASM via wasmtime)
**Priority 7:** Phase 10 — OS integration & packaging

**Estimated complexity:** Medium — session 5 leftovers are mostly integration work, not new features.
