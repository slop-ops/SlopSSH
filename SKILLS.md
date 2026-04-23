# SKILLS.md — Muon SSH Rust/Tauri Rewrite Plan

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Core Language | Rust (edition 2024) | Backend, SSH engine, all logic |
| SSH Library | `russh` + `russh-sftp` | Pure-async SSH2, SFTP, channels |
| Async Runtime | `tokio` | Async I/O, tasks, channels |
| Desktop Shell | Tauri 2 | Native packaging, IPC, menus, tray |
| Frontend Framework | Svelte 5 (runes) | UI components |
| Frontend Bundler | Vite | Build, HMR, bundling |
| Terminal Emulator | xterm.js + xterm-addon-fit + xterm-addon-webgl | In-browser terminal |
| Serialization | `serde` + `serde_json` + `toml` | Config, session store, IPC messages |
| Password Storage | `keyring` crate (OS keychain) | OS-native credential storage |
| Plugin System | `wasmtime` | Sandboxed WASM plugins |
| Logging | `tracing` + `tracing-subscriber` | Structured logging |
| Error Handling | `anyhow` (app) + `thiserror` (libraries) | Error types |
| Crypto | `ring` or Russh built-in | Key parsing, host key verification |
| Process Spawning | `portable-pty` | Local terminal PTY |

---

## Proposed Project Structure

