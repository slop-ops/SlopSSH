# SlopSSH

A modern SSH/SCP/SFTP client built with **Rust**, **Tauri 2**, and **Svelte 5**.

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
- **Credential Store** — OS-native keyring integration (Windows Credential Manager, Linux Secret Service)
- **Plugin System** — WASM-based sandboxed plugins with capability whitelisting
- **7 Languages** — English, Spanish, Russian, French, German, Portuguese, Chinese
- **Dark/Light Themes** — Configurable themes with terminal color palettes
- **Auto-updater** — GitHub releases-based update checking

## Architecture

```
SlopSSH/
├── crates/
│   ├── slopssh-core/        # Pure Rust library (SSH engine, sessions, config)
│   ├── slopssh-tauri/       # Tauri app binary (IPC commands, menus, tray)
│   └── slopssh-plugins/     # Example WASM plugins
├── frontend/                # Svelte 5 + xterm.js UI
├── scripts/                 # Bundled remote shell scripts
└── .github/workflows/       # CI/CD
```

**Key principle:** All business logic lives in `slopssh-core`. The Tauri layer is a thin IPC bridge. The Svelte frontend handles all UI.

## Prerequisites

- **Rust** 1.85+ (edition 2024)
- **Node.js** 20+
- **Tauri 2 CLI**: `cargo install tauri-cli --version "^2"`

### Linux system dependencies

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev
```

### Windows

No additional system dependencies required beyond Rust and Node.js.

## Building from Source

### Linux

```bash
# 1. Clone the repository
git clone https://github.com/slop-ops/SlopSSH.git
cd SlopSSH

# 2. Install frontend dependencies
cd frontend && npm install && cd ..

# 3. Build production binaries
cd crates/slopssh-tauri && cargo tauri build
```

Output artifacts:
- `crates/slopssh-tauri/target/release/bundle/deb/slopssh_0.1.0_amd64.deb` — Debian package
- `crates/slopssh-tauri/target/release/bundle/appimage/slopssh_0.1.0_amd64.AppImage` — AppImage

### Windows

```powershell
# 1. Clone the repository
git clone https://github.com/slop-ops/SlopSSH.git
cd SlopSSH

# 2. Install frontend dependencies
cd frontend && npm install && cd ..

# 3. Build production binaries
cd crates\slopssh-tauri && cargo tauri build
```

Output artifacts:
- `crates\slopssh-tauri\target\release\bundle\nsis\SlopSSH_0.1.0_x64-setup.exe` — NSIS installer
- `crates\slopssh-tauri\target\release\bundle\msi\SlopSSH_0.1.0_x64_en-US.msi` — MSI installer

## Development

```bash
# Development mode (hot reload)
cd crates/slopssh-tauri && cargo tauri dev

# Run Rust tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check

# Frontend
cd frontend && npm run build
cd frontend && npm run check
cd frontend && npm run test
cd frontend && npm run test:e2e
```

## Release Process

1. Update the version in `Cargo.toml` and `crates/slopssh-tauri/tauri.conf.json`
2. Commit the version bump
3. Create and push a git tag: `git tag v0.1.0 && git push origin v0.1.0`
4. The GitHub Actions release workflow will automatically:
   - Run all tests (Rust + frontend)
   - Build Linux (`.deb` + `.AppImage`) and Windows (`.exe` + `.msi`) artifacts
   - Create a GitHub Release with all artifacts attached

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

Config lives in `~/.config/slopssh/` (XDG on Linux) or `%APPDATA%\slopssh\` (Windows):

| File | Purpose |
|------|---------|
| `settings.toml` | App settings |
| `sessions.json` | Saved sessions tree |
| `snippets.json` | Command snippets |
| `window_bounds.json` | Window position/size |
| `plugins/` | WASM plugin files |
| `plugin_settings.json` | Per-plugin settings |

## Session File Format

`.slopssh` files are JSON containing a `SessionInfo` struct:

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

See [crates/slopssh-plugins/README.md](crates/slopssh-plugins/README.md) for the plugin API.

## License

GPL-3.0
