<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import SessionTabBar, { type SessionTabItem } from './SessionTabBar.svelte'
  import TerminalHolder from '../terminal/TerminalHolder.svelte'
  import DualPaneFileBrowser from '../files/DualPaneFileBrowser.svelte'
  import TransferQueue from '../files/TransferQueue.svelte'
  import ToolsPanel from '../tools/ToolsPanel.svelte'
  import StatusBar from './StatusBar.svelte'
  import Icon from '../common/Icon.svelte'
  import NewSessionDialog from '../session/NewSessionDialog.svelte'
  import SettingsDialog from '../settings/SettingsDialog.svelte'
  import Toast from '../common/Toast.svelte'
  import { getTheme, toggleTheme, persistTheme } from '$lib/stores/theme.svelte'
  import { t } from '$lib/utils/i18n'
  import { registerHandler, setEnabled as setShortcutsEnabled } from '$lib/utils/shortcuts'
  import { listen } from '@tauri-apps/api/event'
  import * as api from '$lib/api/invoke'
  import { clearSessionCache } from '$lib/utils/toolCache'
  import { getNextTerminalTitle, placeAdjacent } from '$lib/utils/tabs'
  import type { TabState, SavedTab, SessionInfo, SessionFolder } from '$lib/types'

  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
    isLocal?: boolean
  }

  interface SessionWorkspace {
    sessionId: string
    name?: string
    host?: string
    username?: string
    port?: number
    tabs: Tab[]
    activeTabId: string
    activeView: 'terminal' | 'files' | 'tools'
    splitMode: 'none' | 'horizontal' | 'vertical'
    splitActivePane: 'primary' | 'secondary'
    splitActiveTabId: string
    splitRatio: number
    status?: 'connected' | 'reconnecting' | 'disconnected'
  }

class Workspace implements SessionWorkspace {
    sessionId: string
    name = $state<string | undefined>()
    host = $state<string | undefined>()
    username = $state<string | undefined>()
    port = $state<number | undefined>()
    tabs = $state<Tab[]>([])
    activeTabId = $state<string>('')
    activeView = $state<'terminal' | 'files' | 'tools'>('terminal')
    splitMode = $state<'none' | 'horizontal' | 'vertical'>('none')
    splitActivePane = $state<'primary' | 'secondary'>('primary')
    splitActiveTabId = $state<string>('')
    splitRatio = $state<number>(50)
    status = $state<'connected' | 'reconnecting' | 'disconnected'>('connected')