```
muon-ssh/
├── Cargo.toml                    # Workspace root
├── SKILLS.md                     # This file
├── AGENTS.md                     # Agent instructions
├── crates/
│   ├── muon-core/                # Core library (no UI deps)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ssh/              # SSH engine
│   │       │   ├── mod.rs
│   │       │   ├── connection.rs     # SSH connection lifecycle (russh)
│   │       │   ├── auth.rs           # Authentication methods
│   │       │   ├── channel.rs        # Shell/exec channel management
│   │       │   ├── sftp.rs           # SFTP filesystem (russh-sftp)
│   │       │   ├── port_forward.rs   # Local/remote port forwarding
│   │       │   ├── proxy.rs          # HTTP/SOCKS proxy support
│   │       │   ├── jump_host.rs      # Multi-hop SSH tunneling
│   │       │   ├── host_keys.rs      # known_hosts management
│   │       │   └── x11.rs            # X11 forwarding
│   │       ├── session/          # Session management
│   │       │   ├── mod.rs
│   │       │   ├── store.rs          # Session tree CRUD + persistence
│   │       │   ├── info.rs           # SessionInfo struct
│   │       │   ├── folder.rs         # SessionFolder tree
│   │       │   ├── pool.rs           # Connection pool for background ops
│   │       │   └── import.rs         # Import from PuTTY/WinSCP/SSH config
│   │       ├── config/           # Configuration management
│   │       │   ├── mod.rs
│   │       │   ├── settings.rs       # App settings (TOML)
│   │       │   ├── app_context.rs    # Global app state
│   │       │   └── paths.rs          # Config directory paths
│   │       ├── credentials/      # Credential management
│   │       │   ├── mod.rs
│   │       │   ├── store.rs          # OS keyring integration
│   │       │   └── cache.rs          # In-memory credential cache
│   │       ├── snippets/         # Command snippets
│   │       │   ├── mod.rs
│   │       │   └── manager.rs        # Snippet CRUD + JSON persistence
│   │       ├── file_transfer/    # File transfer engine
│   │       │   ├── mod.rs
│   │       │   ├── engine.rs         # Transfer orchestration
│   │       │   ├── conflict.rs       # Conflict resolution
│   │       │   └── progress.rs       # Progress tracking
│   │       ├── scripts/          # Bundled remote scripts
│   │       │   ├── mod.rs
│   │       │   └── loader.rs         # include_str! bundled scripts
│   │       ├── updater/          # Update checker
│   │       │   ├── mod.rs
│   │       │   └── github.rs         # GitHub releases API checker
│   │       └── plugin/           # Plugin system (WASM)
│   │           ├── mod.rs
│   │           ├── host.rs           # WASM host functions
│   │           ├── loader.rs         # Plugin discovery/loading
│   │           └── api.rs            # Plugin trait/API definitions
│   │
│   ├── muon-tauri/               # Tauri app (thin binary)
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   ├── capabilities/
│   │   │   └── default.json          # Tauri 2 permissions
│   │   ├── src/
│   │   │   ├── main.rs               # Tauri entry point
│   │   │   ├── commands/             # Tauri IPC command handlers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── session_cmds.rs   # Session CRUD commands
│   │   │   │   ├── ssh_cmds.rs       # SSH connect/disconnect
│   │   │   │   ├── sftp_cmds.rs      # File browser operations
│   │   │   │   ├── terminal_cmds.rs  # Terminal session management
│   │   │   │   ├── transfer_cmds.rs  # File transfer commands
│   │   │   │   ├── settings_cmds.rs  # Settings read/write
│   │   │   │   ├── snippet_cmds.rs   # Snippet CRUD
│   │   │   │   ├── plugin_cmds.rs    # Plugin management
│   │   │   │   └── system_cmds.rs    # OS integration commands
│   │   │   ├── state.rs              # Tauri managed state
│   │   │   ├── events.rs             # Tauri event definitions
│   │   │   └── menu.rs               # Native menu setup
│   │   └── icons/                    # App icons
│   │
│   └── muon-plugins/             # Example plugins (optional)
│       ├── k8s-context/
│       │   ├── Cargo.toml
│       │   └── src/lib.rs
│       └── hello/
│           ├── Cargo.toml
│           └── src/lib.rs
│
├── frontend/                     # Svelte 5 + xterm.js
│   ├── package.json
│   ├── vite.config.ts
│   ├── svelte.config.js
│   ├── tsconfig.json
│   ├── index.html
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css                    # Global styles, CSS variables, themes
│   │   ├── main.ts                    # Entry point
│   │   ├── lib/
│   │   │   ├── stores/               # Svelte stores
│   │   │   │   ├── sessions.ts       # Session list state
│   │   │   │   ├── settings.ts       # App settings state
│   │   │   │   ├── transfers.ts      # Active transfers state
│   │   │   │   └── plugins.ts        # Plugin state
│   │   │   ├── api/                  # Tauri IPC wrappers
│   │   │   │   ├── invoke.ts         # Type-safe invoke wrapper
│   │   │   │   ├── session.ts        # Session API
│   │   │   │   ├── ssh.ts            # SSH API
│   │   │   │   ├── sftp.ts           # SFTP/file API
│   │   │   │   ├── terminal.ts       # Terminal API
│   │   │   │   ├── settings.ts       # Settings API
│   │   │   │   └── snippets.ts       # Snippets API
│   │   │   ├── terminal/             # xterm.js integration
│   │   │   │   ├── Terminal.svelte   # xterm.js wrapper component
│   │   │   │   ├── fit.ts            # Terminal resize/fit logic
│   │   │   │   ├── themes.ts         # Terminal color themes
│   │   │   │   └── pty.ts            # PTY data bridge via Tauri events
│   │   │   └── utils/
│   │   │       ├── format.ts         # File size, date formatting
│   │   │       ├── icons.ts          # File type icon mapping
│   │   │       └── i18n.ts           # Internationalization helper
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   ├── AppShell.svelte       # Main layout
│   │   │   │   ├── Sidebar.svelte        # Session list sidebar
│   │   │   │   └── StatusBar.svelte      # Bottom status bar
│   │   │   ├── session/
│   │   │   │   ├── SessionList.svelte    # Saved session tree
│   │   │   │   ├── SessionItem.svelte    # Session card/row
│   │   │   │   ├── SessionFolder.svelte  # Folder in tree
│   │   │   │   ├── NewSessionDialog.svelte
│   │   │   │   ├── SessionConnectDialog.svelte
│   │   │   │   └── ImportDialog.svelte
│   │   │   ├── terminal/
│   │   │   │   ├── TerminalTab.svelte     # Single terminal tab
│   │   │   │   ├── TerminalHolder.svelte  # Terminal tab container
│   │   │   │   ├── LocalTerminal.svelte   # Local PTY terminal
│   │   │   │   └── SnippetPanel.svelte    # Snippet selector
│   │   │   ├── files/
│   │   │   │   ├── FileBrowser.svelte     # Dual-pane file manager
│   │   │   │   ├── FileList.svelte        # File listing (table/list)
│   │   │   │   ├── AddressBar.svelte      # Breadcrumb path bar
│   │   │   │   ├── TransferQueue.svelte   # Background transfers
│   │   │   │   ├── TransferProgress.svelte
│   │   │   │   └── PropertiesDialog.svelte
│   │   │   ├── tools/
│   │   │   │   ├── ProcessViewer.svelte
│   │   │   │   ├── LogViewer.svelte
│   │   │   │   ├── DiskAnalyzer.svelte
│   │   │   │   ├── SearchPanel.svelte
│   │   │   │   ├── SysInfoPanel.svelte
│   │   │   │   ├── SystemLoad.svelte
│   │   │   │   ├── PortViewer.svelte
│   │   │   │   └── KeyManager.svelte
│   │   │   ├── settings/
│   │   │   │   ├── SettingsDialog.svelte
│   │   │   │   ├── GeneralPage.svelte
│   │   │   │   ├── TerminalPage.svelte
│   │   │   │   ├── FileBrowserPage.svelte
│   │   │   │   ├── SecurityPage.svelte
│   │   │   │   └── PluginsPage.svelte
│   │   │   └── common/
│   │   │       ├── Button.svelte
│   │   │       ├── Dialog.svelte
│   │   │       ├── SplitPane.svelte
│   │   │       ├── TabPanel.svelte
│   │   │       ├── ContextMenu.svelte
│   │   │       ├── TreeView.svelte
│   │   │       ├── SearchInput.svelte
│   │   │       └── LineGraph.svelte
│   │   └── i18n/
│   │       ├── en.json
│   │       ├── es.json
│   │       ├── ru.json
│   │       ├── fr.json
│   │       ├── de.json
│   │       ├── pt.json
│   │       └── cn.json
│   ├── static/
│   │   └── fonts/                  # Terminal fonts
│   └── tests/
│       └── ...
│
├── scripts/                      # Bundled remote shell scripts
│   ├── ps.sh
│   ├── search.sh
│   └── linux-sysinfo.sh
│
└── .github/
    └── workflows/
        └── ci.yml                 # GitHub Actions: build + test
```

