# Architecture — Muon SSH (Rust/Tauri)

## Overview

Muon SSH is a cross-platform SSH/SCP/SFTP client built with a three-layer architecture:

1. **`muon-core`** — Pure Rust library containing all business logic
2. **`muon-tauri`** — Thin Tauri binary wiring muon-core to IPC
3. **`frontend`** — Svelte 5 + xterm.js SPA communicating via Tauri IPC

```
┌─────────────────────────────────────────┐
│              Svelte 5 UI                │
│  (xterm.js, FileBrowser, Sidebar, ...)  │
├─────────────────────────────────────────┤
│           Tauri IPC Layer               │
│     invoke() commands + events          │
├─────────────────────────────────────────┤
│            muon-tauri                    │
│   Command handlers (thin wrappers)      │
├─────────────────────────────────────────┤
│            muon-core                    │
│  SSH, sessions, transfers, config, ...  │
└─────────────────────────────────────────┘
```

## Crate Structure

### `crates/muon-core/`

The core library has zero UI or Tauri dependencies. All modules are pure Rust.

| Module | Purpose |
|--------|---------|
| `config/` | Settings (TOML), config paths, editor detection |
| `credentials/` | Credential store (OS keyring + encrypted file fallback) |
| `file_transfer/` | Async upload/download engine with progress tracking |
| `filesystem/` | SFTP and local filesystem abstractions |
| `local_terminal/` | Local PTY via portable-pty |
| `logging/` | tracing-subscriber setup with file rotation |
| `plugin/` | WASM plugin host and sandbox |
| `scripts/` | Bundled remote scripts |
| `session/` | Session store, folder tree, connection pool |
| `snippets/` | Command snippets (JSON persistence) |
| `ssh/` | SSH engine (russh), auth, channels, port forwarding, jump hosts |
| `tab_state/` | Tab state persistence across restarts |
| `tools/` | Remote command execution for tool panels |
| `updater/` | GitHub release checker and downloader |
| `utils/` | Shared utilities (shell_escape, path validation) |

### `crates/muon-tauri/`

The binary crate contains only IPC glue:

- `state.rs` — `AppState` with per-concern mutexes
- `commands/` — 14 command modules, each wrapping muon-core functions
- `menu.rs` — Native menu bar and system tray
- `main.rs` — App initialization, window management, event wiring

### `frontend/`

Svelte 5 SPA using runes (`$state`, `$derived`, `$effect`):

- `components/` — UI components organized by feature
- `lib/api/invoke.ts` — Typed IPC wrappers for all Tauri commands
- `lib/types.ts` — TypeScript interfaces matching Rust structs
- `lib/utils/i18n.ts` — JSON-based i18n with 7 languages
- `lib/stores/theme.ts` — CSS variable theme system
- `i18n/` — Language JSON files (en, cn, de, es, fr, pt, ru)

## Data Flow

### SSH Connection Lifecycle

```
1. Frontend: createSession() → session_cmds → SessionStore.save()
2. Frontend: sshConnect(sessionId) → ssh_cmds → SessionManager.connect()
3. SessionManager resolves auth (password/key/keyboard-interactive)
4. Host key verified: known/unknown/changed → frontend prompt
5. Connection added to ConnectionPool
6. Frontend: sshOpenShell() → spawns read loop → emits terminal-output-{id}
7. xterm.js renders base64-decoded SSH output
8. User input → sshWriteShell() → base64 decode → SSH channel write
```

### File Transfer

```
1. Frontend: transferUpload/Download() → transfer_cmds
2. Gets SFTP session from AppState.sftp_sessions
3. Calls TransferEngine.spawn_upload/download()
4. Engine spawns tokio task, reads/writes in 32KB chunks
5. Progress callback emits transfer-progress-{id} event
6. TransferQueue component updates progress bar
```

### Terminal Data Bridge