    constructor(sessionId: string, info?: Partial<SessionInfo>) {
      this.sessionId = sessionId
      this.name = info?.name
      this.host = info?.host
      this.username = info?.username
      this.port = info?.port
    }
  }

  function createWorkspace(sessionId: string, info?: Partial<SessionInfo>): SessionWorkspace {
    return new Workspace(sessionId, info)
  }

  let showSidebar = $state(true)
  let sidebarCollapsed = $state(false)
  let showNewSession = $state(false)
  let targetFolderIdForNewSession = $state<string | null>(null)
  let terminalSnippetsOpen = $state(false)
  let showSettings = $state(false)
  let activeSessionId = $state('')
  let theme = $state(getTheme())
  let restoring = $state(true)
  let showAbout = $state(false)
  let showImport = $state(false)
  let updateStatus = $state('')
  let appVersion = $state('')
  let selectedSessionId = $state('')
  let sidebarSessions = $state<SessionFolder | null>(null)
  let isDragging = $state(false)
  let disconnectedSessionIds = $state<Set<string>>(new Set())
  let reconnectingSessionId = $state<string | null>(null)
  let reconnectError = $state<string>('')
  let localTabs: Tab[] = $state([])
  let localActiveTabId = $state('')
  let workspaces: Map<string, SessionWorkspace> = $state(new Map())

  $effect(() => {
    if (localTabs.length > 0) {
      if (!localActiveTabId || !localTabs.some((t) => t.id === localActiveTabId)) {
        localActiveTabId = localTabs[localTabs.length - 1].id
      }
    } else {
      localActiveTabId = ''
    }
  })

  let ws = $derived(workspaces.get(activeSessionId) ?? null)

  let tabs = $derived(ws ? ws.tabs : localTabs)
  let activeTabId = $derived(ws ? ws.activeTabId : localActiveTabId)
  let activeView = $derived(ws ? ws.activeView : 'terminal')
  let splitMode = $derived(ws ? ws.splitMode : 'none')
  let splitActivePane = $derived(ws ? ws.splitActivePane : 'primary')
  let splitActiveTabId = $derived(ws ? ws.splitActiveTabId : '')
  let splitRatio = $derived(ws ? ws.splitRatio : 50)

  let connectedSessionIds = $derived(
    [...workspaces.keys()].filter((id) => id !== '__local__' && !disconnectedSessionIds.has(id))
  )

  let sessionTabItems = $derived<SessionTabItem[]>([
    ...Array.from(workspaces.values()).map((w) => {
      const isDisconnected = disconnectedSessionIds.has(w.sessionId)
      const isReconnecting = reconnectingSessionId === w.sessionId
      return {
        id: w.sessionId,
        title: w.name || w.host || `Session`,
        subtitle: w.username && w.host ? `${w.username}@${w.host}` : (w.host ? `${w.host}:${w.port ?? 22}` : undefined),
        status: isReconnecting ? ('reconnecting' as const) : (isDisconnected ? ('disconnected' as const) : ('connected' as const)),
        isLocal: false,
      }
    }),
    ...(localTabs.length > 0 ? [{
      id: '__local__',
      title: t('app.localTerminal'),
      status: 'connected' as const,
      isLocal: true,
    }] : [])
  ])

  function updateWorkspace(sessionId: string, updater: (ws: SessionWorkspace) => void) {
    const existing = workspaces.get(sessionId)
    if (existing) {
      updater(existing)
      workspaces = new Map(workspaces)
    }
  }

  function setActiveView(view: 'terminal' | 'files' | 'tools') {
    if (ws) {
      updateWorkspace(activeSessionId, (w) => { w.activeView = view })
    }
  }

  function setActiveTabId(tabId: string) {
    if (ws) {
      updateWorkspace(activeSessionId, (w) => { w.activeTabId = tabId })
    }
  }

  function setSplitMode(mode: 'none' | 'horizontal' | 'vertical') {
    if (!activeSessionId || activeSessionId === '__local__') return
    updateWorkspace(activeSessionId, (w) => {
      if (mode === 'none' && w.splitMode !== 'none') {
        if (w.activeTabId && w.splitActiveTabId && w.activeTabId !== w.splitActiveTabId) {
          w.tabs = placeAdjacent(w.tabs, w.activeTabId, w.splitActiveTabId)
        }
        if (w.splitActivePane === 'secondary' && w.splitActiveTabId) {
          w.activeTabId = w.splitActiveTabId
        }
      }
      w.splitMode = mode
    })
  }

  function setSplitActivePane(pane: 'primary' | 'secondary') {
    if (ws) {
      updateWorkspace(activeSessionId, (w) => { w.splitActivePane = pane })
    }
  }

  function setSplitRatio(ratio: number) {
    if (ws) {
      updateWorkspace(activeSessionId, (w) => { w.splitRatio = ratio })
    }
  }

  async function restoreTabState() {
    try {
      const state = await api.loadTabState()
      if (state?.tabs?.length) {
        for (const st of state.tabs) {
          if (st.is_local) {
            localTabs.push({
              id: crypto.randomUUID(),
              sessionId: st.session_id,
              channelId: st.channel_id,
              title: st.title,
              isLocal: true,
            })
          } else if (st.session_id) {
            try {
              const connected = await api.sshIsConnected(st.session_id)
              if (connected) {
                let workspace = workspaces.get(st.session_id)
                if (!workspace) {
                  workspace = createWorkspace(st.session_id)
                  workspaces.set(st.session_id, workspace)
                }
                workspace.tabs.push({
                  id: crypto.randomUUID(),
                  sessionId: st.session_id,
                  channelId: crypto.randomUUID(),
                  title: st.title,
                })
                if (!workspace.activeTabId) {
                  workspace.activeTabId = workspace.tabs[workspace.tabs.length - 1].id
                }
              }
            } catch {
              // session not connected, skip
            }
          }
        }
        workspaces = new Map(workspaces)
        localTabs = [...localTabs]
        if (state.active_tab_id) {
          for (const [sid, workspace] of workspaces) {
            if (workspace.tabs.some((t) => t.sessionId === state.active_tab_id)) {
              activeSessionId = sid
              break
            }
          }
        }
        if (!activeSessionId && workspaces.size > 0) {
          activeSessionId = workspaces.keys().next().value ?? ''
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
      const allTabs: SavedTab[] = [
        ...localTabs.map((tab) => ({
          session_id: tab.sessionId,
          channel_id: tab.channelId,
          title: tab.title,
          is_local: true,
        })),
        ...[...workspaces.values()].flatMap((w) =>
          w.tabs.map((tab) => ({
            session_id: tab.sessionId,
            channel_id: tab.channelId,
            title: tab.title,
            is_local: false,
          }))
        ),
      ]
      await api.saveTabState({ tabs: allTabs, active_tab_id: activeSessionId || null })
    } catch {
      // ignore save errors
    }
  }

  function handleConnect(sessionId: string, name: string) {
    disconnectedSessionIds.delete(sessionId)
    disconnectedSessionIds = new Set(disconnectedSessionIds)
    let workspace = workspaces.get(sessionId)
    if (!workspace) {
      let info: SessionInfo | null = null
      if (sidebarSessions) {
        info = findSessionInTree(sidebarSessions, sessionId)
      }
      workspace = createWorkspace(sessionId, info ?? { name })
      if (!workspace.name) workspace.name = name
      workspaces.set(sessionId, workspace)
    } else {
      workspace.status = 'connected'
    }
    if (workspace.tabs.length === 0) {
      const channelId = crypto.randomUUID()
      const tabId = crypto.randomUUID()
      workspace.tabs = [{ id: tabId, sessionId, channelId, title: workspace.name || 'Terminal 1' }]
      workspace.activeTabId = tabId
    }
    workspace.activeView = 'terminal'
    activeSessionId = sessionId
    workspaces = new Map(workspaces)
    api.updateTrayTooltip()
  }

  function handleSessionSelect(sessionId: string) {
    if (workspaces.has(sessionId)) {
      activeSessionId = sessionId
      selectedSessionId = sessionId
    }
  }

  async function handleDisconnectSession(sessionId: string) {
    try {
      await api.sshDisconnect(sessionId)
    } catch {
      // session may already be disconnected
    }
    clearSessionCache(sessionId)
    disconnectedSessionIds.delete(sessionId)
    disconnectedSessionIds = new Set(disconnectedSessionIds)
    workspaces.delete(sessionId)
    workspaces = new Map(workspaces)
    if (activeSessionId === sessionId) {
      activeSessionId = workspaces.size > 0 ? workspaces.keys().next().value ?? '' : (localTabs.length > 0 ? '__local__' : '')
    }
    api.updateTrayTooltip()
  }

  function handleSessionDisconnect(sessionId: string) {
    disconnectedSessionIds.add(sessionId)
    disconnectedSessionIds = new Set(disconnectedSessionIds)
    const targetWs = workspaces.get(sessionId)
    if (targetWs) {
      targetWs.status = 'disconnected'
      workspaces = new Map(workspaces)
    }
  }

  function handleCloseSessionTab(sessionId: string) {
    if (sessionId === '__local__') {
      localTabs = []
      if (activeSessionId === '__local__') {
        activeSessionId = workspaces.size > 0 ? workspaces.keys().next().value ?? '' : ''
      }
    } else {
      handleDisconnectSession(sessionId)
    }
  }

  async function handleReconnectSession(sessionId: string) {
    reconnectingSessionId = sessionId
    reconnectError = ''
    try {
      await api.sshConnect(sessionId)
      disconnectedSessionIds.delete(sessionId)
      disconnectedSessionIds = new Set(disconnectedSessionIds)
      const targetWs = workspaces.get(sessionId)
      if (targetWs) {
        targetWs.status = 'connected'
        workspaces = new Map(workspaces)
      }
    } catch (e) {
      reconnectError = String(e)
    } finally {
      reconnectingSessionId = null
    }
  }

  function toggleSidebar() {
    showSidebar = !showSidebar
  }

  function handleToggleTheme() {
    toggleTheme()
    theme = getTheme()
    persistTheme(theme)
  }

  function openLocalTerminal() {
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    const title = getNextTerminalTitle(localTabs, t('app.localTerminal') || 'Local Terminal')
    localTabs = [...localTabs, { id: tabId, sessionId: '', channelId, title, isLocal: true }]
    localActiveTabId = tabId
    activeSessionId = '__local__'
  }

  function openNewTerminal() {
    if (!activeSessionId || activeSessionId === '__local__') {
      openLocalTerminal()
      return
    }
    const targetWs = workspaces.get(activeSessionId)
    if (!targetWs) return
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    const title = getNextTerminalTitle(targetWs.tabs, 'Terminal')
    const newTab: Tab = { id: tabId, sessionId: activeSessionId, channelId, title }

    updateWorkspace(activeSessionId, (w) => {
      w.tabs = [...w.tabs, newTab]
      if (w.splitMode !== 'none' && w.splitActivePane === 'secondary') {
        w.splitActiveTabId = tabId
      } else {
        w.activeTabId = tabId
      }
      w.activeView = 'terminal'
    })
  }

  function activateSplitMode(mode: 'vertical' | 'horizontal') {
    if (!activeSessionId || activeSessionId === '__local__') return
    const targetWs = workspaces.get(activeSessionId)
    if (!targetWs) return

    updateWorkspace(activeSessionId, (w) => {
      w.splitMode = mode

      if (w.tabs.length === 0) {
        const tab1Id = crypto.randomUUID()
        const tab2Id = crypto.randomUUID()
        const tab1: Tab = { id: tab1Id, sessionId: activeSessionId, channelId: crypto.randomUUID(), title: getNextTerminalTitle([]) }
        const tab2: Tab = { id: tab2Id, sessionId: activeSessionId, channelId: crypto.randomUUID(), title: getNextTerminalTitle([tab1]) }
        w.tabs = [tab1, tab2]
        w.activeTabId = tab1Id
        w.splitActiveTabId = tab2Id
        w.splitActivePane = 'secondary'
        return
      }

      if (w.tabs.length === 1) {
        const tab2Id = crypto.randomUUID()
        const tab2: Tab = { id: tab2Id, sessionId: activeSessionId, channelId: crypto.randomUUID(), title: getNextTerminalTitle(w.tabs) }
        w.tabs = [...w.tabs, tab2]
        w.splitActiveTabId = tab2Id
        w.splitActivePane = 'secondary'
        return
      }

      const validSecondary = w.tabs.find((t) => t.id === w.splitActiveTabId && t.id !== w.activeTabId)
      if (!validSecondary) {
        const other = w.tabs.find((t) => t.id !== w.activeTabId)
        if (other) {
          w.splitActiveTabId = other.id
        }
      }
      w.splitActivePane = 'secondary'
    })
  }

  function closeTerminalTab(sessionId: string, tabId: string) {
    if (sessionId === '__local__') {
      localTabs = localTabs.filter((t) => t.id !== tabId)
      if (localActiveTabId === tabId) {
        localActiveTabId = localTabs.length > 0 ? localTabs[localTabs.length - 1].id : ''
      }
      return
    }

    updateWorkspace(sessionId, (w) => {
      const isPrimary = w.activeTabId === tabId
      const isSecondary = w.splitActiveTabId === tabId
      w.tabs = w.tabs.filter((t) => t.id !== tabId)

      if (w.splitMode !== 'none') {
        if (w.tabs.length < 2) {
          w.splitMode = 'none'
          w.activeTabId = w.tabs[0]?.id ?? ''
          w.splitActiveTabId = ''
        } else {
          if (isPrimary) {
            const next = w.tabs.find((t) => t.id !== w.splitActiveTabId)
            w.activeTabId = next?.id ?? ''
          }
          if (isSecondary) {
            const next = w.tabs.find((t) => t.id !== w.activeTabId)
            w.splitActiveTabId = next?.id ?? ''
          }
        }
      } else {
        if (isPrimary) {
          w.activeTabId = w.tabs.length > 0 ? w.tabs[w.tabs.length - 1].id : ''
        }
      }
    })
  }

  function closeTab(tabId: string) {
    if (activeSessionId) {
      closeTerminalTab(activeSessionId, tabId)
    }
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
        if (activeSessionId && activeSessionId !== '__local__') {
          openNewTerminal()
        } else {
          openLocalTerminal()
        }
        break
      case 'close-tab':
        if (ws && ws.activeTabId) {
          closeTab(ws.activeTabId)
        } else if (activeTabId) {
          closeTab(activeTabId)
        }
        break
      case 'next-tab': {
        const currentTabs = ws ? ws.tabs : localTabs
        const idx = currentTabs.findIndex((t) => t.id === activeTabId)
        if (idx >= 0 && idx < currentTabs.length - 1) {
          if (ws) setActiveTabId(currentTabs[idx + 1].id)
        }
        break
      }
      case 'prev-tab': {
        const currentTabs = ws ? ws.tabs : localTabs
        const idx = currentTabs.findIndex((t) => t.id === activeTabId)
        if (idx > 0) {
          if (ws) setActiveTabId(currentTabs[idx - 1].id)
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
        if (activeSessionId && ws) setActiveView(ws.activeView === 'files' ? 'terminal' : 'files')
        break
      case 'toggle-tools':
        if (activeSessionId && ws) setActiveView(ws.activeView === 'tools' ? 'terminal' : 'tools')
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

  $effect(() => {
    void activeSessionId
    void workspaces
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
          if (selectedSessionId && sidebarSessions) {
            const session = findSessionInTree(sidebarSessions, selectedSessionId)
            if (session) {
              handleConnect(session.id, session.name || session.host)
            }
          } else {
            showNewSession = true
          }
          break
        case 'disconnect':
          if (activeSessionId) {
            try {
              await api.sshDisconnect(activeSessionId)
              clearSessionCache(activeSessionId)
              workspaces.delete(activeSessionId)
              workspaces = new Map(workspaces)
              const remaining = [...workspaces.keys()]
              activeSessionId = remaining.length > 0 ? remaining[remaining.length - 1] : ''
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
                sidebarSessions = await api.listSessions()
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
              sidebarSessions = await api.listSessions()
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
          if (activeSessionId && ws) setActiveView('files')
          break
        case 'process_viewer':
        case 'log_viewer':
        case 'disk_analyzer':
        case 'search':
        case 'port_forwarding':
        case 'port_viewer':
        case 'key_manager':
          if (activeSessionId && ws) setActiveView('tools')
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
    <aside class="sidebar" class:collapsed={sidebarCollapsed} role="navigation" aria-label={t('sidebar.sessionList')}>
      <Sidebar
        onConnect={handleConnect}
        onDisconnect={handleDisconnectSession}
        onNewSession={(folderId) => { targetFolderIdForNewSession = folderId ?? null; showNewSession = true }}
        onSessionSelect={handleSessionSelect}
        bind:showImport
        bind:selectedSessionId
        bind:sessions={sidebarSessions}
        tabs={[...localTabs, ...[...workspaces.values()].flatMap((w) => w.tabs)]}
        {connectedSessionIds}
        {disconnectedSessionIds}
        collapsed={sidebarCollapsed}
      />
    </aside>
  {/if}
  <main class="content" role="main">
    <SessionTabBar
      sessions={sessionTabItems}
      {activeSessionId}
      onSelectSession={(id) => {
        activeSessionId = id
        if (id !== '__local__') {
          selectedSessionId = id
        }
      }}
      onCloseSession={handleCloseSessionTab}
      onNewSession={() => { targetFolderIdForNewSession = null; showNewSession = true }}
      onOpenLocalTerminal={openLocalTerminal}
    />
    <div class="toolbar" role="toolbar" aria-label={t('toolbar.mainToolbar')}>
      <button
        class="toolbar-btn"
        onclick={() => { sidebarCollapsed = !sidebarCollapsed }}
        aria-label={sidebarCollapsed ? t('toolbar.expandSidebar') : t('toolbar.collapseSidebar')}
        title={sidebarCollapsed ? t('toolbar.expandSidebar') : t('toolbar.collapseSidebar')}
      >
        <Icon name={sidebarCollapsed ? 'chevron-right' : 'chevron-left'} size={13} />
      </button>
      <button class="toolbar-btn" onclick={() => { targetFolderIdForNewSession = null; showNewSession = true }} aria-label={t('toolbar.newSession')}>{t('toolbar.newSession')}</button>
      {#if activeSessionId && activeSessionId !== '__local__'}
        <div class="toolbar-separator" role="separator"></div>
        <button class="toolbar-btn" class:active={activeView === 'terminal'} onclick={() => setActiveView('terminal')} aria-pressed={activeView === 'terminal'}>{t('toolbar.terminal')}</button>
        <button class="toolbar-btn" class:active={activeView === 'files'} onclick={() => setActiveView('files')} aria-pressed={activeView === 'files'}>{t('toolbar.files')}</button>
        <button class="toolbar-btn" class:active={activeView === 'tools'} onclick={() => setActiveView('tools')} aria-pressed={activeView === 'tools'}>{t('toolbar.tools')}</button>
      {/if}
      <div class="toolbar-spacer"></div>
      {#if updateStatus}
        <span class="update-status">{updateStatus}</span>
      {/if}
      <button class="toolbar-btn theme-toggle" onclick={handleToggleTheme} title={t('toolbar.toggleTheme')} aria-label={theme === 'dark' ? t('toolbar.switchToLight') : t('toolbar.switchToDark')}>
        {theme === 'dark' ? '\u2600' : '\u263E'}
      </button>
      <button class="toolbar-btn" onclick={() => (showSettings = true)} aria-label={t('toolbar.openSettings')}>{t('toolbar.settings')}</button>
    </div>

    {#if workspaces.size === 0 && localTabs.length === 0}
      <div class="empty-state" role="status">
        <p>{t('app.noActiveSession')}</p>
        <p class="hint">{t('app.connectHint')}</p>
      </div>
    {:else}
      {#each Array.from(workspaces.values()) as workspace (workspace.sessionId)}
        <div class="session-workspace" class:hidden={activeSessionId !== workspace.sessionId}>
          {#if disconnectedSessionIds.has(workspace.sessionId)}
            <div class="disconnect-banner" role="alert">
              <div class="disconnect-banner-content">
                <span class="disconnect-icon">&#9888;</span>
                <div class="disconnect-message">
                  <strong>{t('sessionTab.disconnected')}</strong>
                  {#if reconnectError && reconnectingSessionId === workspace.sessionId}
                    <span class="disconnect-detail"> — {reconnectError}</span>
                  {/if}
                </div>
              </div>
              <div class="disconnect-banner-actions">
                <button
                  class="reconnect-btn"
                  disabled={reconnectingSessionId === workspace.sessionId}
                  onclick={() => handleReconnectSession(workspace.sessionId)}
                >
                  {reconnectingSessionId === workspace.sessionId ? t('sessionTab.reconnecting') : t('sessionTab.reconnect')}
                </button>
                <button
                  class="disconnect-close-btn"
                  onclick={() => handleDisconnectSession(workspace.sessionId)}
                >
                  {t('sessionTab.closeSession')}
                </button>
              </div>
            </div>
          {/if}

          <div class="main-views">
            <div class="view" class:hidden={workspace.activeView !== 'terminal'} role="tabpanel" aria-label={t('toolbar.terminal')}>
              {#if workspace.tabs.length === 0}
                <div class="empty-terminal-state">
                  <div class="empty-terminal-icon">&gt;_</div>
                  <h3>{t('sessionTab.noTerminals')}</h3>
                  <button class="primary-btn" onclick={openNewTerminal}>
                    + {t('sessionTab.openTerminal')}
                  </button>
                </div>
              {:else}
                <div class="terminal-controls">
                  <button class="new-terminal-btn" onclick={openNewTerminal} title={t('terminal.newTerminal')} aria-label="New terminal tab">+</button>
                  <div class="split-group">
                    <button class="split-btn" class:active={workspace.splitMode === 'none'} onclick={() => setSplitMode('none')} title={t('terminal.noSplit')} aria-label="No split">
                      <Icon name="split-none" size={13} />
                    </button>
                    <button class="split-btn" class:active={workspace.splitMode === 'vertical'} onclick={() => activateSplitMode('vertical')} title={t('terminal.splitVertical')} aria-label="Split vertical">
                      <Icon name="split-v" size={13} />
                    </button>
                    <button class="split-btn" class:active={workspace.splitMode === 'horizontal'} onclick={() => activateSplitMode('horizontal')} title={t('terminal.splitHorizontal')} aria-label="Split horizontal">
                      <Icon name="split-h" size={13} />
                    </button>
                  </div>
                  <button
                    class="split-btn snippets-btn"
                    class:active={terminalSnippetsOpen}
                    onclick={() => (terminalSnippetsOpen = !terminalSnippetsOpen)}
                    title={t('terminal.snippets')}
                    aria-label={t('terminal.snippets')}
                  >
                    <Icon name="code" size={13} />
                    <span class="snippets-label">{t('terminal.snippets')}</span>
                  </button>
                </div>
                <TerminalHolder
                  bind:tabs={workspace.tabs}
                  bind:activeTabId={workspace.activeTabId}
                  splitMode={workspace.splitMode}
                  bind:splitActivePane={workspace.splitActivePane}
                  bind:splitActiveTabId={workspace.splitActiveTabId}
                  bind:splitRatio={workspace.splitRatio}
                  isSessionActive={activeSessionId === workspace.sessionId && workspace.activeView === 'terminal'}
                  bind:showSnippets={terminalSnippetsOpen}
                  onSessionDisconnect={handleSessionDisconnect}
                  onCloseTab={(tabId) => closeTerminalTab(workspace.sessionId, tabId)}
                  onSelectTab={(tabId) => {
                    workspace.activeTabId = tabId
                    updateWorkspace(workspace.sessionId, (w) => { w.activeTabId = tabId })
                  }}
                />
              {/if}
            </div>
            <div class="view" class:hidden={workspace.activeView !== 'files'} role="tabpanel" aria-label={t('toolbar.fileBrowser')}>
              <div class="files-layout">
                <DualPaneFileBrowser sessionId={workspace.sessionId} />
                <TransferQueue />
              </div>
            </div>
            <div class="view" class:hidden={workspace.activeView !== 'tools'} role="tabpanel" aria-label={t('toolbar.tools')}>
              <ToolsPanel sessionId={workspace.sessionId} />
            </div>
          </div>
        </div>
      {/each}

      {#if localTabs.length > 0}
        <div class="session-workspace" class:hidden={activeSessionId !== '__local__'}>
          <div class="main-views">
            <div class="view" role="tabpanel" aria-label={t('toolbar.terminal')}>
              <div class="terminal-controls">
                <button class="new-terminal-btn" onclick={openLocalTerminal} title={t('terminal.newTerminal')} aria-label="New terminal tab">+</button>
                <div class="toolbar-spacer"></div>
                <button
                  class="split-btn snippets-btn"
                  class:active={terminalSnippetsOpen}
                  onclick={() => (terminalSnippetsOpen = !terminalSnippetsOpen)}
                  title={t('terminal.snippets')}
                  aria-label={t('terminal.snippets')}
                >
                  <Icon name="code" size={13} />
                  <span class="snippets-label">{t('terminal.snippets')}</span>
                </button>
              </div>
              <TerminalHolder
                bind:tabs={localTabs}
                bind:activeTabId={localActiveTabId}
                isSessionActive={activeSessionId === '__local__'}
                bind:showSnippets={terminalSnippetsOpen}
                onSessionDisconnect={handleSessionDisconnect}
                onCloseTab={(tabId) => closeTerminalTab('__local__', tabId)}
                onSelectTab={(tabId) => {
                  localActiveTabId = tabId
                }}
              />
            </div>
          </div>
        </div>
      {/if}
    {/if}
    <StatusBar
      activeSession={workspaces.get(activeSessionId) ? {
        name: workspaces.get(activeSessionId)?.name,
        host: workspaces.get(activeSessionId)?.host,
        user: workspaces.get(activeSessionId)?.username,
        port: workspaces.get(activeSessionId)?.port,
      } : null}
      {activeSessionId}
      {activeView}
    />
  </main>
</div>

{#if showNewSession}
  <NewSessionDialog
    initialFolderId={targetFolderIdForNewSession}
    onclose={() => { showNewSession = false; targetFolderIdForNewSession = null }}
  />
{/if}

<SettingsDialog bind:open={showSettings} />

<Toast />

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
    min-width: 48px;
    border-right: 1px solid var(--border-primary);
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--bg-secondary);
    flex-shrink: 0;
    transition: width 0.2s ease;
  }

  .sidebar.collapsed {
    width: 48px;
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

  .terminal-controls {
    display: flex;
    gap: 2px;
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-primary);
    flex-shrink: 0;
    align-items: center;
  }

  .new-terminal-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--success);
    width: 26px;
    height: 26px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    margin-right: 6px;
    transition: background 0.15s, color 0.15s;
  }

  .new-terminal-btn:hover {
    background: var(--success);
    color: var(--text-inverse);
    border-color: var(--success);
  }

  .split-group {
    display: flex;
    gap: 2px;
  }

  .split-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-tertiary);
    width: 26px;
    height: 26px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: background 0.15s, color 0.15s;
  }

  .split-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .split-btn.active {
    background: var(--accent-bg);
    border-color: var(--accent);
    color: var(--accent);
  }

  .snippets-btn {
    width: auto;
    padding: 0 8px;
    gap: 4px;
    font-size: 11px;
    font-weight: 500;
    margin-left: 6px;
  }

  .snippets-label {
    display: inline;
    font-size: 11px;
  }

  .session-workspace {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .session-workspace.hidden {
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

  /* Disconnect banner */
  .disconnect-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 16px;
    background: rgba(239, 68, 68, 0.15);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: #fca5a5;
    font-size: 13px;
    flex-shrink: 0;
  }

  .disconnect-banner-content {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .disconnect-icon {
    color: #ef4444;
    font-size: 16px;
  }

  .disconnect-detail {
    opacity: 0.85;
    font-size: 12px;
  }

  .disconnect-banner-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .reconnect-btn {
    background: #ef4444;
    color: #ffffff;
    border: none;
    border-radius: 4px;
    padding: 4px 12px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .reconnect-btn:hover:not(:disabled) {
    background: #dc2626;
  }

  .reconnect-btn:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .disconnect-close-btn {
    background: transparent;
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #fca5a5;
    border-radius: 4px;
    padding: 4px 10px;
    font-size: 12px;
    cursor: pointer;
  }

  .disconnect-close-btn:hover {
    background: rgba(239, 68, 68, 0.2);
  }

  /* Empty terminal state inside active workspace */
  .empty-terminal-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    height: 100%;
    gap: 16px;
    color: var(--text-muted, #64748b);
  }

  .empty-terminal-icon {
    font-family: monospace;
    font-size: 42px;
    font-weight: bold;
    color: var(--text-secondary, #94a3b8);
    opacity: 0.4;
  }

  .empty-terminal-state h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-secondary, #94a3b8);
  }

  .primary-btn {
    background: var(--accent, #6366f1);
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .primary-btn:hover {
    opacity: 0.9;
  }
</style>