---

## Phase Breakdown

### Phase 1: Project Scaffolding & Core Infrastructure
**Goal:** Bootable Tauri 2 + Svelte 5 app with Rust workspace.

| # | Task | Details |
|---|------|---------|
| 1.1 | Initialize Cargo workspace | Create workspace with `muon-core` and `muon-tauri` crates |
| 1.2 | Initialize Tauri 2 project | `npm create tauri-app@latest` with Svelte template, configure `tauri.conf.json` |
| 1.3 | Set up Svelte 5 + Vite | Configure `svelte.config.js`, `vite.config.ts`, TypeScript |
| 1.4 | Add core dependencies | `tokio`, `serde`, `serde_json`, `toml`, `anyhow`, `thiserror`, `tracing`, `dirs` |
| 1.5 | Config directory management | `~/.config/muon-ssh/` (XDG on Linux, AppData on Windows). `paths.rs` with platform logic |
| 1.6 | Settings module | `Settings` struct with serde, TOML serialization, `SettingsManager` with load/save/default |
| 1.7 | Logging setup | `tracing-subscriber` with file + stdout output, configurable log level |
| 1.8 | App context / state | `AppState` struct managing settings, session store, credential store |
| 1.9 | Tauri state management | Register `AppState` as Tauri managed state, wire up `state.rs` |
| 1.10 | Basic IPC ping | Rust command `greet` -> Svelte display, verify end-to-end IPC works |

