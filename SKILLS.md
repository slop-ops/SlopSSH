# SKILLS.md — Muon SSH Production Hardening Plan

All features are implemented. This plan covers making the application production-grade.

**Total findings from audit: 4 CRITICAL, 13 HIGH, 22 MEDIUM, 10 LOW**

---

## Phase 12: Critical Security Fixes

**Goal:** Eliminate all security vulnerabilities. No release without these.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 12.1 | Fix Content Security Policy | CRITICAL | `tauri.conf.json` has `"csp": null`. Set proper CSP: `"default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:"` |
| 12.2 | Host key verification: prompt user instead of auto-accepting | CRITICAL | `connection.rs:121-125` auto-adds unknown host keys. Return `HostKeyStatus::Unknown` to frontend, show fingerprint dialog, only add on user confirmation. |
| 12.3 | Jump host key verification | CRITICAL | `jump_host.rs:27-32` `check_server_key` always returns `Ok(true)`. Implement real host key verification using `HostKeyVerifier`. |
| 12.4 | Fix shell command injection in tool panels | CRITICAL | `LogViewer.svelte`, `DiskAnalyzer.svelte`, `SearchPanel.svelte`, `ProcessViewer.svelte` interpolate user input directly into remote shell commands. Use Rust-side `shell_escape()` for all paths passed to `remote_exec`. |
| 12.5 | Encrypt credential store fallback | HIGH | `credentials/store.rs` `FileBackend` stores passwords in plaintext JSON. Encrypt at rest using `ring` AES-256-GCM with a machine-derived key. Warn user when using file fallback. |
| 12.6 | Fix download entire-file-into-RAM | CRITICAL | `file_transfer/engine.rs` `perform_download` reads entire remote file into memory. Stream using chunked SFTP reads (8KB chunks). Cap at configurable max (default 100MB for buffer ops). |

---

## Phase 13: Build System & CI

**Goal:** `cargo tauri build` produces working installers on all platforms.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 13.1 | Fix tauri.conf.json paths | CRITICAL | `frontendDist: "../dist"` resolves to `crates/dist/`. Fix to `"../../frontend/dist"`. Fix `beforeBuildCommand` and `beforeDevCommand` to use `npm --prefix ../../frontend`. |
| 13.2 | Run `cargo fmt` | HIGH | 97 lines of formatting violations. Run `cargo fmt` and add to CI. |
| 13.3 | Fix CI build-linux job | HIGH | Missing `npm ci` step for frontend. Add before `cargo tauri build`. |
| 13.4 | Add macOS and Windows CI jobs | MEDIUM | CI only builds on Ubuntu. Add `build-macos` and `build-windows` jobs. |
| 13.5 | Create release workflow | MEDIUM | Add `.github/workflows/release.yml` triggered on tag push. Upload deb, AppImage, dmg, exe artifacts to GitHub Release. |
| 13.6 | Fix `$schema` URL in tauri.conf.json | LOW | Current URL returns 404. Use `https://raw.githubusercontent.com/tauri-apps/tauri/refs/heads/dev/packages/cli/config.schema.json`. |
| 13.7 | Change bundle identifier | MEDIUM | `com.muon-ssh.app` conflicts with macOS `.app` extension. Change to `com.muon-ssh.desktop`. |
| 13.8 | Remove duplicate `#[allow(dead_code)]` | LOW | `state.rs:22-23` has same attribute twice. Remove duplicate. |
| 13.9 | Remove `greet` command | LOW | Template code left in production. Remove from `system_cmds.rs` and `main.rs` invoke handler. |
| 13.10 | Fix ESLint or remove lint script | MEDIUM | `eslint` is not installed but `npm run lint` references it. Either add eslint + config or remove the script. |

---

## Phase 14: Error Handling & Resilience

