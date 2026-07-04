# Bug Fix & Feature Implementation Plan

## Issue Analysis Summary

After thorough codebase analysis, here are the root causes mapped to each reported issue:

| # | Issue | Root Cause |
|---|-------|------------|
| 1 | Passwords not saved | `NewSessionDialog` passes password as `password_key` but never calls `credentialSave`. On connect, password comes from UI param, not credential store. |
| 2 | Password input at bottom of sidebar | `Sidebar.svelte:134-148` has a permanent `.password-section` — should be a modal dialog. |
| 3 | Cannot minimize sidebar | Toggle button exists but has no visual indicator of collapsed state; sidebar just disappears. |
| 4 | Dark theme colors too bright | `.session-name` uses `--text-primary: #e0e0e0` which is fine, but `.session-host` uses `--text-tertiary: #6b7280` — may need darker variant. |
| 5 | Split session broken | No split session feature exists in the codebase — needs implementation or removal from UI. |
| 6 | Terminal visual bugs on tab switch | `display: contents`/`none` toggle doesn't properly hide xterm; `fitAddon.fit()` doesn't fire on re-display. |
| 7 | Session logic incorrect | Sidebar shows saved sessions as tabs on connect. No separation between saved hosts and active sessions. |
| 8 | Files error "no sftp session" | `FileBrowser` receives `activeSessionId` but SFTP isn't auto-connected. The error is from `sftp_cmds.rs:22` — no SFTP session exists yet. |
| 9 | Disk/Load/Ports tools broken | Same as #8 — `remoteExec` needs valid SSH handle, but `sessionId` may be wrong or session not connected. |
| 10 | No tool data caching | Every tool re-fetches on mount via `$effect` — no caching layer. |
| 11 | Terminal auto new-line on tab switch | ResizeObserver triggers `fitAddon.fit()` when terminal becomes visible, causing unwanted scroll/resize. |
| 12 | Local terminal hides sessions | `activeSessionId` is set to `''` when local terminal tab is active → `{#if activeSessionId}` block shows empty state. |
| 13 | Session menu items broken | `connect` uses `selectedSessionId` (never set); `disconnect` uses `activeSessionId` (may not be connected); `duplicate`/`delete` need `sidebarSessions` ref. |
| 14 | Snippets hide sessions | Same as #12 — snippet toggle is inside `TerminalHolder` which only renders when `activeSessionId` is set. |

### Additional Bugs Found

- **A1**: `btoa()` in `Terminal.svelte:67` only handles Latin1 — multi-byte UTF-8 keystrokes will corrupt.
- **A2**: `Terminal.svelte:151-158` onDestroy doesn't call `sshCloseShell` — SSH shell channel leaks.
- **A3**: Tab restore generates new UUIDs for tabs — old channel IDs from `tab_state.json` won't match any active SSH channels.
- **A4**: `NewSessionDialog.svelte:32` sends password as `password_key` field but `SessionInfo.password_key` is meant to be a credential store key, not the raw password.
- **A5**: `DiskAnalyzer.svelte:31` calls `remoteExec` which returns `CommandResult` with `stdout` as `Vec<u8>` — frontend expects string.
- **A6**: `PortViewer.svelte:48` skips first line (`slice(1)`) unconditionally — may skip data if no header line.

---

## Sprint 1: Critical Session & Connection Architecture (Issues 7, 12, 13, 14) ✅ COMPLETED

**Goal**: Redesign sidebar to separate saved hosts from active sessions. Fix local terminal and snippets not breaking the UI.

### 1.1 Redesign Sidebar: Saved Hosts vs Active Sessions

**Files**: `Sidebar.svelte`, `AppShell.svelte`, `TerminalHolder.svelte`

- Split sidebar into two sections:
  - **Saved Hosts**: List from `SessionStore` with "Connect" button. Each host click opens a connect flow (password dialog if needed).
  - **Active Sessions**: List of currently connected SSH sessions. Each shows its terminal tabs underneath.
- Remove the bottom password input from sidebar.
- `activeSessionId` should always refer to an active SSH session, never empty when tabs exist.