### Phase 2: SSH Engine
**Goal:** Working SSH connections with all auth methods.

| # | Task | Details |
|---|------|---------|
| 2.1 | Add russh dependency | `russh` + `russh-keys` + `russh-sftp` crates |
| 2.2 | SSH connection struct | `SshConnection` wrapping russh `Handle`, with connect/disconnect/reconnect |
| 2.3 | Authentication engine | Support: none, password, publickey (OpenSSH format), keyboard-interactive. Iterate server-preferred methods |
| 2.4 | Host key verification | Parse `known_hosts` file, verify host keys, expose callback for user accept/reject |
| 2.5 | Shell channel | Open shell channel with PTY allocation (xterm-256color), set env vars, get I/O streams |
| 2.6 | Proxy support | HTTP CONNECT, SOCKS5 proxy via `tokio` + custom connector |
| 2.7 | Jump host tunneling | Recursive multi-hop via TCP direct forwarding. Support `HopEntry` chain |
| 2.8 | Port forwarding | Local (`-L`) and remote (`-R`) forwarding using russh channels |
| 2.9 | X11 forwarding | Request X11 channel, forward to Unix socket (Linux/Mac) or TCP (Windows) |
| 2.10 | Keep-alive & compression | Configurable keep-alive interval, zlib compression negotiation |
| 2.11 | Connection pool | `ConnectionPool` for background SFTP sessions, reuse connections |
| 2.12 | Unit tests | Test auth flows, host key verification, proxy connection (mock) |

### Phase 3: Session Management
**Goal:** Full session CRUD with hierarchical folder tree.

| # | Task | Details |
|---|------|---------|
| 3.1 | SessionInfo struct | Mirror all fields from Java: host, port, user, auth type, proxy, jump hosts, port forwarding rules, X11 flag, favorites |
| 3.2 | SessionFolder tree | Recursive folder structure with `folders: Vec<SessionFolder>` and `items: Vec<SessionInfo>` |
| 3.3 | SessionStore | JSON persistence in `~/.config/muon-ssh/sessions.json`, CRUD operations |
| 3.4 | Session import | SSH config parser (`~/.ssh/config`), legacy muon-ssh JSON import |
| 3.5 | Credential store | `keyring` crate for OS-native password storage (keychain on macOS, Credential Manager on Windows, Secret Service on Linux) |
| 3.6 | Credential cache | In-memory cache for session duration (password, passphrase, user) |
| 3.7 | Tauri IPC commands | `session_list`, `session_create`, `session_update`, `session_delete`, `session_import` |

### Phase 4: Terminal Integration
**Goal:** Working SSH terminal via xterm.js.

| # | Task | Details |
|---|------|---------|
| 4.1 | xterm.js setup | Install `xterm`, `@xterm/addon-fit`, `@xterm/addon-webgl`, `@xterm/addon-search` |
| 4.2 | Terminal.svelte component | Wrapper managing xterm lifecycle, resize, focus, destroy |
| 4.3 | Terminal themes | Port dark/light terminal themes, configurable 16-color palette |
| 4.4 | PTY data bridge | Tauri event system: Rust emits `terminal-output` events with base64-encoded data, frontend sends `terminal-input` commands |
| 4.5 | Terminal session manager | Rust-side: manage multiple terminal sessions per SSH connection, PTY resize requests |
| 4.6 | Terminal tabs UI | `TerminalHolder.svelte` with closable tabs, add-new button |
| 4.7 | Snippet panel | `SnippetPanel.svelte` with search, CRUD, send-to-terminal |
| 4.8 | Reconnection UI | Show disconnected overlay with reconnect button |
| 4.9 | Local terminal | `portable-pty` crate for local PTY (bash/zsh/cmd), separate data bridge |
| 4.10 | Copy/paste | xterm.js clipboard integration, configurable PuTTY-like select-to-copy |

