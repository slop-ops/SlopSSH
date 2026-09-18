<script lang="ts">
  import Terminal from '$lib/terminal/Terminal.svelte'
  import LocalTerminal from './LocalTerminal.svelte'
  import SnippetPanel from './SnippetPanel.svelte'
  import { getNextTerminalTitle } from '$lib/utils/tabs'
  import { t } from '$lib/utils/i18n'

  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
    isLocal?: boolean
  }

  let {
    tabs = $bindable([]),
    activeTabId = $bindable(''),
    splitMode = 'none',
    splitActivePane = $bindable<'primary' | 'secondary'>('primary'),
    splitActiveTabId = $bindable(''),
    splitRatio = $bindable(50),
    isSessionActive = true,
    showSnippets = $bindable(false),
    onSessionDisconnect,
    onSplitRatioChange,
    onCloseTab,
    onSelectTab,
  }: {
    tabs: Tab[]
    activeTabId: string
    splitMode?: 'none' | 'vertical' | 'horizontal'
    splitActivePane?: 'primary' | 'secondary'
    splitActiveTabId?: string
    splitRatio?: number
    isSessionActive?: boolean
    showSnippets?: boolean
    onSessionDisconnect?: (sessionId: string) => void
    onSplitRatioChange?: (ratio: number) => void
    onCloseTab?: (tabId: string) => void
    onSelectTab?: (tabId: string) => void
  } = $props()

  let snippetHandlers: Map<string, (cmd: string) => void> = $state(new Map())
  let isDragging = $state(false)

  // Ensure active tabs are valid
  $effect(() => {
    if (tabs.length > 0) {
      if (!activeTabId || !tabs.some((t) => t.id === activeTabId)) {
        activeTabId = tabs[0].id
      }
      if (splitMode !== 'none') {
        if (!splitActiveTabId || !tabs.some((t) => t.id === splitActiveTabId) || splitActiveTabId === activeTabId) {
          const other = tabs.find((t) => t.id !== activeTabId)
          if (other) {
            splitActiveTabId = other.id
          }
        }
      }
    }
  })

  function closeTab(tabId: string) {
    if (onCloseTab) {
      onCloseTab(tabId)
      return
    }
    const isPrimary = activeTabId === tabId
    const isSecondary = splitActiveTabId === tabId
    tabs = tabs.filter((t) => t.id !== tabId)
    if (splitMode !== 'none') {
      if (tabs.length < 2) {
        splitMode = 'none'
        activeTabId = tabs[0]?.id ?? ''
        splitActiveTabId = ''
      } else {
        if (isPrimary) {
          const next = tabs.find((t) => t.id !== splitActiveTabId)
          activeTabId = next?.id ?? ''
        }
        if (isSecondary) {
          const next = tabs.find((t) => t.id !== activeTabId)
          splitActiveTabId = next?.id ?? ''
        }
      }
    } else {
      if (isPrimary) {
        activeTabId = tabs.length > 0 ? tabs[tabs.length - 1].id : ''
      }
    }
  }

  function handleTabClick(tabId: string) {
    if (splitMode === 'none') {
      activeTabId = tabId
    } else {
      if (tabId === activeTabId) {
        splitActivePane = 'primary'
      } else if (tabId === splitActiveTabId) {
        splitActivePane = 'secondary'
      } else {
        if (splitActivePane === 'secondary') {
          splitActiveTabId = tabId
        } else {
          activeTabId = tabId
        }
      }
    }
    onSelectTab?.(tabId)
  }

  function registerSnippetHandler(tabId: string, handler: (cmd: string) => void) {
    snippetHandlers.set(tabId, handler)
  }

  function sendSnippet(cmd: string) {
    const currentTabId = splitMode !== 'none' && splitActivePane === 'secondary'
      ? splitActiveTabId
      : activeTabId
    const handler = snippetHandlers.get(currentTabId)
    if (handler) {
      handler(cmd)
    }
  }

  function openLocalTerminal() {
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    const title = getNextTerminalTitle(tabs, 'Local')
    tabs = [...tabs, { id: tabId, sessionId: '', channelId, title, isLocal: true }]
    if (splitMode !== 'none' && splitActivePane === 'secondary') {
      splitActiveTabId = tabId
    } else {
      activeTabId = tabId
    }
  }

  function handleSplitDividerMouseDown(e: MouseEvent) {
    isDragging = true
    const container = (e.currentTarget as HTMLElement).parentElement
    if (!container) return

    const rect = container.getBoundingClientRect()
    const isVert = splitMode === 'vertical'

    function onMouseMove(ev: MouseEvent) {
      const pos = isVert ? ev.clientX - rect.left : ev.clientY - rect.top
      const size = isVert ? rect.width : rect.height
      if (size <= 0) return
      let ratio = (pos / size) * 100
      ratio = Math.max(20, Math.min(80, ratio))
      splitRatio = Math.round(ratio)
      onSplitRatioChange?.(splitRatio)
    }

    function onMouseUp() {
      isDragging = false
      window.removeEventListener('mousemove', onMouseMove)
      window.removeEventListener('mouseup', onMouseUp)
    }

    window.addEventListener('mousemove', onMouseMove)
    window.addEventListener('mouseup', onMouseUp)
  }
