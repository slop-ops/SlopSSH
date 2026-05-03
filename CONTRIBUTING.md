# Contributing to SlopSSH

## Development Setup

1. Install prerequisites (see README.md)
2. Clone the repository
3. Run `cargo build` to verify Rust compilation
4. Run `cd frontend && npm install` for frontend deps
5. Run `cargo tauri dev` to start the development environment

## Code Style

### Rust
- Follow `rustfmt` defaults: `cargo fmt`
- Clippy with zero warnings: `cargo clippy --workspace --all-targets -- -D warnings`
- No `unwrap()` in production code — use `anyhow::Result` or `thiserror`
- No `panic!()`, `todo!()`, or `unimplemented!()` in production code
- Async with Tokio: use `tokio::sync::Mutex` for async state
- No comments unless explicitly requested

### Svelte/TypeScript
- Svelte 5 runes (`$state`, `$derived`, `$effect`) — no legacy stores
- PascalCase `.svelte` components, camelCase functions/variables
- No comments unless explicitly requested

### Architecture
- All business logic in `slopssh-core` — no Tauri/UI dependencies
- `slopssh-tauri` is a thin IPC bridge — no business logic
- Frontend communicates exclusively via Tauri `invoke()` and events

## Testing

```bash
# Rust tests
cargo test --workspace

# Frontend unit tests
cd frontend && npm test

# Frontend E2E tests
cd frontend && npm run test:e2e

# Type check
cd frontend && npm run check
```

## Pull Request Process

1. Create a feature branch from `master`
2. Make your changes
3. Ensure all tests pass: `cargo test --workspace && cd frontend && npm run build && npm run check`
4. Ensure clippy passes: `cargo clippy --workspace --all-targets -- -D warnings`
5. Submit PR with a clear description

## Commit Messages

Use conventional commit format:
- `feat(scope): description` for new features
- `fix(scope): description` for bug fixes
- `refactor(scope): description` for refactoring
- `test(scope): description` for tests
- `docs: description` for documentation
- `chore: description` for maintenance

## Adding New Features

1. Implement core logic in `crates/slopssh-core/src/`
2. Add Tauri IPC command in `crates/slopssh-tauri/src/commands/`
3. Register command in `main.rs` invoke handler
4. Add frontend API wrapper in `frontend/src/lib/api/invoke.ts`
5. Build UI component
6. Add tests
7. Update documentation
