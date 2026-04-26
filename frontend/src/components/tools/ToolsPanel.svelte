<script lang="ts">
  import ProcessViewer from './ProcessViewer.svelte'
  import LogViewer from './LogViewer.svelte'
  import DiskAnalyzer from './DiskAnalyzer.svelte'
  import SearchPanel from './SearchPanel.svelte'
  import SysInfoPanel from './SysInfoPanel.svelte'
  import SystemLoad from './SystemLoad.svelte'
  import PortViewer from './PortViewer.svelte'
  import KeyManager from './KeyManager.svelte'
  import PortForwarding from './PortForwarding.svelte'
  import { t } from '$lib/utils/i18n'

  let { sessionId }: { sessionId: string } = $props()

  let activeTool = $state('processes')

  const tools = [
    { id: 'processes', label: () => t('tools.processes') },
    { id: 'logs', label: () => t('tools.logs') },
    { id: 'disk', label: () => t('tools.disk') },
    { id: 'search', label: () => t('tools.search') },
    { id: 'sysinfo', label: () => t('tools.sysinfo') },
    { id: 'load', label: () => t('tools.load') },
    { id: 'ports', label: () => t('tools.ports') },
    { id: 'keys', label: () => t('tools.keys') },
    { id: 'forwarding', label: () => t('tools.forwarding') },
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
        {tool.label()}
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
    {:else if activeTool === 'forwarding'}
      <PortForwarding {sessionId} />
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
