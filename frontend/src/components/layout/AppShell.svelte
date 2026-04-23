<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import TerminalHolder from '../terminal/TerminalHolder.svelte'
  import NewSessionDialog from '../session/NewSessionDialog.svelte'

  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
  }

  let showSidebar = $state(true)
  let tabs: Tab[] = $state([])
  let activeTabId = $state('')
  let showNewSession = $state(false)

  function handleConnect(sessionId: string, name: string) {
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    tabs = [...tabs, { id: tabId, sessionId, channelId, title: name }]
    activeTabId = tabId
  }

  function toggleSidebar() {
    showSidebar = !showSidebar
  }
</script>

<div class="app-shell">
  {#if showSidebar}
    <aside class="sidebar">
      <Sidebar onConnect={handleConnect} onNewSession={() => (showNewSession = true)} />
    </aside>
  {/if}
  <main class="content">
    <div class="toolbar">
      <button class="toolbar-btn" onclick={toggleSidebar}>
        {showSidebar ? '<' : '>'}
      </button>
      <button class="toolbar-btn" onclick={() => (showNewSession = true)}>+ New Session</button>
    </div>
    <TerminalHolder bind:tabs bind:activeTabId />
  </main>
</div>

{#if showNewSession}
  <NewSessionDialog onclose={() => (showNewSession = false)} />
{/if}

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
    background: #1a1a2e;
    color: #e0e0e0;
  }

  .sidebar {
    width: 260px;
    min-width: 200px;
    border-right: 1px solid #2e303a;
    overflow-y: auto;
    background: #16171d;
    flex-shrink: 0;
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    gap: 4px;
    padding: 4px 8px;
    background: #16171d;
    border-bottom: 1px solid #2e303a;
    flex-shrink: 0;
  }

  .toolbar-btn {
    background: transparent;
    border: 1px solid #2e303a;
    color: #9ca3af;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    transition: background 0.15s, color 0.15s;
  }

  .toolbar-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }
</style>