```
Sidebar layout:
┌─────────────────────┐
│ SAVED HOSTS         │
│   host1  [Connect]  │
│   host2  [Connect]  │
│   folder1/          │
│     host3 [Connect]  │
├─────────────────────┤
│ ACTIVE SESSIONS     │
│   ● host1 (2 tabs)  │
│   ● host2 (1 tab)   │
└─────────────────────┘
```

### 1.2 Fix `activeSessionId` for Local Terminals

**Files**: `AppShell.svelte`

- Decouple the "viewing session" from "active session for tools/files".
- When a local terminal tab is active, keep `activeSessionId` pointing to the last active SSH session (if any).
- Add a separate `viewSessionId` concept for determining which SSH session's files/tools to show.

### 1.3 Fix Snippets Not Breaking UI

**Files**: `TerminalHolder.svelte`, `AppShell.svelte`

- Snippet panel toggle is inside `TerminalHolder` which is only rendered when `activeSessionId` is set.
- Move snippet panel to be rendered independently or ensure it doesn't affect session visibility.

### 1.4 Fix Session Menu Items

**Files**: `AppShell.svelte`

- `connect` menu: Show password dialog, then call `sshConnect` + `handleConnect`.
- `disconnect` menu: Call `sshDisconnect`, remove the tab for that session.
- `duplicate` menu: Use `sidebarSessions` to find session, create copy.
- `delete` menu: Confirm, then delete from store, reload sidebar.

---

## Sprint 2: Password & Credential Flow (Issues 1, 2) ✅ COMPLETED

**Goal**: Save passwords properly and show password prompt as a modal.

### 2.1 Save Password on Session Create

**Files**: `NewSessionDialog.svelte`, `session_cmds.rs`

- In `NewSessionDialog.save()`, after `createSession`, call `api.credentialSave(sessionId, 'password', password)` if auth type is password.
- Change `password_key` to be a generated credential key (e.g., the session ID), not the raw password.
- For public key auth, save passphrase similarly.

### 2.2 Load Password on Connect

**Files**: `Sidebar.svelte` (or new `PasswordDialog.svelte`), `ssh_cmds.rs`

- On connect, first try `credentialGet(sessionId, 'password')`.
- If found, use it directly.
- If not found, show password dialog modal.
- After successful connect, offer to save password via `credentialSave`.

### 2.3 Replace Sidebar Password Input with Modal

**Files**: `Sidebar.svelte`, new `PasswordDialog.svelte`

- Remove `.password-section` from `Sidebar.svelte`.
- Create `PasswordDialog.svelte` modal component.
- On session click: check credential store → if no password, show dialog → on submit, connect.

### 2.4 Fix `password_key` Semantics

**Files**: `NewSessionDialog.svelte`, `session/info.rs`

- `SessionInfo.password_key` should store the credential store key (not the raw password).
- When creating a session, generate a stable key like `session.id` and store the actual password in `CredentialStore`.

---

## Sprint 3: SFTP Auto-Connect & Tool Fixes (Issues 8, 9, 10) ✅ COMPLETED

**Goal**: Fix Files panel, make tools work, add caching.

### 3.1 Auto-Connect SFTP on First File Browse

**Files**: `FileBrowser.svelte`, `sftp_cmds.rs`

- In `FileBrowser.loadHome()`, before calling `sftpHome`, call `api.sftpConnect(sessionId)`.
- Handle the case where SFTP is already connected (ignore "already exists" error).
- Alternatively, make `sftp_home` auto-create the SFTP session if one doesn't exist.

### 3.2 Fix `remoteExec` for Tools

**Files**: `tools_cmds.rs`, `DiskAnalyzer.svelte`, `SystemLoad.svelte`, `PortViewer.svelte`

- `remoteExec` needs `ssh_manager.get_handle(session_id)` — verify the SSH session is connected.
- `CommandResult.stdout` is `Vec<u8>` — the frontend calls `.split('\n')` on it. Ensure the Rust side converts to string first, or the frontend handles it.
- Fix `DiskAnalyzer.svelte:31` — `result.stdout` needs `.split('\n')` to work on a string, not bytes.