**Goal:** No silent failures. Every error path is visible to user or logs.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 14.1 | Add tracing to all SSH operations | HIGH | Zero tracing in SSH connect/disconnect, channel open/close, auth attempts. Add `info!` for lifecycle, `debug!` for data flow, `warn!` for failures. |
| 14.2 | Add tracing to file transfers | HIGH | Zero tracing in upload/download. Add `info!` for start/complete, `debug!` for chunk progress, `warn!` for failures. |
| 14.3 | Add tracing to SFTP operations | HIGH | Zero tracing in list_dir, mkdir, remove, rename, read, write. Add `debug!` for each operation. |
| 14.4 | Add tracing to Tauri IPC layer | HIGH | Zero tracing in any IPC command. Add `debug!` on entry, `warn!` on error for all 67 commands. |
| 14.5 | Add file-based log output | MEDIUM | `logging/mod.rs` computes log_dir but never uses it. Add `tracing-appender` file writer alongside stderr. |
| 14.6 | Fix silently swallowed host key save | HIGH | `connection.rs:123` `let _ = host_keys::add_host_key(...)` — failure to save means user re-prompted every time. Log the error. |
| 14.7 | Fix silently swallowed port forward bind failure | HIGH | `port_forward.rs:87-89` — bind failure returns success with dead forward. Return error instead. |
| 14.8 | Fix jump host empty password fallback | HIGH | `jump_host.rs:176` `unwrap_or_default()` converts missing password to empty string. Return auth error instead. |
| 14.9 | Fix malformed jump host JSON silently skipped | HIGH | `session_manager.rs:66` `.filter_map(|jh| serde_json::from_str(jh).ok())` — typos in config silently ignored. Log warning. |
| 14.10 | Fix upload file shutdown error ignored | MEDIUM | `engine.rs:218` `let _ = remote_file.shutdown().await` — could lose data. Log warning. |
| 14.11 | Fix terminal output event failure silent | MEDIUM | `ssh_cmds.rs:99` `let _ = app_clone.emit(...)` — if emit fails, user sees no terminal output. Log error. |
| 14.12 | Replace `expect()` in tray icon creation | MEDIUM | `menu.rs:138` `.expect("Failed to load tray icon")`. Return Result, gracefully degrade without tray. |
| 14.13 | Add panic hook for crash logging | HIGH | No crash reporting. Add `std::panic::set_hook` that logs to file and shows error dialog. |
| 14.14 | Fix frontend empty `catch {}` blocks | MEDIUM | `SettingsDialog.svelte` has 7 empty catch blocks. Show errors to user via error state. |

---

## Phase 15: Input Validation & Settings

**Goal:** Invalid inputs are rejected early with clear messages.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 15.1 | Add `Settings::validate()` method | HIGH | Settings accepts any values (font_size=0, timeout=0, etc.). Add validation that clamps to valid ranges. Call in `save()`. |
| 15.2 | Add `SessionInfo::validate()` method | HIGH | Allows empty host, port 0, PublicKey auth with no key_path. Validate and return errors. |
| 15.3 | Validate host/port before connection | HIGH | Empty or malformed host produces confusing DNS errors. Validate format early. |
| 15.4 | Validate SFTP paths | MEDIUM | Paths with `..` traversal could be unexpected. Normalize and validate. |
| 15.5 | Extract shared `shell_escape()` utility | MEDIUM | Duplicated in `sftp_cmds.rs` and `tools_cmds.rs`. Move to `muon-core/src/utils.rs`. |
| 15.6 | Add validation to port forwarding | MEDIUM | No validation of bind_port=0 or bind_host=0.0.0.0. Validate inputs. |

---

## Phase 16: Frontend Type Safety & Quality

**Goal:** Zero `any` types. TypeScript strict mode. All components properly typed.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 16.1 | Enable TypeScript strict mode | HIGH | `tsconfig.app.json` missing `strict: true`. Add it and fix all resulting errors. |
| 16.2 | Define typed interfaces for IPC | HIGH | `invoke.ts` has 30+ functions returning `Promise<any>`. Define interfaces: `SessionInfo`, `Settings`, `FileEntry`, `TransferProgress`, `Snippet`, `PluginInfo`, etc. |
| 16.3 | Replace `any` in Svelte components | HIGH | `SettingsDialog`, `FileBrowser`, `Sidebar`, `ProcessViewer`, `PortViewer`, `KeyManager`, `SnippetPanel`, `ImportDialog`, `PortForwarding` all use `any`. Type everything. |
| 16.4 | Wire i18n to all components | HIGH | i18n system exists with 7 language files but NO component uses `t()`. All strings hardcoded in English. Replace every visible string with `t('key')`. |
| 16.5 | Call `loadLocale()` on language change | MEDIUM | `SettingsDialog` saves language but never calls `loadLocale()`. Wire it up. |
| 16.6 | Wire terminal settings to Terminal components | MEDIUM | `Terminal.svelte` and `LocalTerminal.svelte` hardcode `darkTheme`, `fontFamily: 'JetBrains Mono'`, `fontSize: 14`. Read from settings. |
| 16.7 | Wire light theme to terminals | MEDIUM | `lightTheme` exists in `themes.ts` but is never used. Apply based on theme setting. |
| 16.8 | Fix NewSessionDialog password not sent | MEDIUM | Password and keyPath collected in form but not passed to `createSession()`. Fix data flow. |
| 16.9 | Replace hardcoded hex colors with CSS variables | MEDIUM | `TerminalHolder`, `ProcessViewer`, `LogViewer`, `DiskAnalyzer`, `SearchPanel`, `SystemLoad`, `PortViewer`, `KeyManager`, `PortForwarding`, `SnippetPanel`, `NewSessionDialog` all use hardcoded colors. Use CSS variables. |
| 16.10 | Fix `document.execCommand()` deprecated calls | LOW | `AppShell.svelte` uses `document.execCommand('copy'/'paste'/'selectAll')`. Use Clipboard API. |
| 16.11 | Fix TransferQueue polling | LOW | Polls every 2s instead of listening to `transfer-progress-{id}` events. Switch to event-driven. |
| 16.12 | Implement empty menu action handlers | MEDIUM | `import_sessions`, `quit`, `connect`, `disconnect`, `duplicate`, `delete_session`, `local_terminal`, `about`, `check_updates` are all no-ops. Implement each. |

