# Muon SSH — Rust/Tauri Rewrite

A modern, cross-platform SSH/SCP/SFTP client built with **Rust**, **Tauri 2**, and **Svelte 5**.

## Features

- **SSH Terminal** — Full xterm.js terminal with 256-color support, themes, and copy/paste
- **Dual-pane File Browser** — SFTP file browser with drag-and-drop transfers
- **Port Forwarding** — Local (-L) and remote (-R) port forwarding
- **X11 Forwarding** — X11 display forwarding support
- **Multi-hop SSH** — Jump host tunneling with credential management
- **Proxy Support** — HTTP CONNECT and SOCKS5 proxy
- **Remote Tools** — Process viewer, log viewer, disk analyzer, search, system info
- **SSH Key Manager** — Generate, deploy, and manage SSH keys
- **Connection Pool** — Reuse SSH connections for SFTP and background operations
- **Credential Store** — OS-native keyring integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Plugin System** — WASM-based sandboxed plugins with capability whitelisting
- **7 Languages** — English, Spanish, Russian, French, German, Portuguese, Chinese
- **Dark/Light Themes** — Configurable themes with terminal color palettes
- **Auto-updater** — GitHub releases-based update checking

## Architecture

```
muon-ssh/
├── crates/
│   ├── muon-core/        # Pure Rust library (SSH engine, sessions, config)
│   ├── muon-tauri/       # Tauri app binary (IPC commands, menus, tray)
│   └── muon-plugins/     # Example WASM plugins
├── frontend/             # Svelte 5 + xterm.js UI
├── scripts/              # Bundled remote shell scripts
└── .github/workflows/    # CI/CD
```

**Key principle:** All business logic lives in `muon-core`. The Tauri layer is a thin IPC bridge. The Svelte frontend handles all UI.

## Prerequisites

- **Rust** 1.85+ (edition 2024)
- **Node.js** 20+
- **Tauri 2 CLI**: `cargo install tauri-cli --version "^2"`
- **System deps** (Linux): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libappindicator3-dev`

## Build & Run

```bash
# Development (hot reload)
cd crates/muon-tauri && cargo tauri dev

# Production build
cd crates/muon-tauri && cargo tauri build

# Frontend only
cd frontend && npm run dev
```

## Development

```bash
# Run Rust tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Lint
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check

# Frontend
cd frontend && npm run build
cd frontend && npm run check
cd frontend && npm run test
cd frontend && npm run test:e2e
```

## Project Structure

| Layer | Tech | Role |
|-------|------|------|
| Core | Rust | SSH engine, session management, file transfers, config |
| Desktop | Tauri 2 | Native packaging, IPC bridge, menus, tray |
| Frontend | Svelte 5 + xterm.js | All UI, communicates via Tauri `invoke()` and events |

### IPC Data Flow

- **Rust → Frontend:** `app.emit()` events (terminal output, transfer progress, plugin events)
- **Frontend → Rust:** `invoke()` commands (connect, list files, save settings)
- **Binary data:** Base64-encoded over events

## Configuration

Config lives in `~/.config/muon-ssh/` (XDG on Linux):

| File | Purpose |
|------|---------|
| `settings.toml` | App settings |
| `sessions.json` | Saved sessions tree |
| `snippets.json` | Command snippets |
| `window_bounds.json` | Window position/size |
| `plugins/` | WASM plugin files |
| `plugin_settings.json` | Per-plugin settings |

## Session File Format

`.muon` files are JSON containing a `SessionInfo` struct:

```json
{
  "id": "uuid",
  "name": "My Server",
  "host": "example.com",
  "port": 22,
  "username": "user",
  "auth_type": "PublicKey",
  "private_key_path": "~/.ssh/id_ed25519"
}
```

## Plugins

See [crates/muon-plugins/README.md](crates/muon-plugins/README.md) for the plugin API.

## License

Apache-2.0
