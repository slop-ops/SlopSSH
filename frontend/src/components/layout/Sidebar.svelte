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
    onSessionSelect,
    showImport = $bindable(false),
    selectedSessionId = $bindable(''),
    sessions: sessions = $bindable<SessionFolder | null>(null),
    tabs = [],
    connectedSessionIds = [],
    disconnectedSessionIds = new Set(),
    collapsed = false,
  }: {
    onConnect: (id: string, name: string) => void
    onDisconnect: (sessionId: string) => void
    onNewSession: () => void
    onSessionSelect?: (sessionId: string) => void
    showImport?: boolean
    selectedSessionId?: string
    sessions?: SessionFolder | null
    tabs?: Tab[]
    connectedSessionIds?: string[]
    disconnectedSessionIds?: Set<string>
    collapsed?: boolean
  } = $props()

  let connectingId = $state('')
  let error = $state('')
  let passwordDialogSession = $state<{ id: string; name: string } | null>(null)
  let showNewFolder = $state(false)
  let newFolderName = $state('')
  let collapsedFolders = $state<Set<string>>(new Set())
  let contextMenu = $state<{ x: number; y: number; sessionId: string } | null>(null)
  let dragOverFolderId = $state<string | null>(null)

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

  async function createFolder() {
    if (!newFolderName.trim()) return
    try {
      await api.createFolder(newFolderName.trim())
      newFolderName = ''
      showNewFolder = false
      await loadSessions()
    } catch (e) {
      error = String(e)
    }
  }

  function toggleFolder(folderId: string) {
    if (collapsedFolders.has(folderId)) {
      collapsedFolders.delete(folderId)
    } else {
      collapsedFolders.add(folderId)
    }
    collapsedFolders = collapsedFolders
  }

  async function moveSessionToFolder(sessionId: string, targetFolderId: string | null) {
    try {
      await api.moveSession(sessionId, targetFolderId ?? undefined)
      await loadSessions()
    } catch (e) {
      error = String(e)
    }
    contextMenu = null
  }

  function handleContextMenu(e: MouseEvent, sessionId: string) {
    e.preventDefault()
    contextMenu = { x: e.clientX, y: e.clientY, sessionId }
  }

  function closeContextMenu() {
    contextMenu = null
  }

  function getAllFolders(folder: SessionFolder, prefix = ''): Array<{ id: string; name: string }> {
    const result: Array<{ id: string; name: string }> = []
    for (const sub of folder.folders || []) {
      const fullName = prefix ? `${prefix} / ${sub.name}` : sub.name
      result.push({ id: sub.id, name: fullName })
      result.push(...getAllFolders(sub, fullName))
    }
    return result
  }

  function handleDragStart(e: DragEvent, sessionId: string) {
    if (e.dataTransfer) {
      e.dataTransfer.setData('text/plain', sessionId)
      e.dataTransfer.effectAllowed = 'move'
    }
  }

  function handleDragOver(e: DragEvent, folderId: string | null) {
    e.preventDefault()
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move'
    }
    dragOverFolderId = folderId
  }

  function handleDragLeave() {
    dragOverFolderId = null
  }

  async function handleDrop(e: DragEvent, targetFolderId: string | null) {
    e.preventDefault()
    dragOverFolderId = null
    const sessionId = e.dataTransfer?.getData('text/plain')
    if (sessionId) {
      await moveSessionToFolder(sessionId, targetFolderId)
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

  function getSessionInitials(sessionId: string): string {
    const name = getSessionName(sessionId)
    const parts = name.split(/[\s._-]+/)
    if (parts.length >= 2) {
      return (parts[0][0] + parts[1][0]).toUpperCase()
    }
    return name.slice(0, 2).toUpperCase()
  }

  interface FlatItem {
    id: string
    name: string
    host: string
    username: string
    port: number
    isFolder?: boolean
    folderId?: string
  }

  function flattenSessions(
    folder: SessionFolder,
    depth: number = 0,
    parentFolderId: string | null = null,
  ): Array<{ item: FlatItem; depth: number }> {
    const result: Array<{ item: FlatItem; depth: number }> = []
    for (const item of folder.items || []) {
      result.push({ item: { ...item, folderId: parentFolderId ?? undefined }, depth })
    }
    for (const sub of folder.folders || []) {
      result.push({ item: { ...sub, isFolder: true, host: '', username: '', port: 0, folderId: sub.id }, depth })
      if (!collapsedFolders.has(sub.id)) {
        result.push(...flattenSessions(sub, depth + 1, sub.id))
      }
    }
    return result
  }

  $effect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (contextMenu && !(e.target as HTMLElement).closest('.context-menu')) {
        closeContextMenu()
      }
    }
    window.addEventListener('click', handleClickOutside)
    return () => window.removeEventListener('click', handleClickOutside)
  })
