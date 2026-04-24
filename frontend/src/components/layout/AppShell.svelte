<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import TerminalHolder from '../terminal/TerminalHolder.svelte'
  import FileBrowser from '../files/FileBrowser.svelte'
  import TransferQueue from '../files/TransferQueue.svelte'
  import ToolsPanel from '../tools/ToolsPanel.svelte'
  import NewSessionDialog from '../session/NewSessionDialog.svelte'
  import SettingsDialog from '../settings/SettingsDialog.svelte'
  import { getTheme, toggleTheme } from '$lib/stores/theme'
  import { registerHandler, setEnabled as setShortcutsEnabled } from '$lib/utils/shortcuts'
  import { listen } from '@tauri-apps/api/event'

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
  let theme = $state(getTheme())

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

  function handleToggleTheme() {
    toggleTheme()
    theme = getTheme()
  }

  function handleShortcutAction(action: string) {
    switch (action) {
      case 'new-tab':
        if (activeSessionId) {
          handleConnect(activeSessionId, `Tab ${tabs.length + 1}`)
        }
        break
      case 'close-tab':
        if (activeTabId) {
          closeTab(activeTabId)
        }
        break
      case 'next-tab': {
        const idx = tabs.findIndex((t) => t.id === activeTabId)
        if (idx >= 0 && idx < tabs.length - 1) {
          activeTabId = tabs[idx + 1].id
        }
        break
      }
      case 'prev-tab': {
        const idx = tabs.findIndex((t) => t.id === activeTabId)
        if (idx > 0) {
          activeTabId = tabs[idx - 1].id
        }
        break
      }
      case 'toggle-sidebar':
        toggleSidebar()
        break
      case 'open-settings':
        showSettings = !showSettings
        break
      case 'new-session':
        showNewSession = true
        break
      case 'toggle-files':
        if (activeSessionId) activeView = activeView === 'files' ? 'terminal' : 'files'
        break
      case 'toggle-tools':
        if (activeSessionId) activeView = activeView === 'tools' ? 'terminal' : 'tools'
        break
      case 'escape':
        if (showNewSession) showNewSession = false
        else if (showSettings) showSettings = false
        break
      case 'refresh':
        break
      case 'toggle-fullscreen':
        if (document.fullscreenElement) {
          document.exitFullscreen()
        } else {
          document.documentElement.requestFullscreen()
        }
        break
    }
  }

  function closeTab(tabId: string) {
    tabs = tabs.filter((t) => t.id !== tabId)
    if (activeTabId === tabId) {
      activeTabId = tabs.length > 0 ? tabs[tabs.length - 1].id : ''
    }
    if (tabs.length === 0) {
      activeSessionId = ''
    }
  }

  $effect(() => {
    if (tabs.length > 0) {
      const activeTab = tabs.find((t) => t.id === activeTabId)
      if (activeTab) {
        activeSessionId = activeTab.sessionId
      }
    }
  })

  $effect(() => {
    const unregister = registerHandler(handleShortcutAction)
    return unregister
  })

  $effect(() => {
    setShortcutsEnabled(!showNewSession && !showSettings)
  })

  $effect(() => {
    const unlisten = listen<string>('menu-event', (event) => {
      switch (event.payload) {
        case 'new_session':
          showNewSession = true
          break
        case 'import_sessions':
          break
        case 'close_tab':
          if (activeTabId) closeTab(activeTabId)
          break
        case 'quit':
          break
        case 'copy':
          document.execCommand('copy')
          break
        case 'paste':
          document.execCommand('paste')
          break
        case 'select_all':
          document.execCommand('selectAll')
          break
        case 'settings':
          showSettings = !showSettings
          break
        case 'connect':
          break
        case 'disconnect':
          break
        case 'duplicate':
          break
        case 'delete_session':
          break
        case 'toggle_sidebar':
          toggleSidebar()
          break
        case 'local_terminal':
          break
        case 'zoom_in':
          document.documentElement.style.fontSize = `${parseFloat(getComputedStyle(document.documentElement).fontSize) + 1}px`
          break
        case 'zoom_out':
          document.documentElement.style.fontSize = `${Math.max(8, parseFloat(getComputedStyle(document.documentElement).fontSize) - 1)}px`
          break
        case 'zoom_reset':
          document.documentElement.style.fontSize = ''
          break
        case 'fullscreen':
          if (document.fullscreenElement) {
            document.exitFullscreen()
          } else {
            document.documentElement.requestFullscreen()
          }
          break
        case 'file_browser':
          if (activeSessionId) activeView = 'files'
          break
        case 'process_viewer':
        case 'log_viewer':
        case 'disk_analyzer':
        case 'search':
        case 'port_forwarding':
        case 'port_viewer':
        case 'key_manager':
          if (activeSessionId) activeView = 'tools'
          break
        case 'about':
          break
        case 'check_updates':
          break
      }
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  })
</script>

<div class="app-shell" role="application" aria-label="Muon SSH">
  {#if showSidebar}
    <aside class="sidebar" role="navigation" aria-label="Session list">
      <Sidebar onConnect={handleConnect} onNewSession={() => (showNewSession = true)} />
    </aside>
  {/if}
  <main class="content" role="main">
    <div class="toolbar" role="toolbar" aria-label="Main toolbar">
      <button class="toolbar-btn" onclick={toggleSidebar} aria-label={showSidebar ? 'Hide sidebar' : 'Show sidebar'} aria-expanded={showSidebar}>
        {showSidebar ? '<' : '>'}
      </button>
      <button class="toolbar-btn" onclick={() => (showNewSession = true)} aria-label="New session">+ New Session</button>
      {#if activeSessionId}
        <div class="toolbar-separator" role="separator"></div>
        <button class="toolbar-btn" class:active={activeView === 'terminal'} onclick={() => (activeView = 'terminal')} aria-pressed={activeView === 'terminal'}>Terminal</button>
        <button class="toolbar-btn" class:active={activeView === 'files'} onclick={() => (activeView = 'files')} aria-pressed={activeView === 'files'}>Files</button>
        <button class="toolbar-btn" class:active={activeView === 'tools'} onclick={() => (activeView = 'tools')} aria-pressed={activeView === 'tools'}>Tools</button>
      {/if}
      <div class="toolbar-spacer"></div>
      <button class="toolbar-btn theme-toggle" onclick={handleToggleTheme} title="Toggle theme" aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}>
        {theme === 'dark' ? '&#9728;' : '&#9790;'}
      </button>
      <button class="toolbar-btn" onclick={() => (showSettings = true)} aria-label="Open settings">Settings</button>
    </div>

    {#if activeSessionId}
      <div class="main-views">
        <div class="view" class:hidden={activeView !== 'terminal'} role="tabpanel" aria-label="Terminal">
          <TerminalHolder bind:tabs bind:activeTabId />
        </div>
        <div class="view" class:hidden={activeView !== 'files'} role="tabpanel" aria-label="File browser">
          <div class="files-layout">
            <FileBrowser sessionId={activeSessionId} />
            <TransferQueue />
          </div>
        </div>
        <div class="view" class:hidden={activeView !== 'tools'} role="tabpanel" aria-label="Tools">
          <ToolsPanel sessionId={activeSessionId} />
        </div>
      </div>
    {:else}
      <div class="empty-state" role="status">
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
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .sidebar {
    width: 260px;
    min-width: 200px;
    border-right: 1px solid var(--border-primary);
    overflow-y: auto;
    background: var(--bg-secondary);
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
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-primary);
    flex-shrink: 0;
    align-items: center;
  }

  .toolbar-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    transition: background 0.15s, color 0.15s;
  }

  .toolbar-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .toolbar-btn.active {
    background: var(--accent-bg);
    border-color: var(--accent);
    color: var(--accent-text);
  }

  .theme-toggle {
    font-size: 14px;
    padding: 4px 8px;
  }

  .toolbar-separator {
    width: 1px;
    height: 20px;
    background: var(--border-primary);
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
    color: var(--text-secondary);
    gap: 8px;
  }

  .empty-state p {
    margin: 0;
    font-size: 14px;
  }

  .hint {
    font-size: 12px !important;
    color: var(--text-tertiary);
  }
</style>