</script>

<div class="terminal-holder">
  <div class="main-area">
    {#if tabs.length > 0}
      <div class="tab-bar">
        <div class="tab-items">
          {#each tabs as tab (tab.id)}
            {@const isTabPrimary = tab.id === activeTabId}
            {@const isTabSecondary = splitMode !== 'none' && tab.id === splitActiveTabId}
            {@const isTabFocused = splitMode === 'none'
              ? isTabPrimary
              : (splitActivePane === 'primary' ? isTabPrimary : isTabSecondary)}
            <div
              class="tab"
              class:active={isTabFocused}
              class:in-split={splitMode !== 'none' && (isTabPrimary || isTabSecondary)}
              class:local={tab.isLocal}
              role="tab"
              tabindex={0}
              onclick={() => handleTabClick(tab.id)}
              onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') handleTabClick(tab.id) }}
            >
              <span class="tab-title">{tab.title}</span>
              {#if splitMode !== 'none'}
                {#if isTabPrimary}
                  <span class="pane-badge">{splitMode === 'vertical' ? 'Left' : 'Top'}</span>
                {:else if isTabSecondary}
                  <span class="pane-badge">{splitMode === 'vertical' ? 'Right' : 'Bottom'}</span>
                {/if}
              {/if}
              <span
                class="tab-close"
                role="button"
                tabindex={-1}
                aria-label="Close tab"
                onclick={(e: Event) => { e.stopPropagation(); closeTab(tab.id) }}
                onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.stopPropagation(); closeTab(tab.id) } }}
              >
                x
              </span>
            </div>
          {/each}
        </div>
      </div>
      <div
        class="tab-content"
        class:split-vertical={splitMode === 'vertical'}
        class:split-horizontal={splitMode === 'horizontal'}
      >
        {#each tabs as tab (tab.id)}
          {@const isPrimary = tab.id === activeTabId}
          {@const isSecondary = splitMode !== 'none' && tab.id === splitActiveTabId}
          {@const isVisible = splitMode === 'none' ? isPrimary : (isPrimary || isSecondary)}
          {@const isPanelFocused = isSessionActive && (splitMode === 'none' ? isPrimary : (splitActivePane === 'primary' ? isPrimary : isSecondary))}
          <div
            class="terminal-panel"
            class:active={isVisible}
            class:inactive={!isVisible}
            class:single-pane={splitMode === 'none' && isPrimary}
            class:pane-primary={splitMode !== 'none' && isPrimary}
            class:pane-secondary={splitMode !== 'none' && isSecondary}
            class:active-pane={splitMode !== 'none' && isVisible && (splitActivePane === 'primary' ? isPrimary : isSecondary)}
            style:order={isPrimary ? 1 : (isSecondary ? 3 : 99)}
            style:flex={splitMode === 'none' ? (isPrimary ? '1 1 100%' : undefined) : (isPrimary ? `0 0 ${splitRatio}%` : (isSecondary ? '1 1 0%' : undefined))}
            onclick={() => {
              if (splitMode !== 'none') {
                if (isPrimary) splitActivePane = 'primary'
                else if (isSecondary) splitActivePane = 'secondary'
              }
            }}
            role="region"
            aria-label={isPrimary ? 'Primary terminal pane' : 'Secondary terminal pane'}
          >
            {#if tab.isLocal}
              <LocalTerminal channelId={tab.channelId} isActive={isPanelFocused} />
            {:else}
              <Terminal
                sessionId={tab.sessionId}
                channelId={tab.channelId}
                isActive={isPanelFocused}
                onSendSnippet={(handler) => registerSnippetHandler(tab.id, handler)}
                onDisconnect={() => onSessionDisconnect?.(tab.sessionId)}
              />
            {/if}
          </div>
        {/each}

        {#if splitMode !== 'none' && tabs.length > 1}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="split-divider"
            class:vertical={splitMode === 'vertical'}
            class:horizontal={splitMode === 'horizontal'}
            class:dragging={isDragging}
            style:order={2}
            ondblclick={() => { splitRatio = 50; onSplitRatioChange?.(50) }}
            onmousedown={handleSplitDividerMouseDown}
            title="Drag to resize, double-click to center (50/50)"
          ></div>
        {/if}
      </div>
    {:else}
      <div class="empty">
        <p>{t('app.noTerminalSessions')}</p>
        <button class="local-btn-empty" onclick={openLocalTerminal}>{t('app.openLocalTerminal')}</button>
      </div>
    {/if}
  </div>
  {#if showSnippets && tabs.length > 0}
    <SnippetPanel onSend={sendSnippet} onclose={() => (showSnippets = false)} />
  {/if}
</div>

<style>
  .terminal-holder {
    display: flex;
    height: 100%;
    width: 100%;
    background: var(--bg-primary);
    overflow: hidden;
  }

  .main-area {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .tab-bar {
    display: flex;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-primary);
    min-height: 36px;
    flex-shrink: 0;
    align-items: center;
  }

  .tab-items {
    display: flex;
    overflow-x: auto;
    flex: 1;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: transparent;
    border-right: 1px solid var(--border-primary);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
    white-space: nowrap;
    transition: background 0.15s, color 0.15s;
    user-select: none;
  }

  .tab:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    border-bottom: 2px solid var(--border-active);
  }

  .tab.in-split:not(.active) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab.local {
    border-left: 2px solid var(--success);
  }

  .tab-title {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pane-badge {
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--bg-hover);
    color: var(--accent-text);
    border: 1px solid var(--border-primary);
    margin-left: 2px;
  }

  .tab.active .pane-badge {
    background: var(--accent-bg);
    border-color: var(--accent);
  }

  .tab-close {
    font-size: 10px;
    padding: 2px 4px;
    border-radius: 3px;
    line-height: 1;
    opacity: 0.5;
  }

  .tab-close:hover {
    opacity: 1;
    background: var(--error-bg);
    color: var(--error);
  }

  .tab-content {
    position: relative;
    flex: 1;
    overflow: hidden;
    height: 100%;
    width: 100%;
    display: flex;
  }

  .tab-content.split-vertical {
    flex-direction: row;
  }

  .tab-content.split-horizontal {
    flex-direction: column;
  }

  .terminal-panel {
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .terminal-panel.single-pane {
    width: 100%;
    height: 100%;
  }

  .terminal-panel.pane-primary,
  .terminal-panel.pane-secondary {
    position: relative;
  }

  .terminal-panel.active {
    position: relative;
    visibility: visible;
    z-index: 1;
  }

  .terminal-panel.inactive {
    position: absolute;
    top: 0;
    left: -99999px;
    width: 100%;
    height: 100%;
    visibility: hidden;
    pointer-events: none;
    z-index: -1;
  }

  .terminal-panel.active-pane {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .split-divider {
    flex-shrink: 0;
    background: var(--border-primary);
    transition: background 0.15s;
    z-index: 2;
  }

  .split-divider:hover,
  .split-divider.dragging {
    background: var(--accent);
  }

  .split-divider.vertical {
    width: 4px;
    cursor: col-resize;
  }

  .split-divider.horizontal {
    height: 4px;
    cursor: row-resize;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
    gap: 12px;
  }

  .empty p {
    margin: 0;
  }

  .local-btn-empty {
    background: var(--bg-hover);
    border: 1px solid var(--border-primary);
    color: var(--success);
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
  }

  .local-btn-empty:hover {
    background: var(--bg-hover);
  }
</style>
