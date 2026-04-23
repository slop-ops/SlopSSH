# AGENTS.md

## Project: Muon SSH — Rust/Tauri Rewrite

This is a rewrite of the Java/Swing muon-ssh client to Rust/Tauri 2 with Svelte 5 frontend.

### Build Commands

```bash
# Build everything
cargo build

# Build Tauri app (release)
cd crates/muon-tauri && cargo tauri build

# Run in dev mode (hot reload frontend + Rust)
cd crates/muon-tauri && cargo tauri dev

# Run Rust tests
cargo test --workspace

# Run Rust tests with output
cargo test --workspace -- --nocapture

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --check

# Format apply
cargo fmt

# Frontend only
cd frontend && npm run dev
cd frontend && npm run build
cd frontend && npm run check        # Svelte type check
cd frontend && npm run lint         # ESLint
```

### Lint/Typecheck Commands (run after every code change)

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cd frontend && npm run check
cd frontend && npm run lint
```

### Architecture

- **`crates/muon-core/`** — Pure Rust library. No Tauri or UI dependencies. All SSH, session, config, and business logic lives here.
- **`crates/muon-tauri/`** — Thin Tauri binary. Wires muon-core to Tauri IPC commands. No business logic.
- **`frontend/`** — Svelte 5 + xterm.js. All UI. Communicates with Rust exclusively via Tauri `invoke()` and events.

### Key Conventions

1. **Rust style:** Follow `rustfmt` defaults. Use `clippy` with `-D warnings`. No `unwrap()` in production code — use `anyhow::Result` or `thiserror` error types.

2. **Async runtime:** Tokio with multi-thread runtime. All I/O is async. Use `tokio::sync::Mutex` (not `std::sync::Mutex`) for async-accessed state.

3. **Error handling:** `anyhow::Result` for application code in `muon-tauri`. `thiserror` derive for library error types in `muon-core`. Never panic in library code.

4. **Serde:** All config/session structs derive `Serialize, Deserialize`. Use `#[serde(default)]` for forward compatibility.

5. **Tauri IPC:** Command parameters are flat structs (no nested generics). Return `Result<T, String>` where `T: Serialize`. Binary data is base64-encoded.

6. **Terminal data flow:** SSH channel bytes -> base64 -> Tauri `terminal-output` event -> xterm.js `write()`. xterm.js `onData` -> Tauri `terminal-input` command -> SSH channel write.

7. **Frontend state:** Svelte 5 runes (`$state`, `$derived`, `$effect`). No Svelte stores (legacy API). Use runes exclusively.

8. **No comments** in code unless explicitly requested.

9. **Naming:** Rust: `snake_case` for functions/variables, `PascalCase` for types. Svelte: `PascalCase.svelte` for components, `camelCase` for functions/variables.

10. **Dependencies:** Check `Cargo.toml` and `package.json` before adding new dependencies. Prefer crates already in the workspace.

### Migration Reference

The original Java source is at `muon-app/src/main/java/muon/app/`. Refer to it when porting features. Key mapping:

| Java Package | Rust Location | Svelte Location |
|-------------|--------------|----------------|
| `muon.app.ssh` | `crates/muon-core/src/ssh/` | — |
| `muon.app.common` | `crates/muon-core/src/config/` | — |
| `muon.app.ui.components.session` | `crates/muon-core/src/session/` | `frontend/src/components/session/` |
| `muon.app.ui.components.session.terminal` | `crates/muon-core/src/ssh/channel.rs` | `frontend/src/components/terminal/` |
| `muon.app.ui.components.session.files` | `crates/muon-core/src/ssh/sftp.rs`, `muon-core/src/file_transfer/` | `frontend/src/components/files/` |
| `muon.app.ui.components.settings` | `crates/muon-core/src/config/settings.rs` | `frontend/src/components/settings/` |
| `muon.app.ui.laf` | — | `frontend/src/app.css` |
| `muon.app.util` | Various utility modules | `frontend/src/lib/utils/` |
