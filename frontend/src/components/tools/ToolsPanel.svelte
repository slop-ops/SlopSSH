<script lang="ts">
  import ProcessViewer from './ProcessViewer.svelte'
  import LogViewer from './LogViewer.svelte'
  import DiskAnalyzer from './DiskAnalyzer.svelte'
  import SearchPanel from './SearchPanel.svelte'
  import SysInfoPanel from './SysInfoPanel.svelte'
  import SystemLoad from './SystemLoad.svelte'
  import PortViewer from './PortViewer.svelte'
  import KeyManager from './KeyManager.svelte'

  let { sessionId }: { sessionId: string } = $props()

  let activeTool = $state('processes')

  const tools = [
    { id: 'processes', label: 'Processes' },
    { id: 'logs', label: 'Logs' },
    { id: 'disk', label: 'Disk' },
    { id: 'search', label: 'Search' },
    { id: 'sysinfo', label: 'System Info' },
    { id: 'load', label: 'System Load' },
    { id: 'ports', label: 'Ports' },
    { id: 'keys', label: 'SSH Keys' },
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
    {:else if activeTool === 'disk'}
      <DiskAnalyzer {sessionId} />
    {:else if activeTool === 'search'}
      <SearchPanel {sessionId} />
    {:else if activeTool === 'sysinfo'}
      <SysInfoPanel {sessionId} />
    {:else if activeTool === 'load'}
      <SystemLoad {sessionId} />
    {:else if activeTool === 'ports'}
      <PortViewer {sessionId} />
    {:else if activeTool === 'keys'}
      <KeyManager {sessionId} />
    {/if}
  </div>
</div>

<style>
  .tools-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary);
  }

  .tool-tabs {
    display: flex;
    gap: 0;
    padding: 0 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-primary);
    overflow-x: auto;
    flex-shrink: 0;
  }

  .tool-tab {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 8px 12px;
    cursor: pointer;
    font-size: 12px;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
    white-space: nowrap;
  }

  .tool-tab:hover {
    color: var(--text-primary);
  }

  .tool-tab.active {
    color: var(--accent-text);
    border-bottom-color: var(--accent);
  }

  .tool-content {
    flex: 1;
    overflow: hidden;
  }
</style>