---

## Phase 17: Test Coverage

**Goal:** Meaningful test coverage across all layers.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 17.1 | Add tests for `remote_exec` module | HIGH | `RemoteExecutor::execute()` has zero tests. Mock SSH channel, test command execution, timeout, exit codes. |
| 17.2 | Add tests for `session_manager` | HIGH | All methods untested. Test connect/disconnect, shell open/close, SFTP channel management with mocks. |
| 17.3 | Add tests for `jump_host` | HIGH | All methods untested. Test multi-hop connection logic with mocked russh. |
| 17.4 | Add tests for `key_manager` | HIGH | All methods untested. Test key generation, deployment, listing. |
| 17.5 | Add tests for `connection_pool` | HIGH | All methods untested. Test get_or_create, release, cleanup, close. |
| 17.6 | Add tests for `host_keys` verification | HIGH | `verify()` and `add_host_key()` untested. Test known/unknown/changed key scenarios. |
| 17.7 | Add integration tests for IPC commands | HIGH | All 67 Tauri commands have zero test coverage. Create test harness with mock `AppState`. Test session CRUD, SSH lifecycle, SFTP ops. |
| 17.8 | Add tests for `file_transfer/engine` spawn tasks | MEDIUM | `spawn_upload()` and `spawn_download()` untested. Test transfer lifecycle. |
| 17.9 | Add tests for settings validation | MEDIUM | Test `validate()` rejects invalid values, clamps to ranges. |
| 17.10 | Add tests for `shell_escape` utility | MEDIUM | Test edge cases: spaces, quotes, newlines, unicode, empty string. |
| 17.11 | Add frontend component tests | MEDIUM | Test `Sidebar`, `SettingsDialog`, `NewSessionDialog`, `FileBrowser` rendering and interaction. |
| 17.12 | Test target: 200+ Rust tests, 30+ frontend tests | HIGH | Current: 154 Rust + 14 frontend. Target: 200+ Rust + 30+ frontend. |

---

## Phase 18: Accessibility

**Goal:** All components have proper ARIA roles and keyboard navigation.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 18.1 | Add ARIA to all tool panels | MEDIUM | ProcessViewer, LogViewer, DiskAnalyzer, SearchPanel, SysInfoPanel, SystemLoad, PortViewer, KeyManager all missing `role="region"` and `aria-label`. |
| 18.2 | Add ARIA to FileList table | MEDIUM | No `role="grid"`, `role="row"`, `role="columnheader"`. Add proper table ARIA pattern. |
| 18.3 | Add ARIA to ContextMenu | MEDIUM | No `role="menu"`, `role="menuitem"`. Add keyboard arrow navigation between items. |
| 18.4 | Add ARIA to dialogs | MEDIUM | `FileEditor`, `ImportDialog`, `KeyManager` generate dialog missing `role="dialog"`, `aria-modal`. |
| 18.5 | Add keyboard navigation to ToolsPanel tabs | MEDIUM | No keyboard tab switching. Add arrow key navigation with `role="tablist"`. |
| 18.6 | Add keyboard navigation to FileList | MEDIUM | No arrow key row selection, Enter to open, Delete to delete. |
| 18.7 | Add `aria-label` to all buttons | LOW | Kill button, tab close buttons, action buttons lack descriptive labels. |
| 18.8 | Add focus trap to dialogs | MEDIUM | Dialogs don't trap focus. Tab key escapes to background. |
| 18.9 | Add focus management on dialog open/close | MEDIUM | Opening a dialog doesn't move focus. Closing doesn't restore focus. |

