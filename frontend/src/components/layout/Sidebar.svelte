<script lang="ts">
  import * as api from '$lib/api/invoke'

  let { onConnect }: { onConnect: (id: string) => void } = $props()

  let sessions = $state<any>(null)
  let password = $state('')
  let connectingId = $state('')
  let error = $state('')

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

  async function connect(id: string) {
    if (password) {
      connectingId = id
      error = ''
      try {
        await api.sshConnect(id, password)
        password = ''
        onConnect?.(id)
      } catch (e) {
        error = String(e)
      } finally {
        connectingId = ''
      }
    }
  }
</script>

<div class="sidebar-content">
  <div class="header">
    <h2>Sessions</h2>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if sessions}
    {#each sessions.items || [] as session}
      <div class="session-item" onclick={() => connect(session.id)}>
        <span class="session-name">{session.name || session.host}</span>
        <span class="session-host">{session.username}@{session.host}:{session.port}</span>
        {#if connectingId === session.id}
          <span class="connecting">Connecting...</span>
        {/if}
      </div>
    {/each}

    {#if (!sessions.items || sessions.items.length === 0) && (!sessions.folders || sessions.folders.length === 0)}
      <div class="empty">No sessions configured</div>
    {/if}

    {#each sessions.folders || [] as folder}
      <div class="folder">
        <span class="folder-name">{folder.name}</span>
        {#each folder.items || [] as session}
          <div class="session-item nested" onclick={() => connect(session.id)}>
            <span class="session-name">{session.name || session.host}</span>
            <span class="session-host">{session.username}@{session.host}:{session.port}</span>
          </div>
        {/each}
      </div>
    {/each}
  {:else}
    <div class="loading">Loading...</div>
  {/if}

  <div class="password-section">
    <label for="password">Password:</label>
    <input
      id="password"
      type="password"
      bind:value={password}
      placeholder="Enter password..."
    />
  </div>
</div>

<style>
  .sidebar-content {
    padding: 8px;
  }

  .header h2 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #e0e0e0;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .session-item {
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-bottom: 4px;
    transition: background 0.15s;
  }

  .session-item:hover {
    background: #2a2a3e;
  }

  .session-item.nested {
    margin-left: 16px;
  }

  .session-name {
    color: #e0e0e0;
    font-size: 13px;
    font-weight: 500;
  }

  .session-host {
    color: #9ca3af;
    font-size: 11px;
  }

  .connecting {
    color: #4a90d9;
    font-size: 11px;
  }

  .folder {
    margin-bottom: 8px;
  }

  .folder-name {
    color: #9ca3af;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 4px 12px;
    display: block;
  }

  .empty, .loading {
    color: #9ca3af;
    font-size: 13px;
    padding: 12px;
    text-align: center;
  }

  .error {
    color: #e06c75;
    font-size: 12px;
    padding: 8px;
    background: rgba(224, 108, 117, 0.1);
    border-radius: 4px;
    margin-bottom: 8px;
  }

  .password-section {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid #2e303a;
  }

  .password-section label {
    display: block;
    font-size: 11px;
    color: #9ca3af;
    margin-bottom: 4px;
  }

  .password-section input {
    width: 100%;
    box-sizing: border-box;
    padding: 6px 8px;
    border-radius: 4px;
    border: 1px solid #2e303a;
    background: #2a2a3e;
    color: #e0e0e0;
    font-size: 13px;
  }

  .password-section input:focus {
    outline: none;
    border-color: #4a90d9;
  }
</style>