### 3.3 Add Tool Data Caching

**Files**: All tool components, new `toolCache.ts`

- Create a simple in-memory cache keyed by `{sessionId, toolId}`.
- Each tool checks cache first; only fetches if cache is empty.
- Add a "Refresh" button to each tool that clears its cache and re-fetches.
- Cache is invalidated when session disconnects.

```
Cache structure:
Map<string, { data: any, timestamp: number }>

Key = `${sessionId}:${toolId}`
```

### 3.4 Fix `PortViewer` Parsing

**Files**: `PortViewer.svelte`

- `parsePorts` skips first line unconditionally (`slice(1)`). `ss -tlnp` may not always have a header, or may have different formats.
- Parse the header line to determine column positions.
- Handle both `ss` and `netstat` output formats.

---

## Sprint 4: Terminal Fixes (Issues 6, 11)

**Goal**: Fix terminal visual bugs on tab switch.

### 4.1 Fix Tab Display Toggle

**Files**: `TerminalHolder.svelte`

- Replace `display: contents`/`none` with `visibility: hidden; position: absolute; height: 0; overflow: hidden` for inactive tabs.
- This keeps the xterm instance mounted and sized correctly.
- Call `fitAddon.fit()` on the newly active terminal after switching.

```svelte
<div class="terminal-panel"
  style:visibility={activeTabId === tab.id ? 'visible' : 'hidden'}
  style:position={activeTabId === tab.id ? 'relative' : 'absolute'}
  style:height={activeTabId === tab.id ? '100%' : '0'}
  style:overflow={activeTabId === tab.id ? 'visible' : 'hidden'}
>
```

### 4.2 Prevent Unwanted Resize on Tab Switch

**Files**: `Terminal.svelte`

- Add a guard in the ResizeObserver callback: only call `fitAddon.fit()` if the terminal is visible (`offsetParent !== null`).
- After tab switch, manually call `fitAddon.fit()` and `terminal.focus()` on the now-visible terminal.

### 4.3 Fix UTF-8 Encoding

**Files**: `Terminal.svelte`, `LocalTerminal.svelte`

- Replace `btoa(data)` with a proper UTF-8 encoder:
  ```ts
  function encodeBase64(str: string): string {
    const bytes = new TextEncoder().encode(str)
    return btoa(String.fromCharCode(...bytes))
  }
  ```
- Or use `Buffer.from(str).toString('base64')` if available.

### 4.4 Clean Up Shell Channel on Destroy

**Files**: `Terminal.svelte`

- In `onDestroy`, call `api.sshCloseShell(sessionId, channelId)` before disposing the terminal.

---

## Sprint 5: UI Polish (Issues 3, 4, 5)

**Goal**: Sidebar minimize, dark theme fixes, split session.

### 5.1 Sidebar Collapse Animation

**Files**: `AppShell.svelte`, `Sidebar.svelte`

- Add a collapsed state that shows only icons (session initials or dots).
- Animate width transition from 260px to 48px.
- Show tooltip with session name on hover when collapsed.
- Toggle button should show `>` when collapsed, `<` when expanded.

### 5.2 Dark Theme Color Adjustments

**Files**: `app.css`, `Sidebar.svelte`

- Adjust `.session-host` color to be dimmer in dark theme.
- Review all sidebar colors against the dark theme palette.
- Ensure folder names, session names, and host text have proper contrast hierarchy.

### 5.3 Split Session Implementation

**Files**: `TerminalHolder.svelte`, `AppShell.svelte`

- Add horizontal/vertical split button to tab bar.
- When split, render two terminal panels side-by-side or stacked.
- Both panels show terminals from the same session.
- Store split state in tab configuration.

---

## Sprint 6: Remaining Fixes & Cleanup

### 6.1 Fix Tab State Restore

**Files**: `AppShell.svelte`

- On restore, verify each saved tab's session is still connected.
- Remove tabs whose sessions are no longer active.
- Reconnect to previously active tab.

