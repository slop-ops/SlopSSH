<script lang="ts">
  import * as api from '$lib/api/invoke'
  import ImportDialog from '$components/session/ImportDialog.svelte'
  import PasswordDialog from '$components/session/PasswordDialog.svelte'
  import { t } from '$lib/utils/i18n'
  import type { SessionFolder, SessionInfo } from '$lib/types'

  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
    isLocal?: boolean
  }

  let {
    onConnect,
    onDisconnect,
    onNewSession,
    showImport = $bindable(false),
    selectedSessionId = $bindable(''),
    sessions: sessions = $bindable<SessionFolder | null>(null),
    tabs = [],
    connectedSessionIds = [],
  }: {
    onConnect: (id: string, name: string) => void
    onDisconnect: (sessionId: string) => void
    onNewSession: () => void
    showImport?: boolean
    selectedSessionId?: string
    sessions?: SessionFolder | null
    tabs?: Tab[]
    connectedSessionIds?: string[]
  } = $props()

  let connectingId = $state('')
  let error = $state('')
  let passwordDialogSession = $state<{ id: string; name: string } | null>(null)

  $effect(() => {
    loadSessions()
  })

  async function loadSessions() {
    try {
      sessions = await api.listSessions()
    } catch (e) {
      error = String(e)
    }
  }

  async function connect(id: string, name: string) {
    selectedSessionId = id
    connectingId = id
    error = ''
    try {
      await api.sshConnect(id)
      onConnect?.(id, name)
    } catch (e) {
      const msg = String(e)
      if (msg.includes('password_required')) {
        passwordDialogSession = { id, name }
      } else {
        error = msg
      }
    } finally {
      connectingId = ''
    }
  }

  async function handlePasswordSubmit(password: string, save: boolean) {
    if (!passwordDialogSession) return
    const { id, name } = passwordDialogSession
    passwordDialogSession = null
    connectingId = id
    error = ''
    try {
      await api.sshConnect(id, password, save)
      onConnect?.(id, name)
    } catch (e) {
      error = String(e)
    } finally {
      connectingId = ''
    }
  }

  function handlePasswordCancel() {
    passwordDialogSession = null
  }

  async function deleteSession(id: string) {
    try {
      await api.deleteSession(id)
      await loadSessions()
    } catch (e) {
      error = String(e)
    }
  }

  function getSessionName(sessionId: string): string {
    if (!sessions) return sessionId
    const found = findSessionInTree(sessions, sessionId)
    return found?.name || found?.host || sessionId
  }

  function findSessionInTree(folder: SessionFolder, id: string): SessionInfo | null {
    for (const item of folder.items) {
      if (item.id === id) return item
    }
    for (const sub of folder.folders) {
      const found = findSessionInTree(sub, id)
      if (found) return found
    }
    return null
  }

  function getTabCount(sessionId: string): number {
    return tabs.filter((t) => t.sessionId === sessionId && !t.isLocal).length
  }

  interface FlatItem {
    id: string
    name: string
    host: string
    username: string
    port: number
    isFolder?: boolean
  }

  function flattenSessions(
    folder: SessionFolder,
    depth: number = 0,
  ): Array<{ item: FlatItem; depth: number }> {
    const result: Array<{ item: FlatItem; depth: number }> = []
    for (const item of folder.items || []) {
      result.push({ item, depth })
    }
    for (const sub of folder.folders || []) {
      result.push({ item: { ...sub, isFolder: true, host: '', username: '', port: 0 }, depth })
      result.push(...flattenSessions(sub, depth + 1))
    }
    return result
  }
</script>