</script>

<div class="sidebar-content" class:collapsed>
  {#if collapsed}
    <div class="collapsed-items">
      {#if connectedSessionIds.length > 0}
        {#each connectedSessionIds as sessionId}
          <button class="collapsed-item active" class:selected={selectedSessionId === sessionId} onclick={() => { selectedSessionId = sessionId; onSessionSelect?.(sessionId) }} title={getSessionName(sessionId)}>
            <span class="collapsed-dot"></span>
          </button>
        {/each}
        <div class="collapsed-divider"></div>
      {/if}
      <button class="collapsed-item" onclick={onNewSession} title={t('sidebar.addSession')}>
        <span class="collapsed-icon">+</span>
      </button>
    </div>
  {:else}
    {#if connectedSessionIds.length > 0}
    <div class="section">
      <h2 class="section-title">{t('sidebar.activeSessions')}</h2>
      {#each connectedSessionIds as sessionId}
        <div class="active-session" class:selected={selectedSessionId === sessionId} class:disconnected={disconnectedSessionIds.has(sessionId)}>
          <button class="active-session-info" onclick={() => { selectedSessionId = sessionId; onSessionSelect?.(sessionId) }}>
            <span class="active-dot" class:error={disconnectedSessionIds.has(sessionId)}></span>
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
          <button class="folder-btn" onclick={() => { showNewFolder = !showNewFolder }} title={t('sidebar.newFolder')} aria-label="New folder">&#128193;</button>
          <button class="import-btn" onclick={() => (showImport = true)} title={t('sidebar.importSshConfig')} aria-label="Import SSH config">&#8595;</button>
          <button class="add-btn" onclick={onNewSession} aria-label="Add session">+</button>
        </div>
      </div>

      {#if showNewFolder}
        <div class="new-folder-row">
          <input
            type="text"
            bind:value={newFolderName}
            placeholder={t('sidebar.folderNamePlaceholder')}
            class="new-folder-input"
            onkeydown={(e) => { if (e.key === 'Enter') createFolder(); if (e.key === 'Escape') { showNewFolder = false; newFolderName = '' } }}
          />
          <button class="new-folder-confirm" onclick={createFolder} disabled={!newFolderName.trim()}>&#10003;</button>
          <button class="new-folder-cancel" onclick={() => { showNewFolder = false; newFolderName = '' }}>&#10005;</button>
        </div>
      {/if}

      {#if error}
        <div class="error">{error}</div>
      {/if}

      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="drop-zone"
        class:drag-over={dragOverFolderId === 'root'}
        ondragover={(e) => handleDragOver(e, 'root')}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDrop(e, null)}
      >
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
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="folder"
                  class:collapsed={collapsedFolders.has(item.id)}
                  class:drag-over={dragOverFolderId === item.id}
                  style:padding-left="{depth * 12 + 12}px"
                  onclick={() => toggleFolder(item.id)}
                  onkeydown={(e) => { if (e.key === 'Enter') toggleFolder(item.id) }}
                  role="button"
                  tabindex={0}
                  ondragover={(e) => { e.stopPropagation(); handleDragOver(e, item.id) }}
                  ondragleave={handleDragLeave}
                  ondrop={(e) => { e.stopPropagation(); handleDrop(e, item.id) }}
                >
                  <span class="folder-chevron">{collapsedFolders.has(item.id) ? '\u25B6' : '\u25BC'}</span>
                  <span class="folder-icon">{'\uD83D\uDCC1'}</span>
                  <span class="folder-name">{item.name}</span>
                </div>
              {:else}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="session-item"
                  style:padding-left="{depth * 12 + 12}px"
                  class:selected={selectedSessionId === item.id}
                  class:connected={connectedSessionIds.includes(item.id)}
                  draggable="true"
                  ondragstart={(e) => handleDragStart(e, item.id)}
                  oncontextmenu={(e) => handleContextMenu(e, item.id)}
                >
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
  {/if}
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

{#if contextMenu && sessions}
  {@const allFolders = getAllFolders(sessions)}
  <div class="context-menu" style:left="{contextMenu.x}px" style:top="{contextMenu.y}px">
    <div class="context-menu-title">{t('sidebar.moveToFolder')}</div>
    <button class="context-menu-item" onclick={() => moveSessionToFolder(contextMenu.sessionId, null)}>
      {t('sidebar.rootLevel')}
    </button>
    {#each allFolders as folder}
      <button class="context-menu-item" onclick={() => moveSessionToFolder(contextMenu.sessionId, folder.id)}>
        {folder.name}
      </button>
    {/each}
  </div>
{/if}

<style>
  .sidebar-content {
    padding: 8px;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .sidebar-content.collapsed {
    padding: 8px 4px;
    align-items: center;
  }

  .collapsed-items {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding-top: 4px;
  }

  .collapsed-item {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background: transparent;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
    position: relative;
  }

  .collapsed-item:hover {
    background: var(--bg-hover);
  }

  .collapsed-item.selected {
    background: var(--bg-active);
  }

  .collapsed-item.active .collapsed-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--success);
  }

  .collapsed-icon {
    color: var(--text-secondary);
    font-size: 16px;
    font-weight: 600;
  }

  .collapsed-divider {
    width: 24px;
    height: 1px;
    background: var(--border-primary);
    margin: 4px 0;
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

  .add-btn,
  .import-btn,
  .folder-btn {
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

  .import-btn:hover,
  .add-btn:hover,
  .folder-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .new-folder-row {
    display: flex;
    gap: 4px;
    margin-bottom: 8px;
    padding: 0 4px;
  }

  .new-folder-input {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border-active);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
  }

  .new-folder-confirm,
  .new-folder-cancel {
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

  .new-folder-confirm:hover {
    background: var(--success);
    color: var(--text-inverse);
    border-color: var(--success);
  }

  .new-folder-cancel:hover {
    background: var(--error);
    color: var(--text-inverse);
    border-color: var(--error);
  }

  .drop-zone {
    min-height: 20px;
    transition: background 0.15s;
  }

  .drop-zone.drag-over {
    background: var(--accent-bg);
    border-radius: 4px;
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

  .active-dot.error {
    background: var(--error);
  }

  .active-session.disconnected {
    opacity: 0.6;
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
    color: #e06c75;
    cursor: pointer;
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 4px;
    opacity: 0.6;
    transition: opacity 0.15s, color 0.15s, background 0.15s;
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
    cursor: grab;
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
    color: #e06c75;
    cursor: pointer;
    font-size: 11px;
    padding: 4px 8px;
    border-radius: 4px;
    opacity: 0.6;
    transition: opacity 0.15s, color 0.15s, background 0.15s;
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
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s;
    user-select: none;
  }

  .folder:hover {
    background: var(--bg-hover);
  }

  .folder.drag-over {
    background: var(--accent-bg);
    outline: 1px dashed var(--accent);
  }

  .folder-chevron {
    color: var(--text-tertiary);
    font-size: 9px;
    width: 12px;
    text-align: center;
  }

  .folder-icon {
    font-size: 13px;
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

  .context-menu {
    position: fixed;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: 6px;
    padding: 4px 0;
    min-width: 160px;
    max-height: 300px;
    overflow-y: auto;
    z-index: 1000;
    box-shadow: var(--shadow-lg);
  }

  .context-menu-title {
    padding: 4px 12px;
    font-size: 11px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
    border-bottom: 1px solid var(--border-primary);
    margin-bottom: 4px;
  }

  .context-menu-item {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text-primary);
    padding: 6px 12px;
    font-size: 12px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.15s;
  }

  .context-menu-item:hover {
    background: var(--bg-hover);
  }
</style>