---

## Phase 19: Performance & Resource Management

**Goal:** No memory leaks. Efficient resource usage. Streaming I/O everywhere.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 19.1 | Stream file downloads | CRITICAL | See 12.6 — reads entire file into RAM. Implement chunked streaming. |
| 19.2 | Fix memory leaks in terminal components | HIGH | `Terminal.svelte` and `LocalTerminal.svelte` add `contextmenu` listener via `addEventListener` but never remove. Clean up in destroy. |
| 19.3 | Fix `setTimeout` cleanup in components | MEDIUM | `SettingsDialog` and `KeyManager` use `setTimeout` for clearing messages but never cancel on unmount. |
| 19.4 | Fix SystemLoad interval cleanup | MEDIUM | Interval created in `start()` but cleaned in separate `$effect`. Unify lifecycle. |
| 19.5 | Split AppState into per-concern mutexes | MEDIUM | Single `tokio::Mutex<AppState>` causes contention. Split into separate mutexes for settings, SSH manager, transfer engine, SFTP sessions. |
| 19.6 | Track spawned transfer tasks | MEDIUM | `engine.rs:65,101` discards `tokio::spawn` handles. Track in HashMap for cleanup on disconnect. |
| 19.7 | Add app shutdown cleanup | MEDIUM | No cleanup on app exit. SSH sessions, SFTP connections, local PTYs, and port forwards not cleanly closed. Add shutdown hook. |
| 19.8 | Fix blocking `std::process::Command` in async | HIGH | `key_manager.rs:149,229` uses blocking `Command` in async fn. Wrap in `tokio::task::spawn_blocking`. |
| 19.9 | Add file size check before editor open | LOW | `FileEditor.svelte` loads entire file into memory. Add size limit (default 10MB). |
| 19.10 | Code-split frontend bundle | LOW | Single 594KB JS chunk. Lazy-load tools panels and xterm.js addons with dynamic import. |

---

## Phase 20: Production Features

**Goal:** Features expected in a production desktop application.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 20.1 | Implement auto-update download | MEDIUM | `check_for_updates` queries GitHub API but never downloads/installs. Integrate `tauri-plugin-updater` or implement download + install. |
| 20.2 | Add session backup on save | MEDIUM | Single `sessions.json` file, no backup. Keep last 5 copies (`sessions.json.bak.1`, etc.). |
| 20.3 | Add session/tab state persistence | MEDIUM | App crash loses all open sessions/tabs. Periodically save open tab list, offer restore on next launch. |
| 20.4 | Add "running in background" indicator | LOW | Close button hides window with no visual feedback. Add tray tooltip showing active connections/transfers. |
| 20.5 | Add portable mode | LOW | Detect `portable.marker` file next to binary, use relative paths for config. |

---

## Phase 21: Documentation

**Goal:** All public APIs documented. Developer onboarding clear.

| # | Task | Severity | Details |
|---|------|----------|---------|
| 21.1 | Add doc comments to all `muon-core` public items | MEDIUM | Zero doc comments on any public function, struct, or enum. Add `///` doc comments to all public items. |
| 21.2 | Add module-level documentation | MEDIUM | No `//!` module docs. Add to each `mod.rs` describing module purpose and key types. |
| 21.3 | Add ARCHITECTURE.md | LOW | Describe data flow, IPC model, plugin system, SSH lifecycle. |
| 21.4 | Add CHANGELOG.md | LOW | Track version changes. |

---

## Priority Order for Next Sessions

| Session | Phases | Focus |
|---------|--------|-------|
| 11 | 12, 13 | **Security fixes + build system** — App must build and be secure |
| 12 | 14, 15 | **Error handling + validation** — No silent failures |
| 13 | 16, 18 | **Frontend quality + accessibility** — Type safety, i18n, ARIA |
| 14 | 17 | **Test coverage** — Get to 200+ tests |
| 15 | 19, 20 | **Performance + production features** — Streaming, cleanup, auto-update |
| 16 | 21 | **Documentation** — Doc comments, architecture docs |

---

## Verification Checklist (run before committing)

```bash
# Must pass:
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo test --workspace
cd frontend && npm run build
cd frontend && npm run check
cd frontend && npm run test

# Should pass (requires dev server):
cd frontend && npm run test:e2e

# Must work:
cd crates/muon-tauri && cargo tauri build
```

## Current Stats

- **Rust tests:** 154
- **Frontend tests:** 14
- **IPC commands:** 67
- **Known issues:** 4 CRITICAL, 13 HIGH, 22 MEDIUM, 10 LOW
