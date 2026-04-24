<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import TerminalHolder from '../terminal/TerminalHolder.svelte'
  import FileBrowser from '../files/FileBrowser.svelte'
  import TransferQueue from '../files/TransferQueue.svelte'
  import ToolsPanel from '../tools/ToolsPanel.svelte'
  import NewSessionDialog from '../session/NewSessionDialog.svelte'
  import SettingsDialog from '../settings/SettingsDialog.svelte'

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
  let showSettings = $state(false)
  let activeView = $state('terminal')
  let activeSessionId = $state('')

  function handleConnect(sessionId: string, name: string) {
    activeSessionId = sessionId
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    tabs = [...tabs, { id: tabId, sessionId, channelId, title: name }]
    activeTabId = tabId
  }

  function toggleSidebar() {
    showSidebar = !showSidebar
  }

  $effect(() => {
    if (tabs.length > 0) {
      const activeTab = tabs.find((t) => t.id === activeTabId)
      if (activeTab) {
        activeSessionId = activeTab.sessionId
      }
    }
  })
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
      {#if activeSessionId}
        <div class="toolbar-separator"></div>
        <button class="toolbar-btn" class:active={activeView === 'terminal'} onclick={() => (activeView = 'terminal')}>Terminal</button>
        <button class="toolbar-btn" class:active={activeView === 'files'} onclick={() => (activeView = 'files')}>Files</button>
        <button class="toolbar-btn" class:active={activeView === 'tools'} onclick={() => (activeView = 'tools')}>Tools</button>
      {/if}
      <div class="toolbar-spacer"></div>
      <button class="toolbar-btn" onclick={() => (showSettings = true)}>Settings</button>
    </div>

    {#if activeSessionId}
      <div class="main-views">
        <div class="view" class:hidden={activeView !== 'terminal'}>
          <TerminalHolder bind:tabs bind:activeTabId />
        </div>
        <div class="view" class:hidden={activeView !== 'files'}>
          <div class="files-layout">
            <FileBrowser sessionId={activeSessionId} />
            <TransferQueue />
          </div>
        </div>
        <div class="view" class:hidden={activeView !== 'tools'}>
          <ToolsPanel sessionId={activeSessionId} />
        </div>
      </div>
    {:else}
      <div class="empty-state">
        <p>No active session</p>
        <p class="hint">Connect to a session from the sidebar to get started</p>
      </div>
    {/if}
  </main>
</div>

{#if showNewSession}
  <NewSessionDialog onclose={() => (showNewSession = false)} />
{/if}

<SettingsDialog bind:open={showSettings} />

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
    align-items: center;
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

  .toolbar-btn.active {
    background: #4a90d922;
    border-color: #4a90d9;
    color: #4a90d9;
  }

  .toolbar-separator {
    width: 1px;
    height: 20px;
    background: #2e303a;
    margin: 0 4px;
  }

  .toolbar-spacer {
    flex: 1;
  }

  .main-views {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .view {
    position: absolute;
    inset: 0;
  }

  .view.hidden {
    display: none;
  }

  .files-layout {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #9ca3af;
    gap: 8px;
  }

  .empty-state p {
    margin: 0;
    font-size: 14px;
  }

  .hint {
    font-size: 12px !important;
    color: #6b7280;
  }
</style>