```
SSH bytes (raw) → base64 encode → Tauri event "terminal-output-{id}"
                                         ↓
                                   xterm.js (base64 decode + render)

xterm onData → ssh_write_shell() → base64 decode → SSH channel write
```

## State Management

### Rust-side: Per-concern Mutexes

`AppState` contains individual `tokio::sync::Mutex` per field to avoid contention:

```rust
pub struct AppState {
    pub settings: Mutex<Settings>,
    pub session_store: Mutex<SessionStore>,
    pub ssh_manager: Mutex<SessionManager>,
    pub sftp_sessions: Mutex<HashMap<String, Arc<Mutex<Option<SftpSession>>>>>,
    pub transfer_engine: Arc<TransferEngine>,
    pub port_forward_manager: Mutex<PortForwardManager>,
    pub connection_pool: Mutex<ConnectionPool>,
    pub local_terminal: std::sync::Mutex<LocalTerminalManager>,
    pub plugin_manager: Mutex<PluginManager>,
    pub credential_store: Mutex<CredentialStore>,
}
```

Commands lock only the fields they need.

### Frontend-side: Svelte 5 Runes

- `AppShell.svelte` owns tab state (`tabs[]`, `activeTabId`, `activeSessionId`)
- `TerminalHolder.svelte` receives tabs via `$bindable()` props
- Settings managed via `SettingsDialog.svelte` → IPC → Rust
- Tab state persisted to disk via `tab_state.rs`

## Security

- **Host key verification:** Unknown keys require user confirmation
- **Credential encryption:** AES-256-GCM with machine-derived key
- **Shell escaping:** All user input sanitized via `shell_escape()` before remote execution
- **CSP:** Content Security Policy set in `tauri.conf.json`
- **Input validation:** `SessionInfo::validate()`, `Settings::validate()`, `PortForwardRule::validate()`

## Persistence

| Data | File | Format |
|------|------|--------|
| Sessions | `~/.config/muon-ssh/sessions.json` | JSON (with 5 rotating backups) |
| Settings | `~/.config/muon-ssh/settings.toml` | TOML |
| Snippets | `~/.config/muon-ssh/snippets.json` | JSON |
| Tab state | `~/.config/muon-ssh/tab_state.json` | JSON |
| Window bounds | `~/.config/muon-ssh/window_bounds.json` | JSON |
| Logs | `~/.local/share/muon-ssh/logs/` | Daily rotating files |
| Credentials | OS keyring (encrypted file fallback) | AES-256-GCM |

In **portable mode** (when `portable.marker` exists next to binary), all data is stored relative to the binary directory.

## Testing

- **300 Rust unit tests** across all modules
- **67 frontend unit tests** (i18n, shortcuts, themes, types)
- **3 Playwright E2E test specs**
- All tests run via `cargo test --workspace` and `npm run test`

## IPC Commands (72 total)

| Category | Count | Commands |
|----------|-------|----------|
| Settings | 2 | get_settings, save_settings |
| Sessions | 5 | list, create, update, delete, create_folder |
| SSH | 7 | connect, disconnect, open/write/resize/close_shell, accept_host_key |
| SFTP | 12 | connect, disconnect, list_dir, mkdir, remove, rename, read/write_file, stat, home, upload/download_sudo |
| Snippets | 4 | list, create, update, delete |
| Transfers | 5 | upload, download, cancel, list, clear_completed |
| Keys | 5 | list_local, list_remote, generate, deploy, read |
| Port Forward | 3 | start, stop, list |
| Import | 2 | import_ssh_config, import_ssh_config_to_folder |
| Credentials | 3 | save, get, delete |
| Archives | 2 | create, extract |
| Local Terminal | 4 | open, write, resize, close |
| System | 7 | get_version, get_app_version, detect_editors, open_in_editor, check_for_updates, download_update, update_tray_tooltip |
| Plugins | 9 | list, discover, set_enabled, remove, get/set/get_all_settings, fire_event, show_notification |
| Tab State | 3 | save_tab_state, load_tab_state, clear_tab_state |
