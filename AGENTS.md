# AGENTS.md

## Project: Muon SSH — Rust/Tauri Rewrite

Cross-platform SSH/SCP/SFTP client built with Rust, Tauri 2, and Svelte 5.

**Current state:** All features implemented. Production hardening in progress.

### Build Commands

```bash
# Dev mode (hot reload frontend + Rust)
cd crates/muon-tauri && cargo tauri dev

# Build Tauri app (release)
cd crates/muon-tauri && cargo tauri build

# Run Rust tests
cargo test --workspace

# Run Rust tests with output
cargo test --workspace -- --nocapture

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings

# Format check / apply
cargo fmt --check
cargo fmt

# Frontend
cd frontend && npm run dev
cd frontend && npm run build
cd frontend && npm run check
cd frontend && npm run test
cd frontend && npm run test:e2e
```

### Lint/Typecheck Commands (run after every code change)

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cd frontend && npm run build
cd frontend && npm run check
cd frontend && npm run test
```

### Architecture

- **`crates/muon-core/`** — Pure Rust library. No Tauri or UI deps. SSH, session, config, business logic.
- **`crates/muon-tauri/`** — Thin Tauri binary. Wires muon-core to IPC. No business logic.
- **`frontend/`** — Svelte 5 + xterm.js. Communicates via Tauri `invoke()` and events only.
- **`crates/muon-plugins/`** — Example WASM plugins (standalone crates, not in workspace).

### Key Conventions

1. **Rust style:** `rustfmt` defaults. `clippy -D warnings`. No `unwrap()` in production — use `anyhow::Result` or `thiserror`.

2. **Async runtime:** Tokio multi-thread. All I/O async. `tokio::sync::Mutex` for async state (not `std::sync::Mutex`).

3. **Error handling:** `anyhow::Result` in `muon-tauri`. `thiserror` in `muon-core`. Never panic in library code. Never silently swallow errors with `let _ =` unless cleanup — log with `tracing::warn!` at minimum.

4. **Logging:** Use `tracing::{info, debug, warn, error}`. `info!` for lifecycle events (connect, disconnect, transfer complete). `debug!` for data flow. `warn!` for recoverable errors. `error!` for failures.

5. **Serde:** Derive `Serialize, Deserialize`. Use `#[serde(default)]` for forward compat.

6. **Tauri IPC:** Flat param structs. Return `Result<T, String>` where `T: Serialize`. Binary data base64-encoded.

7. **Terminal data flow:** SSH bytes -> base64 -> Tauri `terminal-output-{id}` event -> xterm.js. xterm `onData` -> `ssh_write_shell` command -> SSH channel.

8. **Frontend state:** Svelte 5 runes (`$state`, `$derived`, `$effect`). No legacy stores. Clean up event listeners in `$effect` return functions.

9. **No comments** in code unless explicitly requested.

10. **Naming:** Rust: `snake_case` fn/var, `PascalCase` types. Svelte: `PascalCase.svelte` components, `camelCase` fn/var.

11. **Dependencies:** Check `Cargo.toml` and `package.json` before adding. Prefer existing workspace crates.

12. **TypeScript:** Use strict types. No `any` — define interfaces. `tsconfig.app.json` should have `strict: true`.

13. **Security:** All user inputs (host, path, command) must be validated before use. Shell commands must use `shell_escape()`. Never interpolate user input directly into shell commands.

14. **i18n:** All user-visible strings must use `t('key')` from `$lib/utils/i18n`. No hardcoded English strings.

15. **Accessibility:** All interactive elements need `aria-label` or visible text. Dialogs need `role="dialog"` and `aria-modal`. Tables need proper `role` attributes. Focus management on dialog open/close.
