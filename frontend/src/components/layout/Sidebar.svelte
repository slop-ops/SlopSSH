<script lang="ts">
  import * as api from '$lib/api/invoke'

  let {
    onConnect,
    onNewSession,
  }: {
    onConnect: (id: string, name: string) => void
    onNewSession: () => void
  } = $props()

  let sessions = $state<any>(null)
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

  function flattenSessions(
    folder: any,
    depth: number = 0,
  ): Array<{ item: any; depth: number }> {
    const result: Array<{ item: any; depth: number }> = []
    for (const item of folder.items || []) {
      result.push({ item, depth })
    }
    for (const sub of folder.folders || []) {
      result.push({ item: { isFolder: true, ...sub }, depth })
      result.push(...flattenSessions(sub, depth + 1))
    }
    return result
  }
</script>

<div class="sidebar-content">
  <div class="header">
    <h2>Sessions</h2>
    <button class="add-btn" onclick={onNewSession}>+</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if sessions}
    {@const flatItems = flattenSessions(sessions)}
    {#if flatItems.length === 0}
      <div class="empty">
        <p>No sessions configured</p>
        <button class="empty-add" onclick={onNewSession}>+ Add Session</button>
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
                <span class="connecting">Connecting...</span>
              {/if}
            </button>
            <button class="delete-btn" onclick={(e: Event) => { e.stopPropagation(); deleteSession(item.id) }}>
              x
            </button>
          </div>
        {/if}
      {/each}
    {/if}
  {:else}
    <div class="loading">Loading...</div>
  {/if}

  <div class="password-section">
    <label for="password">Password</label>
    <input
      id="password"
      type="password"
      bind:value={password}
      placeholder="Enter password..."
      onkeydown={(e) => {
        if (e.key === 'Enter') {
          const firstItem = sessions?.items?.[0]
          if (firstItem) connect(firstItem.id, firstItem.name || firstItem.host)
        }
      }}
    />
  </div>
</div>

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
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .add-btn {
    background: transparent;
    border: 1px solid #2e303a;
    color: #9ca3af;
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

  .add-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
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
    background: #2a2a3e;
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
    color: #e0e0e0;
    font-size: 13px;
    font-weight: 500;
  }

  .session-host {
    color: #6b7280;
    font-size: 11px;
  }

  .connecting {
    color: #4a90d9;
    font-size: 11px;
  }

  .delete-btn {
    background: none;
    border: none;
    color: #6b7280;
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
    color: #e06c75;
    background: #e06c7522;
  }

  .folder {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
  }

  .folder-icon {
    color: #6b7280;
    font-size: 12px;
  }

  .folder-name {
    color: #9ca3af;
    font-size: 12px;
    font-weight: 600;
  }

  .empty {
    text-align: center;
    padding: 24px 16px;
    color: #6b7280;
  }

  .empty p {
    margin: 0 0 12px 0;
    font-size: 13px;
  }

  .empty-add {
    background: #4a90d9;
    border: none;
    color: #fff;
    padding: 6px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
  }

  .loading {
    color: #6b7280;
    font-size: 13px;
    padding: 12px;
    text-align: center;
  }

  .error {
    background: #e06c7522;
    color: #e06c75;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
    margin-bottom: 8px;
  }

  .password-section {
    margin-top: auto;
    padding-top: 12px;
    border-top: 1px solid #2e303a;
  }

  .password-section label {
    display: block;
    font-size: 11px;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .password-section input {
    width: 100%;
    box-sizing: border-box;
    background: #1a1a2e;
    border: 1px solid #2e303a;
    border-radius: 6px;
    padding: 6px 10px;
    color: #e0e0e0;
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.2s;
  }

  .password-section input:focus {
    border-color: #4a90d9;
  }
</style>
