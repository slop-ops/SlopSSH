<script lang="ts">
  import Terminal from '$lib/terminal/Terminal.svelte'
  import LocalTerminal from './LocalTerminal.svelte'
  import SnippetPanel from './SnippetPanel.svelte'
  import { t } from '$lib/utils/i18n'

  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
    isLocal?: boolean
  }

  let { tabs = $bindable(), activeTabId = $bindable() }: { tabs: Tab[]; activeTabId: string } = $props()

  let showSnippets = $state(false)
  let snippetHandlers: Map<string, (cmd: string) => void> = $state(new Map())

  function closeTab(tabId: string) {
    tabs = tabs.filter((t) => t.id !== tabId)
    if (activeTabId === tabId) {
      activeTabId = tabs.length > 0 ? tabs[tabs.length - 1].id : ''
    }
  }

  function registerSnippetHandler(tabId: string, handler: (cmd: string) => void) {
    snippetHandlers.set(tabId, handler)
  }

  function sendSnippet(cmd: string) {
    const handler = snippetHandlers.get(activeTabId)
    if (handler) {
      handler(cmd)
    }
  }

  function openLocalTerminal() {
    const channelId = crypto.randomUUID()
    const tabId = crypto.randomUUID()
    tabs = [...tabs, { id: tabId, sessionId: '', channelId, title: 'Local', isLocal: true }]
    activeTabId = tabId
  }
</script>

<div class="terminal-holder">
  <div class="main-area">
    {#if tabs.length > 0}
      <div class="tab-bar">
        <div class="tab-items">
          {#each tabs as tab (tab.id)}
            <div
              class="tab"
              class:active={activeTabId === tab.id}
              class:local={tab.isLocal}
              role="tab"
              tabindex={0}
              onclick={() => (activeTabId = tab.id)}
              onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') activeTabId = tab.id }}
            >
              <span class="tab-title">{tab.title}</span>
              <span class="tab-close" role="button" tabindex={-1} aria-label="Close tab" onclick={(e: Event) => { e.stopPropagation(); closeTab(tab.id) }}>x</span>
            </div>
          {/each}
        </div>
        <button class="local-btn" onclick={openLocalTerminal} title={t('app.openLocalTerminal')} aria-label="Open local terminal">+$</button>
        <button class="snippet-toggle" class:active={showSnippets} onclick={() => (showSnippets = !showSnippets)} title="Snippets" aria-label="Toggle snippets">
          S
        </button>
      </div>
      <div class="tab-content">
        {#each tabs as tab (tab.id)}
          <div class="terminal-panel" style:display={activeTabId === tab.id ? 'contents' : 'none'}>
            {#if tab.isLocal}
              <LocalTerminal channelId={tab.channelId} />
            {:else}
              <Terminal
                sessionId={tab.sessionId}
                channelId={tab.channelId}
                onSendSnippet={(handler) => registerSnippetHandler(tab.id, handler)}
              />
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty">
        <p>{t('app.noTerminalSessions')}</p>
        <button class="local-btn-empty" onclick={openLocalTerminal}>{t('app.openLocalTerminal')}</button>
      </div>
    {/if}
  </div>
  {#if showSnippets && tabs.length > 0}
    <SnippetPanel onSend={sendSnippet} />
  {/if}
</div>

<style>
  .terminal-holder {
    display: flex;
    height: 100%;
    background: var(--bg-primary);
  }

  .main-area {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
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

  .tab.local {
    border-left: 2px solid var(--success);
  }

  .tab-title {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
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

  .local-btn,
  .snippet-toggle {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    width: 28px;
    height: 28px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 4px;
    padding: 0;
  }

  .local-btn:hover,
  .snippet-toggle:hover,
  .snippet-toggle.active {
    background: var(--bg-hover);
    color: var(--accent);
    border-color: var(--border-active);
  }

  .local-btn:hover {
    color: var(--success);
    border-color: var(--success);
  }

  .tab-content {
    flex: 1;
    overflow: hidden;
  }

  .terminal-panel {
    height: 100%;
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
