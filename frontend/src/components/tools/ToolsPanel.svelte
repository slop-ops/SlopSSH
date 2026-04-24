<script lang="ts">
  import ProcessViewer from './ProcessViewer.svelte'
  import LogViewer from './LogViewer.svelte'

  let { sessionId }: { sessionId: string } = $props()

  let activeTool = $state('processes')

  const tools = [
    { id: 'processes', label: 'Processes' },
    { id: 'logs', label: 'Logs' },
  ]
</script>

<div class="tools-panel">
  <div class="tool-tabs">
    {#each tools as tool}
      <button
        class="tool-tab"
        class:active={activeTool === tool.id}
        onclick={() => (activeTool = tool.id)}
      >
        {tool.label}
      </button>
    {/each}
  </div>
  <div class="tool-content">
    {#if activeTool === 'processes'}
      <ProcessViewer {sessionId} />
    {:else if activeTool === 'logs'}
      <LogViewer {sessionId} />
    {/if}
  </div>
</div>

<style>
  .tools-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1f2b;
  }

  .tool-tabs {
    display: flex;
    gap: 2px;
    padding: 0 8px;
    background: #16171d;
    border-bottom: 1px solid #2e303a;
  }

  .tool-tab {
    background: transparent;
    border: none;
    color: #9ca3af;
    padding: 8px 14px;
    cursor: pointer;
    font-size: 12px;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
  }

  .tool-tab:hover {
    color: #e0e0e0;
  }

  .tool-tab.active {
    color: #4a90d9;
    border-bottom-color: #4a90d9;
  }

  .tool-content {
    flex: 1;
    overflow: hidden;
  }
</style>
