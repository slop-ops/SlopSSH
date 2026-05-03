<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import TerminalHolder from '../terminal/TerminalHolder.svelte'
  import FileBrowser from '../files/FileBrowser.svelte'
  import TransferQueue from '../files/TransferQueue.svelte'
  import ToolsPanel from '../tools/ToolsPanel.svelte'
  import NewSessionDialog from '../session/NewSessionDialog.svelte'
  import SettingsDialog from '../settings/SettingsDialog.svelte'
  import { getTheme, toggleTheme } from '$lib/stores/theme'
  import { t } from '$lib/utils/i18n'
  import { registerHandler, setEnabled as setShortcutsEnabled } from '$lib/utils/shortcuts'
  import { listen } from '@tauri-apps/api/event'
  import * as api from '$lib/api/invoke'
  import type { TabState, SavedTab, SessionInfo, SessionFolder } from '$lib/types'

  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
    isLocal?: boolean
  }

  let showSidebar = $state(true)
  let tabs: Tab[] = $state([])
  let activeTabId = $state('')
  let showNewSession = $state(false)
  let showSettings = $state(false)
  let activeView = $state('terminal')
  let activeSessionId = $state('')
  let theme = $state(getTheme())
  let restoring = $state(true)
  let showAbout = $state(false)
  let showImport = $state(false)
  let updateStatus = $state('')
  let appVersion = $state('')
  let selectedSessionId = $state('')
  let sidebarSessions = $state<SessionFolder | null>(null)

  async function restoreTabState() {
    try {
      const state = await api.loadTabState()
      if (state?.tabs?.length) {
        const restored: Tab[] = state.tabs.map((st: SavedTab) => ({
          id: crypto.randomUUID(),
          sessionId: st.session_id,
          channelId: st.channel_id,
          title: st.title,
          isLocal: st.is_local,
        }))
        tabs = restored
        if (restored.length > 0) {
          activeTabId = restored[0].id
          activeSessionId = restored[0].sessionId
        }
      }
    } catch {
      // ignore restore errors
    }
    restoring = false
  }

  restoreTabState()

  async function persistTabState() {
    if (restoring) return
    try {
      const savedTabs: SavedTab[] = tabs.map((tab) => ({
        session_id: tab.sessionId,
        channel_id: tab.channelId,
        title: tab.title,
        is_local: tab.isLocal ?? false,
      }))
      await api.saveTabState({ tabs: savedTabs, active_tab_id: activeTabId || null })
    } catch {
      // ignore save errors
    }
  }

  function handleConnect(sessionId: string, name: string) {
    activeSessionId = sessionId
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    tabs = [...tabs, { id: tabId, sessionId, channelId, title: name }]
    activeTabId = tabId
    api.updateTrayTooltip()
  }

  function toggleSidebar() {
    showSidebar = !showSidebar
  }

  function handleToggleTheme() {
    toggleTheme()
    theme = getTheme()
  }

  function openLocalTerminal() {
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    tabs = [...tabs, { id: tabId, sessionId: '', channelId, title: t('app.localTerminal'), isLocal: true }]
    activeTabId = tabId
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

  function handleShortcutAction(action: string) {
    switch (action) {
      case 'new-tab':
        if (activeSessionId) {
          handleConnect(activeSessionId, t('app.newTab', { count: String(tabs.length + 1) }))
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
    api.updateTrayTooltip()
  }

  $effect(() => {
    if (tabs.length > 0) {
      const activeTab = tabs.find((t) => t.id === activeTabId)
      if (activeTab) {
        activeSessionId = activeTab.sessionId
      }
    }
    void tabs
    void activeTabId
    persistTabState()
  })

  $effect(() => {
    const unregister = registerHandler(handleShortcutAction)
    return unregister
  })

  $effect(() => {
    setShortcutsEnabled(!showNewSession && !showSettings)
  })

  $effect(() => {
    const unlisten = listen<string>('menu-event', async (event) => {
      switch (event.payload) {
        case 'new_session':
          showNewSession = true
          break
        case 'import_sessions':
          showImport = true
          break
        case 'close_tab':
          if (activeTabId) closeTab(activeTabId)
          break
        case 'quit':
          window.close()
          break
        case 'copy': {
          const sel = window.getSelection()?.toString() ?? ''
          if (sel) {
            try {
              await navigator.clipboard.writeText(sel)
            } catch {
              document.execCommand('copy')
            }
          }
          break
        }
        case 'paste': {
          try {
            const text = await navigator.clipboard.readText()
            if (text) {
              const el = document.activeElement
              if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
                const start = el.selectionStart ?? 0
                const end = el.selectionEnd ?? 0
                el.setRangeText(text, start, end, 'end')
                el.dispatchEvent(new Event('input', { bubbles: true }))
              }
            }
          } catch {
            document.execCommand('paste')
          }
          break
        }
        case 'select_all':
          document.execCommand('selectAll')
          break
        case 'settings':
          showSettings = !showSettings
          break
        case 'connect':
          if (selectedSessionId) {
            try {
              const sessions = await api.listSessions()
              const session = findSessionInTree(sessions, selectedSessionId)
              if (session) await api.sshConnect(session.id)
              handleConnect(session!.id, session!.name || session!.host)
            } catch (e) {
              console.error('Connect failed:', e)
            }
          } else {
            showNewSession = true
          }
          break
        case 'disconnect':
          if (activeSessionId) {
            try {
              await api.sshDisconnect(activeSessionId)
            } catch (e) {
              console.error('Disconnect failed:', e)
            }
          }
          break
        case 'duplicate':
          if (selectedSessionId && sidebarSessions) {
            const session = findSessionInTree(sidebarSessions, selectedSessionId)
            if (session) {
              try {
                const { id, last_connected, ...rest } = session
                await api.createSession({ ...rest, name: rest.name + ' (copy)' })
              } catch (e) {
                console.error('Duplicate failed:', e)
              }
            }
          }
          break
        case 'delete_session':
          if (selectedSessionId) {
            try {
              await api.deleteSession(selectedSessionId)
            } catch (e) {
              console.error('Delete failed:', e)
            }
          }
          break
        case 'toggle_sidebar':
          toggleSidebar()
          break
        case 'local_terminal':
          openLocalTerminal()
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
          try {
            appVersion = await api.getAppVersion()
          } catch {
            appVersion = 'unknown'
          }
          showAbout = true
          break
        case 'check_updates':
          updateStatus = t('about.checking')
          try {
            const result = await api.checkForUpdates()
            if (result?.has_update) {
              updateStatus = t('about.updateAvailable', { version: result.version ?? '' })
            } else {
              updateStatus = t('about.upToDate')
            }
          } catch {
            updateStatus = t('about.updateFailed')
          }
          break
      }
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  })
</script>

  <div class="app-shell" role="application" aria-label={t('app.title')}>
  {#if showSidebar}
    <aside class="sidebar" role="navigation" aria-label={t('sidebar.sessionList')}>
      <Sidebar onConnect={handleConnect} onNewSession={() => (showNewSession = true)} bind:showImport bind:selectedSessionId bind:sessions={sidebarSessions} />
    </aside>
  {/if}
  <main class="content" role="main">
    <div class="toolbar" role="toolbar" aria-label={t('toolbar.mainToolbar')}>
      <button class="toolbar-btn" onclick={toggleSidebar} aria-label={showSidebar ? t('toolbar.hideSidebar') : t('toolbar.showSidebar')} aria-expanded={showSidebar}>
        {showSidebar ? '<' : '>'}
      </button>
      <button class="toolbar-btn" onclick={() => (showNewSession = true)} aria-label={t('toolbar.newSession')}>{t('toolbar.newSession')}</button>
      {#if activeSessionId}
        <div class="toolbar-separator" role="separator"></div>
        <button class="toolbar-btn" class:active={activeView === 'terminal'} onclick={() => (activeView = 'terminal')} aria-pressed={activeView === 'terminal'}>{t('toolbar.terminal')}</button>
        <button class="toolbar-btn" class:active={activeView === 'files'} onclick={() => (activeView = 'files')} aria-pressed={activeView === 'files'}>{t('toolbar.files')}</button>
        <button class="toolbar-btn" class:active={activeView === 'tools'} onclick={() => (activeView = 'tools')} aria-pressed={activeView === 'tools'}>{t('toolbar.tools')}</button>
      {/if}
      <div class="toolbar-spacer"></div>
      {#if updateStatus}
        <span class="update-status">{updateStatus}</span>
      {/if}
      <button class="toolbar-btn theme-toggle" onclick={handleToggleTheme} title={t('toolbar.toggleTheme')} aria-label={theme === 'dark' ? t('toolbar.switchToLight') : t('toolbar.switchToDark')}>
        {theme === 'dark' ? '&#9728;' : '&#9790;'}
      </button>
      <button class="toolbar-btn" onclick={() => (showSettings = true)} aria-label={t('toolbar.openSettings')}>{t('toolbar.settings')}</button>
    </div>

    {#if activeSessionId}
      <div class="main-views">
        <div class="view" class:hidden={activeView !== 'terminal'} role="tabpanel" aria-label={t('toolbar.terminal')}>
          <TerminalHolder bind:tabs bind:activeTabId />
        </div>
        <div class="view" class:hidden={activeView !== 'files'} role="tabpanel" aria-label={t('toolbar.fileBrowser')}>
          <div class="files-layout">
            <FileBrowser sessionId={activeSessionId} />
            <TransferQueue />
          </div>
        </div>
        <div class="view" class:hidden={activeView !== 'tools'} role="tabpanel" aria-label={t('toolbar.tools')}>
          <ToolsPanel sessionId={activeSessionId} />
        </div>
      </div>
    {:else}
      <div class="empty-state" role="status">
        <p>{t('app.noActiveSession')}</p>
        <p class="hint">{t('app.connectHint')}</p>
      </div>
    {/if}
  </main>
</div>

{#if showNewSession}
  <NewSessionDialog onclose={() => (showNewSession = false)} />
{/if}

<SettingsDialog bind:open={showSettings} />

{#if showAbout}
  <div class="backdrop" onclick={(e) => { if (e.target === e.currentTarget) showAbout = false }} onkeydown={(e) => { if (e.key === 'Escape') showAbout = false }} role="dialog" aria-modal="true" aria-label={t('about.title')} tabindex={-1}>
    <div class="about-dialog" role="document">
      <div class="about-header">
        <h3>{t('about.title')}</h3>
        <button class="close-btn" onclick={() => (showAbout = false)} aria-label={t('common.close')}>x</button>
      </div>
      <div class="about-body">
        <p class="about-name">SlopSSH</p>
        <p class="about-version">{t('about.version')}: {appVersion}</p>
        <p class="about-desc">{t('about.description')}</p>
      </div>
      <div class="about-actions">
        <button class="save-btn" onclick={async () => { updateStatus = t('about.checking'); try { const r = await api.checkForUpdates(); updateStatus = r?.has_update ? t('about.updateAvailable', { version: r.version ?? '' }) : t('about.upToDate') } catch { updateStatus = t('about.updateFailed') } }}>{t('about.checkUpdates')}</button>
        <button class="cancel-btn" onclick={() => (showAbout = false)}>{t('common.close')}</button>
      </div>
    </div>
  </div>
{/if}

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

  .update-status {
    font-size: 11px;
    color: var(--text-tertiary);
    padding: 2px 8px;
    background: var(--bg-hover);
    border-radius: 4px;
  }

  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .about-dialog {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: 8px;
    width: 360px;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
  }

  .about-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-primary);
  }

  .about-header h3 {
    margin: 0;
    font-size: 14px;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 14px;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .about-body {
    padding: 24px 16px;
    text-align: center;
  }

  .about-name {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 8px 0;
  }

  .about-version {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 0 0 12px 0;
  }

  .about-desc {
    font-size: 12px;
    color: var(--text-tertiary);
    margin: 0;
  }

  .about-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border-primary);
  }

  .save-btn {
    background: var(--accent);
    border: none;
    color: var(--text-inverse);
    padding: 6px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .save-btn:hover {
    background: var(--accent-hover);
  }

  .cancel-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 6px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
</style>
