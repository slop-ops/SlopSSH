<script lang="ts">
  import * as api from '$lib/api/invoke'
  import ImportDialog from '$components/session/ImportDialog.svelte'
  import { t } from '$lib/utils/i18n'
  import type { SessionFolder, SessionInfo } from '$lib/types'

  let {
    onConnect,
    onNewSession,
    showImport = $bindable(false),
    selectedSessionId = $bindable(''),
    sessions: sessions = $bindable<SessionFolder | null>(null),
  }: {
    onConnect: (id: string, name: string) => void
    onNewSession: () => void
    showImport?: boolean
    selectedSessionId?: string
    sessions?: SessionFolder | null
  } = $props()

  let password = $state('')
  let connectingId = $state('')
  let error = $state('')
  let editingId = $state<string | null>(null)

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
      await api.sshConnect(id, password || undefined)
      password = ''
      onConnect?.(id, name)
    } catch (e) {
      error = String(e)
    } finally {
      connectingId = ''
    }
  }

  async function deleteSession(id: string) {
    try {
      await api.deleteSession(id)
      await loadSessions()
    } catch (e) {
      error = String(e)
    }
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
          <div class="session-item" style:padding-left="{depth * 12 + 12}px">
            <button class="session-info" onclick={() => connect(item.id, item.name || item.host)}>
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

  <div class="password-section">
    <label for="password">{t('sidebar.password')}</label>
    <input
      id="password"
      type="password"
      bind:value={password}
      placeholder={t('sidebar.enterPassword')}
      onkeydown={(e) => {
        if (e.key === 'Enter') {
          const firstItem = sessions?.items?.[0]
          if (firstItem) connect(firstItem.id, firstItem.name || firstItem.host)
        }
      }}
    />
  </div>
</div>

{#if showImport}
  <ImportDialog onClose={async () => { showImport = false; await loadSessions() }} />
{/if}

<style>
  .sidebar-content {
    padding: 8px;
    display: flex;
    flex-direction: column;
    height: 100%;
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

  .password-section {
    margin-top: auto;
    padding-top: 12px;
    border-top: 1px solid var(--border-primary);
  }

  .password-section label {
    display: block;
    font-size: 11px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .password-section input {
    width: 100%;
    box-sizing: border-box;
    background: var(--bg-input);
    border: 1px solid var(--border-primary);
    border-radius: 6px;
    padding: 6px 10px;
    color: var(--text-primary);
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.2s;
  }

  .password-section input:focus {
    border-color: var(--border-active);
  }
</style>