### Phase 5: SFTP & File Browser
**Goal:** Dual-pane file browser with drag-and-drop transfers.

| # | Task | Details |
|---|------|---------|
| 5.1 | SFTP filesystem | `russh-sftp` for remote file ops: list, stat, delete, mkdir, rename, chmod, read, write |
| 5.2 | Local filesystem | `tokio::fs` wrapped behind same `FileSystem` trait |
| 5.3 | FileSystem trait | Common interface: `list_dir`, `delete`, `mkdir`, `rename`, `chmod`, `read_file`, `write_file`, `stat` |
| 5.4 | File transfer engine | Streaming transfer with progress callback, conflict resolution (auto-rename/overwrite/skip/prompt) |
| 5.5 | Background transfers | Thread pool for concurrent transfers, queue with progress tracking |
| 5.6 | File browser UI | `FileBrowser.svelte` dual-pane, `FileList.svelte` (table + list view modes) |
| 5.7 | Address bar | Breadcrumb navigation with path editing |
| 5.8 | Context menus | Right-click operations: delete, rename, permissions, edit, archive |
| 5.9 | Drag and drop | HTML5 DnD for cross-pane file transfer initiation |
| 5.10 | Transfer queue UI | `TransferQueue.svelte` showing active/completed transfers |
| 5.11 | Archive operations | Remote tar/gz/zip creation and extraction via shell commands |
| 5.12 | Remote file editing | Download -> open in external editor -> watch -> re-upload cycle |
| 5.13 | Sudo fallback | Transfer to /tmp then `sudo cp/mv` for permission-restricted targets |

### Phase 6: Tools & Utilities
**Goal:** Port all tool panels from the original app.

| # | Task | Details |
|---|------|---------|
| 6.1 | Process viewer | Execute `ps.sh` remotely, display in filterable/sortable table, kill process action |
| 6.2 | Log viewer | Paged remote log file reading with search, word wrap, line numbers |
| 6.3 | Disk analyzer | Execute `du`, parse output, display treemap/bar chart |
| 6.4 | Search panel | Remote `find` + `grep` execution, paginated results |
| 6.5 | System info | Execute `linux-sysinfo.sh`, display formatted output |
| 6.6 | System load | Periodic CPU/mem/swap stats with real-time line graphs |
| 6.7 | Port viewer | Parse `ss -tlnp` or `netstat`, display in table |
| 6.8 | SSH key manager | Local and remote SSH key listing, generation, deployment |
| 6.9 | Bundled scripts | `include_str!` for ps.sh, search.sh, linux-sysinfo.sh |

### Phase 7: Settings & Preferences
**Goal:** Full settings UI mirroring the original app.

| # | Task | Details |
|---|------|---------|
| 7.1 | Settings struct expansion | All ~60 fields from Java: terminal, UI, transfer, connection, security, editor, language |
| 7.2 | Settings dialog | Multi-page Svelte dialog: General, Terminal, File Browser, Log Viewer, Display, Security, Shortcuts, Editor |
| 7.3 | Theme system | CSS variables for dark/light themes, terminal color palette configuration |
| 7.4 | Keyboard shortcuts | Configurable shortcuts stored in settings, global and per-component |
| 7.5 | External editors | Auto-detect installed editors (VS Code, Notepad++, etc.), manual path config |

### Phase 8: Plugin System
**Goal:** WASM-based plugin runtime.