<div class="sidebar-content">
  {#if connectedSessionIds.length > 0}
    <div class="section">
      <h2 class="section-title">{t('sidebar.activeSessions')}</h2>
      {#each connectedSessionIds as sessionId}
        <div class="active-session" class:selected={selectedSessionId === sessionId}>
          <button class="active-session-info" onclick={() => { selectedSessionId = sessionId }}>
            <span class="active-dot"></span>
            <span class="active-name">{getSessionName(sessionId)}</span>
            <span class="active-tabs">{getTabCount(sessionId)}</span>
          </button>
          <button class="disconnect-btn" onclick={() => onDisconnect(sessionId)} title={t('sidebar.disconnect')} aria-label="Disconnect session">
            &#x2715;
          </button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="section">
    <div class="header">
      <h2>{t('sidebar.sessions')}</h2>
      <div class="header-actions">
        <button class="import-btn" onclick={() => (showImport = true)} title={t('sidebar.importSshConfig')} aria-label="Import SSH config">&#8595;</button>
        <button class="add-btn" onclick={onNewSession} aria-label="Add session">+</button>
      </div>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    {#if sessions}
      {@const flatItems = flattenSessions(sessions)}
      {#if flatItems.length === 0}
        <div class="empty">
          <p>{t('sidebar.noSessions')}</p>
          <button class="empty-add" onclick={onNewSession}>{t('sidebar.addSession')}</button>
        </div>
      {:else}
        {#each flatItems as { item, depth }}
          {#if item.isFolder}
            <div class="folder" style:padding-left="{depth * 12 + 12}px">
              <span class="folder-icon">{'{'}{'}'}</span>
              <span class="folder-name">{item.name}</span>
            </div>
          {:else}
            <div class="session-item" style:padding-left="{depth * 12 + 12}px" class:selected={selectedSessionId === item.id} class:connected={connectedSessionIds.includes(item.id)}>
              <button class="session-info" onclick={() => { selectedSessionId = item.id; connect(item.id, item.name || item.host) }}>
                <span class="session-name">{item.name || item.host}</span>
                <span class="session-host">{item.username}@{item.host}:{item.port}</span>
                {#if connectingId === item.id}
                  <span class="connecting">{t('sidebar.connecting')}</span>
                {/if}
              </button>
              <button class="delete-btn" onclick={(e: Event) => { e.stopPropagation(); deleteSession(item.id) }} aria-label="Delete session">
                x
              </button>
            </div>
          {/if}
        {/each}
      {/if}
    {:else}
      <div class="loading">{t('sidebar.loading')}</div>
    {/if}
  </div>
</div>

{#if showImport}
  <ImportDialog onClose={async () => { showImport = false; await loadSessions() }} />
{/if}

{#if passwordDialogSession}
  <PasswordDialog
    sessionName={passwordDialogSession.name}
    onsubmit={handlePasswordSubmit}
    oncancel={handlePasswordCancel}
  />
{/if}

<style>
  .sidebar-content {
    padding: 8px;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  .section {
    margin-bottom: 8px;
  }

  .section-title {
    margin: 0 0 6px 0;
    font-size: 11px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 1px;
    font-weight: 600;
    padding: 0 4px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .header h2 {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .header-actions {
    display: flex;
    gap: 4px;
  }

  .add-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    width: 24px;
    height: 24px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .import-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    width: 24px;
    height: 24px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .import-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .add-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .active-session {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-radius: 6px;
    margin-bottom: 2px;
    transition: background 0.15s;
  }

  .active-session:hover {
    background: var(--bg-hover);
  }

  .active-session.selected {
    background: var(--bg-active);
  }

  .active-session-info {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    cursor: pointer;
    flex: 1;
    min-width: 0;
  }

  .active-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success);
    flex-shrink: 0;
  }

  .active-name {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .active-tabs {
    font-size: 10px;
    color: var(--text-tertiary);
    background: var(--bg-hover);
    padding: 1px 6px;
    border-radius: 10px;
    flex-shrink: 0;
  }

  .disconnect-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
  }

  .active-session:hover .disconnect-btn {
    opacity: 1;
  }

  .disconnect-btn:hover {
    color: var(--error);
    background: var(--error-bg);
  }

  .session-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-radius: 6px;
    margin-bottom: 2px;
    transition: background 0.15s;
  }

  .session-item:hover {
    background: var(--bg-hover);
  }

  .session-item.selected {
    background: var(--bg-active);
  }

  .session-item.connected {
    opacity: 0.5;
  }

  .session-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 6px 8px;
    cursor: pointer;
    flex: 1;
  }

  .session-name {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .session-host {
    color: var(--text-tertiary);
    font-size: 11px;
  }

  .connecting {
    color: var(--accent-text);
    font-size: 11px;
  }

  .delete-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
  }

  .session-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    color: var(--error);
    background: var(--error-bg);
  }

  .folder {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
  }

  .folder-icon {
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .folder-name {
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .empty {
    text-align: center;
    padding: 24px 16px;
    color: var(--text-tertiary);
  }

  .empty p {
    margin: 0 0 12px 0;
    font-size: 13px;
  }

  .empty-add {
    background: var(--accent);
    border: none;
    color: var(--text-inverse);
    padding: 6px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
  }

  .loading {
    color: var(--text-tertiary);
    font-size: 13px;
    padding: 12px;
    text-align: center;
  }

  .error {
    background: var(--error-bg);
    color: var(--error);
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
    margin-bottom: 8px;
  }
</style>
