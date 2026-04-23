<script lang="ts">
  import Terminal from '$lib/terminal/Terminal.svelte'

  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
  }

  let { tabs = $bindable(), activeTabId = $bindable() }: { tabs: Tab[]; activeTabId: string } = $props()

  function closeTab(tabId: string) {
    tabs = tabs.filter((t) => t.id !== tabId)
    if (activeTabId === tabId) {
      activeTabId = tabs.length > 0 ? tabs[tabs.length - 1].id : ''
    }
  }
</script>

<div class="terminal-holder">
  {#if tabs.length > 0}
    <div class="tab-bar">
      {#each tabs as tab (tab.id)}
        <div
          class="tab"
          class:active={activeTabId === tab.id}
          role="tab"
          tabindex={0}
          onclick={() => (activeTabId = tab.id)}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') activeTabId = tab.id }}
        >
          <span class="tab-title">{tab.title}</span>
          <span class="tab-close" role="button" tabindex={-1} onclick={(e: Event) => { e.stopPropagation(); closeTab(tab.id) }}>x</span>
        </div>
      {/each}
    </div>
    <div class="tab-content">
      {#each tabs as tab (tab.id)}
        <div class="terminal-panel" style:display={activeTabId === tab.id ? 'contents' : 'none'}>
          <Terminal sessionId={tab.sessionId} channelId={tab.channelId} />
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty">
      <p>No terminal sessions</p>
    </div>
  {/if}
</div>

<style>
  .terminal-holder {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1a1a2e;
  }

  .tab-bar {
    display: flex;
    background: #16171d;
    border-bottom: 1px solid #2e303a;
    overflow-x: auto;
    min-height: 36px;
    flex-shrink: 0;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-right: 1px solid #2e303a;
    color: #9ca3af;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    white-space: nowrap;
    transition: background 0.15s, color 0.15s;
    border-radius: 0;
  }

  .tab:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .tab.active {
    background: #1a1a2e;
    color: #e0e0e0;
    border-bottom: 2px solid #4a90d9;
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
    background: #e06c7544;
    color: #e06c75;
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
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #9ca3af;
  }
</style>