| # | Task | Details |
|---|------|---------|
| 8.1 | Plugin API definition | Rust trait: `MuonPlugin` with `name()`, `version()`, `init()`, `on_session_connect()`, `on_session_disconnect()`, `render_panel()` (returns HTML/JSON) |
| 8.2 | WASM host functions | Exposed to plugins: `execute_command()`, `read_setting()`, `show_notification()` |
| 8.3 | Plugin loader | Scan `~/.config/muon-ssh/plugins/`, load `.wasm` files via wasmtime |
| 8.4 | Plugin sandboxing | WASM memory limits, timeout enforcement, capability whitelist |
| 8.5 | K8s context plugin | Port to WASM: runs `kubectl config current-context`, displays in status bar |
| 8.6 | Plugin settings UI | Enable/disable plugins, plugin configuration panel |
| 8.7 | Plugin IPC | Tauri events for plugin -> frontend communication |

### Phase 9: Internationalization
**Goal:** Full i18n support matching the original 7 languages.

| # | Task | Details |
|---|------|---------|
| 9.1 | i18n framework | JSON-based translation files in `src/i18n/`, Svelte store for current locale |
| 9.2 | Extract all strings | Port all 330 keys from Java ResourceBundle |
| 9.3 | Translate | English (base), Chinese, Spanish, Portuguese, Russian, German, French |
| 9.4 | Language selector | Settings dropdown, persists to config |
| 9.5 | RTL support | Ensure layout works for potential RTL languages |

### Phase 10: OS Integration & Packaging
**Goal:** Native desktop experience on Windows and Linux.

| # | Task | Details |
|---|------|---------|
| 10.1 | Native menus | Tauri menu: File, Edit, Session, View, Tools, Help |
| 10.2 | System tray | Optional tray icon with session quick-access |
| 10.3 | File type associations | Register `.muon` session files |
| 10.4 | Window management | Remember position/size, minimize to tray |
| 10.5 | Auto-updater | Tauri 2 built-in updater with GitHub releases backend |
| 10.6 | Windows packaging | Tauri MSI + NSIS installers |
| 10.7 | Linux packaging | `.deb` (dpkg), `.AppImage`, Flatpak manifest |
| 10.8 | GitHub Actions CI | Build matrix: Windows (MSVC), Linux (glibc) |

### Phase 11: Polish & Testing
**Goal:** Production-ready quality.

| # | Task | Details |
|---|------|---------|
| 11.1 | Integration tests | Test SSH connect -> shell -> SFTP -> disconnect cycle |
| 11.2 | Frontend E2E tests | Playwright tests for critical UI flows |
| 11.3 | Error handling audit | Ensure all error paths have user-facing messages |
| 11.4 | Performance profiling | Terminal throughput, SFTP transfer speed benchmarks |
| 11.5 | Accessibility | Keyboard navigation, screen reader support for key flows |
| 11.6 | Documentation | README, CONTRIBUTING, architecture docs |

---

## Java to Rust Migration Mapping

