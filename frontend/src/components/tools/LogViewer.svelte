<script lang="ts">
  import * as api from '$lib/api/invoke'

  let { sessionId }: { sessionId: string } = $props()

  let logPath = $state('/var/log/syslog')
  let lines = $state<string[]>([])
  let loading = $state(false)
  let error = $state('')
  let searchQuery = $state('')
  let lineCount = $state(200)
  let autoRefresh = $state(false)
  let refreshInterval: ReturnType<typeof setInterval> | undefined

  $effect(() => {
    return () => {
      if (refreshInterval) clearInterval(refreshInterval)
    }
  })

  async function loadLog() {
    if (!logPath.trim()) return
    loading = true
    error = ''
    try {
      const result = await api.remoteExec(
        sessionId,
        `tail -n ${lineCount} ${logPath} 2>&1`,
        15,
      )
      lines = result.stdout.split('\n')
      if (result.exitCode !== 0 && lines.length <= 1) {
        error = result.stdout
        lines = []
      }
    } catch (e) {
      error = String(e)
      lines = []
    } finally {
      loading = false
    }
  }

  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh
    if (autoRefresh) {
      refreshInterval = setInterval(loadLog, 5000)
    } else {
      if (refreshInterval) clearInterval(refreshInterval)
      refreshInterval = undefined
    }
  }

  let filtered = $derived(
    !searchQuery
      ? lines
      : lines.filter((l) => l.toLowerCase().includes(searchQuery.toLowerCase())),
  )

  let matchCount = $derived(
    !searchQuery ? 0 : lines.filter((l) => l.toLowerCase().includes(searchQuery.toLowerCase())).length,
  )
</script>

<div class="log-viewer">
  <div class="toolbar">
    <input
      type="text"
      bind:value={logPath}
      placeholder="/var/log/syslog"
      class="path-input"
      onkeydown={(e) => { if (e.key === 'Enter') loadLog() }}
    />
    <input
      type="number"
      bind:value={lineCount}
      min="10"
      max="10000"
      class="count-input"
      title="Lines to show"
    />
    <button class="action-btn" onclick={loadLog} disabled={loading}>
      {loading ? '...' : 'Load'}
    </button>
    <button class="action-btn" class:active={autoRefresh} onclick={toggleAutoRefresh}>
      Auto
    </button>
  </div>

  <div class="search-bar">
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="Search in log..."
      class="search-input"
    />
    {#if searchQuery}
      <span class="match-count">{matchCount} matches</span>
    {/if}
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="log-content">
    {#each filtered as line, i}
      <div class="log-line" class:highlight={searchQuery && line.toLowerCase().includes(searchQuery.toLowerCase())}>
        <span class="line-num">{i + 1}</span>
        <span class="line-text">{line}</span>
      </div>
    {/each}

    {#if lines.length === 0 && !loading}
      <div class="empty">
        {error ? '' : 'No log data — enter a path and click Load'}
      </div>
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1f2b;
  }

  .toolbar {
    display: flex;
    gap: 6px;
    padding: 8px 12px;
    border-bottom: 1px solid #2e303a;
    align-items: center;
  }

  .path-input {
    flex: 1;
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 4px;
    padding: 5px 10px;
    color: #e0e0e0;
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    outline: none;
  }

  .path-input:focus {
    border-color: #4a90d9;
  }

  .count-input {
    width: 60px;
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 4px;
    padding: 5px 6px;
    color: #e0e0e0;
    font-size: 12px;
    outline: none;
    text-align: center;
  }

  .action-btn {
    background: transparent;
    border: 1px solid #2e303a;
    color: #9ca3af;
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .action-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.active {
    background: #4a90d922;
    border-color: #4a90d9;
    color: #4a90d9;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-bottom: 1px solid #2e303a;
  }

  .search-input {
    flex: 1;
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 4px;
    padding: 4px 10px;
    color: #e0e0e0;
    font-size: 12px;
    outline: none;
  }

  .search-input:focus {
    border-color: #4a90d9;
  }

  .match-count {
    font-size: 11px;
    color: #6b7280;
    white-space: nowrap;
  }

  .error {
    background: #e06c7522;
    color: #e06c75;
    padding: 6px 12px;
    font-size: 12px;
  }

  .log-content {
    flex: 1;
    overflow: auto;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
  }

  .log-line {
    display: flex;
    gap: 12px;
    padding: 1px 12px;
    line-height: 1.5;
  }

  .log-line:hover {
    background: #2a2a3e;
  }

  .log-line.highlight {
    background: #4a90d912;
  }

  .line-num {
    color: #4a4a5a;
    user-select: none;
    min-width: 40px;
    text-align: right;
    flex-shrink: 0;
  }

  .line-text {
    color: #c0c0c0;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .empty {
    text-align: center;
    color: #6b7280;
    padding: 24px;
    font-size: 13px;
  }
</style>