### 6.2 Fix `ssh_connect` to Save Password After Success

**Files**: `ssh_cmds.rs`

- After successful `ssh_manager.connect()`, if password was provided and session uses password auth, save it to credential store.
- This ensures password is persisted even when connecting from sidebar (not just from NewSessionDialog).

### 6.3 Error Handling Improvements

- Add proper error messages for all tool failures.
- Show connection status indicator on active session tabs.
- Handle SSH disconnect events to update UI state.

---

## Implementation Order

```
Sprint 1 (Critical Architecture)  ✅ COMPLETED
  ├── 1.1 Redesign sidebar ✅
  ├── 1.2 Fix activeSessionId ✅
  ├── 1.3 Fix snippets ✅
  └── 1.4 Fix session menu ✅

Sprint 2 (Password & Credentials)  ✅ COMPLETED
  ├── 2.1 Save password on create ✅
  ├── 2.2 Load password on connect ✅
  ├── 2.3 Password modal dialog ✅
  └── 2.4 Fix password_key semantics ✅

Sprint 3 (SFTP & Tools)  ✅ COMPLETED
  ├── 3.1 Auto-connect SFTP ✅
  ├── 3.2 Fix remoteExec ✅
  ├── 3.3 Tool caching ✅
  └── 3.4 Fix port parsing ✅

Sprint 4 (Terminal)  ← Mostly independent
  ├── 4.1 Fix tab display
  ├── 4.2 Fix resize on switch
  ├── 4.3 Fix UTF-8
  └── 4.4 Shell cleanup

Sprint 5 (UI Polish)  ← Depends on Sprint 1
  ├── 5.1 Sidebar collapse
  ├── 5.2 Dark theme
  └── 5.3 Split session

Sprint 6 (Cleanup)  ← After all others
  ├── 6.1 Tab restore
  ├── 6.2 Password persistence
  └── 6.3 Error handling
```

## Key Files to Modify

| File | Sprints | Changes |
|------|---------|---------|
| `frontend/src/components/layout/AppShell.svelte` | 1,2,5,6 | Session architecture, password flow, sidebar collapse |
| `frontend/src/components/layout/Sidebar.svelte` | 1,2,5 | Redesign, password modal, collapse |
| `frontend/src/components/terminal/TerminalHolder.svelte` | 1,4,5 | Tab display fix, split session |
| `frontend/src/lib/terminal/Terminal.svelte` | 4 | Resize fix, UTF-8, cleanup |
| `frontend/src/components/terminal/LocalTerminal.svelte` | 4 | UTF-8 fix |
| `frontend/src/components/session/NewSessionDialog.svelte` | 2 | Password save |
| `frontend/src/components/files/FileBrowser.svelte` | 3 | SFTP auto-connect |
| `frontend/src/components/tools/DiskAnalyzer.svelte` | 3 | stdout type fix |
| `frontend/src/components/tools/SystemLoad.svelte` | 3 | Caching |
| `frontend/src/components/tools/PortViewer.svelte` | 3 | Parsing fix, caching |
| `frontend/src/components/tools/ToolsPanel.svelte` | 3 | Caching |
| `frontend/src/app.css` | 5 | Dark theme colors |
| `frontend/src/lib/api/invoke.ts` | 2 | Credential save on connect |
| `crates/slopssh-tauri/src/commands/ssh_cmds.rs` | 2,6 | Password persistence |
| `crates/slopssh-tauri/src/commands/session_cmds.rs` | 2 | Password save on create |
| `crates/slopssh-tauri/src/commands/sftp_cmds.rs` | 3 | Auto-create SFTP session |
| `crates/slopssh-tauri/src/commands/tools_cmds.rs` | 3 | String conversion |

## New Files to Create

| File | Sprint | Purpose |
|------|--------|---------|
| `frontend/src/components/session/PasswordDialog.svelte` | 2 | Modal password prompt |
| `frontend/src/lib/utils/toolCache.ts` | 3 | Tool data caching utility |
