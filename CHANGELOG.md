# Changelog

All notable changes to SlopSSH (Rust/Tauri rewrite) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Session backup rotation: keeps last 5 backup copies (`sessions.json.bak.1` through `.bak.5`) before each save
- Tab state persistence: open tabs saved to disk and restored on next launch
- Auto-update download: download release assets from GitHub to config directory
- Dynamic tray tooltip: shows active SSH sessions and transfer counts in system tray
- Portable mode: detect `portable.marker` file next to binary for relative config paths
- File size check in editor: refuses to open files larger than 10 MB
- `is_portable()` function in config paths module
- `cleanup_tasks()` and `abort_all()` methods on TransferEngine for spawned task management
- `download_update()` method on UpdateChecker for fetching release assets
- `update_tray_tooltip` IPC command for dynamic tray tooltip updates
- `download_update` IPC command for downloading updates
- `save_tab_state`, `load_tab_state`, `clear_tab_state` IPC commands
- `tab_state.rs` module with `TabState` and `SavedTab` structs
- `fileTooLarge` i18n key in all 7 languages

### Changed

- **AppState split into per-concern mutexes** — individual `tokio::sync::Mutex` per field reduces lock contention across all 72 IPC commands
- `TransferEngine::spawn_upload()` and `spawn_download()` now track spawned tasks in a `HashMap<String, JoinHandle<()>>`
- `SessionStore` now supports configurable file paths for testing (`save_to()`, `load_from()`)
- `get_settings` and `open_in_editor` are now async (previously used `blocking_lock()`)
- Config paths module now supports portable mode with `portable.marker` detection

### Removed

- Outer `Mutex<AppState>` wrapper — replaced with internal per-field mutexes
- Unused `state` parameters from commands that don't access AppState

### Documentation

- Added `///` doc comments to all public items in `slopssh-core` (300+ items)
- Added `//!` module-level documentation to all modules in `slopssh-core` (30+ modules)
- Added `ARCHITECTURE.md` describing data flow, IPC model, plugin system, SSH lifecycle

### Tests

- Session store tests: 4 new tests for backup rotation (total: 10)
- Tab state tests: 3 new tests for serialization (total: 3)
- Config paths tests: 1 new test for portable mode detection (total: 12)
- Total: 300 Rust tests + 67 frontend tests = 367 total

## [0.1.0] - 2026-04-15

### Added

- Initial release of SlopSSH Rust/Tauri rewrite
- SSH engine with russh (password, pubkey, keyboard-interactive auth)
- SFTP file browser with upload/download
- Terminal emulator with xterm.js
- Session management with folder tree
- Port forwarding (local and remote)
- Jump host tunneling
- X11 forwarding
- Connection pooling
- System tools (process viewer, log viewer, disk analyzer, search, system info, system load, port viewer)
- SSH key manager
- Command snippets
- Plugin system (WASM sandbox)
- i18n with 7 languages (English, Chinese, German, Spanish, French, Portuguese, Russian)
- Dark/light theme system
- Keyboard shortcuts
- File type associations
- System tray integration
- Auto-updater (GitHub API)
- Cross-platform packaging (Linux deb/AppImage, Windows NSIS/MSI)
- CI/CD via GitHub Actions
- 292 Rust unit tests
- 67 frontend unit tests
- 3 Playwright E2E test specs