| Java Class | Rust Module | Notes |
|-----------|-------------|-------|
| `App.java` | `muon-tauri/src/main.rs` | Tauri replaces Swing bootstrap |
| `AppContext.java` | `muon-core/src/config/app_context.rs` | Arc<Mutex<AppState>> |
| `Settings.java` + `SettingsManager.java` | `muon-core/src/config/settings.rs` | Serde + TOML |
| `SSHHandler.java` | `muon-core/src/ssh/connection.rs` | russh async |
| `RemoteSessionInstance.java` | `muon-core/src/ssh/channel.rs` | Wraps SshConnection |
| `SshFileSystem.java` | `muon-core/src/ssh/sftp.rs` | russh-sftp |
| `SshTtyConnector.java` | `muon-core/src/ssh/channel.rs` | PTY over russh |
| `PortForwardingSession.java` | `muon-core/src/ssh/port_forward.rs` | russh direct/forwarded channels |
| `CustomSocketFactory.java` | `muon-core/src/ssh/proxy.rs` | tokio + custom connector |
| `GraphicalHostKeyVerifier.java` | `muon-core/src/ssh/host_keys.rs` | known_hosts parsing + callback |
| `SessionInfo.java` | `muon-core/src/session/info.rs` | Serde struct |
| `SessionStore.java` | `muon-core/src/session/store.rs` | JSON file CRUD |
| `SessionFolder.java` + `SavedSessionTree.java` | `muon-core/src/session/folder.rs` | Recursive tree |
| `PasswordStore.java` | `muon-core/src/credentials/store.rs` | OS keyring |
| `CachedCredentialProvider.java` | `muon-core/src/credentials/cache.rs` | HashMap in memory |
| `SnippetManager.java` | `muon-core/src/snippets/manager.rs` | JSON file CRUD |
| `FileTransfer.java` | `muon-core/src/file_transfer/engine.rs` | Async streaming |
| `SshFileOperations.java` | `muon-core/src/ssh/sftp.rs` + shell commands | Combined approach |
| `ScriptLoader.java` | `muon-core/src/scripts/loader.rs` | `include_str!` |
| `UpdateChecker.java` | `muon-core/src/updater/github.rs` | Tauri updater replaces |
| `AppWindow.java` | `frontend/src/components/layout/AppShell.svelte` | Svelte layout |
| `AppSkinDark/Light` | `frontend/src/app.css` | CSS custom properties |
| `TerminalComponent.java` | `frontend/src/lib/terminal/Terminal.svelte` | xterm.js |
| `TerminalHolder.java` | `frontend/src/components/terminal/TerminalHolder.svelte` | Tab container |
| `CustomJediterm.java` | `frontend/src/lib/terminal/Terminal.svelte` | xterm.js replaces entirely |
| `CustomizedSettingsProvider.java` | `frontend/src/lib/terminal/themes.ts` | xterm theme config |
| `FileBrowser.java` | `frontend/src/components/files/FileBrowser.svelte` | Svelte dual-pane |
| `SettingsDialog.java` | `frontend/src/components/settings/SettingsDialog.svelte` | Svelte dialog |
| `SessionListPanel.java` | `frontend/src/components/layout/Sidebar.svelte` | Svelte sidebar |
| `ProcessViewer.java` | `frontend/src/components/tools/ProcessViewer.svelte` | Svelte + remote exec |
| `LogViewer.java` | `frontend/src/components/tools/LogViewer.svelte` | Svelte paged viewer |
| `DiskspaceAnalyzer.java` | `frontend/src/components/tools/DiskAnalyzer.svelte` | Svelte chart |
| `SearchPanel.java` | `frontend/src/components/tools/SearchPanel.svelte` | Svelte search |
| `ClosableTabbedPanel.java` | `frontend/src/components/common/TabPanel.svelte` | Svelte tabs |
| `SessionExportImport.java` | `muon-core/src/session/import.rs` | ZIP export/import |

---

## Key Architecture Decisions

### 1. IPC via Tauri Commands + Events
Rust -> Frontend via `app.emit()` events (terminal output, transfer progress). Frontend -> Rust via `invoke()` commands (connect, list files, save settings). Binary data (terminal I/O) base64-encoded over events.

### 2. Async Everything
All SSH I/O is async via tokio. Tauri commands use `#[tauri::command(async)]`. Long-running operations (file transfer, port forwarding) run in spawned tokio tasks and report progress via events.

### 3. Connection Pooling
Each SSH session holds a primary connection for the terminal and a pool of secondary connections for SFTP and background operations. Connections are reused and recycled.

### 4. Fresh Config Format
New `~/.config/muon-ssh/` directory. Settings in `settings.toml`. Sessions in `sessions.json`. Passwords in OS keyring. Provide import tool for legacy `~/.muon-ssh/` data.

### 5. xterm.js Terminal
All terminal rendering is handled by xterm.js in the browser view. Rust side only manages the SSH channel and pipes raw PTY bytes to/from the frontend via Tauri events. This eliminates the need for JediTerm entirely.

### 6. WASM Plugin Isolation
Plugins run in wasmtime sandboxes with restricted capabilities. Host functions are explicitly exposed. Plugins cannot access the filesystem or network directly.
