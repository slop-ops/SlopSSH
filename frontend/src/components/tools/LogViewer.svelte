<script lang="ts">
  import { onDestroy, tick } from 'svelte'
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import { getCached, setCache } from '$lib/utils/toolCache'

  let { sessionId }: { sessionId: string } = $props()

  let logPath = $state('/var/log/syslog')
  let lines = $state<string[]>([])
  let loading = $state(false)
  let error = $state('')
  let searchQuery = $state('')
  let lineCount = $state(200)
  let autoRefreshSecs = $state<number>(0)
  let refreshTimer: ReturnType<typeof setInterval> | null = null
  let autoScroll = $state(true)
  let logContainer: HTMLElement | null = $state(null)

  const PRESETS = [
    { label: 'System Log', path: '/var/log/syslog' },
    { label: 'Messages', path: '/var/log/messages' },
    { label: 'Auth / Security', path: '/var/log/auth.log' },
    { label: 'Kernel (dmesg)', path: '/var/log/dmesg' },
    { label: 'Nginx Access', path: '/var/log/nginx/access.log' },
    { label: 'Nginx Error', path: '/var/log/nginx/error.log' },
    { label: 'Apache Error', path: '/var/log/apache2/error.log' },
  ]

  $effect(() => {
    if (sessionId) {
      const cached = getCached<{ path: string; lines: string[]; count: number }>(`${sessionId}:log`)
      if (cached) {
        logPath = cached.path
        lines = cached.lines
        lineCount = cached.count
      } else {
        loadLog()
      }
    }
  })

  $effect(() => {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
    if (autoRefreshSecs > 0 && sessionId) {
      refreshTimer = setInterval(() => {
        if (!loading) loadLog()
      }, autoRefreshSecs * 1000)
    }
  })

  onDestroy(() => {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  })

  async function loadLog() {
    if (!logPath.trim() || !sessionId || loading) return
    loading = true
    error = ''
    try {
      const safePath = logPath.replace(/"/g, '\\"')
      const result = await api.remoteExec(
        sessionId,
        `tail -n ${lineCount} "${safePath}" 2>&1`,
        10,
      )
      const rawLines = result.stdout.split('\n')
      if (result.exitCode !== 0 && rawLines.length <= 1) {
        error = result.stdout
        lines = []
      } else {
        lines = rawLines
        setCache(`${sessionId}:log`, { path: logPath, lines, count: lineCount })
        if (autoScroll) {
          await tick()
          scrollToBottom()
        }
      }
    } catch (e) {
      error = String(e)
      lines = []
    } finally {
      loading = false
    }
  }

  function scrollToBottom() {
    if (logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight
    }
  }

  function applyPreset(path: string) {
    logPath = path
    loadLog()
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

<div class="log-viewer" role="region" aria-label="Log viewer">
  <!-- Top Bar -->
  <div class="toolbar">
    <div class="preset-dropdown">
      <select
        onchange={(e) => {
          const val = (e.target as HTMLSelectElement).value
          if (val) applyPreset(val)
        }}
        class="preset-select"
        title="Quick log presets"
      >
        <option value="">Quick Presets...</option>
        {#each PRESETS as p}
          <option value={p.path}>{p.label} ({p.path})</option>
        {/each}
      </select>
    </div>

    <input
      type="text"
      bind:value={logPath}
      placeholder="/var/log/syslog"
      class="path-input"
      onkeydown={(e) => { if (e.key === 'Enter') loadLog() }}
    />

    <div class="lines-group">
      <span class="lines-label">Lines:</span>
      <input
        type="number"
        bind:value={lineCount}
        min="10"
        max="10000"
        class="count-input"
        title="Number of lines to fetch"
      />
    </div>

    <div class="auto-refresh-group">
      <span class="lines-label">Auto:</span>
      <select bind:value={autoRefreshSecs} class="auto-select">
        <option value={0}>Paused</option>
        <option value={3}>3s</option>
        <option value={5}>5s</option>
        <option value={10}>10s</option>
      </select>
    </div>

    <button
      type="button"
      class="action-btn"
      class:active={autoScroll}
      onclick={() => (autoScroll = !autoScroll)}
      title="Auto-scroll to latest log entries"
    >
      ⬇ Scroll Lock
    </button>

    <button type="button" class="action-btn primary" onclick={loadLog} disabled={loading}>
      {loading ? '...' : 'Load'}
    </button>
  </div>

  <!-- Search & Filter Bar -->
  <div class="search-bar">
    <div class="search-box">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Filter logs by keyword..."
        class="search-input"
      />
      {#if searchQuery}
        <button type="button" class="clear-btn" onclick={() => (searchQuery = '')}>✕</button>
      {/if}
    </div>

    {#if searchQuery}
      <span class="match-count">{matchCount} matching lines</span>
    {/if}

    <div class="spacer"></div>
    <span class="total-count">{lines.length} lines loaded</span>
  </div>

  {#if error}
    <div class="error-banner">
      <span>⚠️ {error}</span>
      <button type="button" class="dismiss-btn" onclick={() => (error = '')}>✕</button>
    </div>
  {/if}

  <!-- Log Content Area -->
  <div class="log-content" bind:this={logContainer}>
    {#each filtered as line, i}
      <div
        class="log-line"
        class:highlight={searchQuery && line.toLowerCase().includes(searchQuery.toLowerCase())}
      >
        <span class="line-num">{i + 1}</span>
        <span class="line-text">{line}</span>
      </div>
    {/each}

    {#if lines.length === 0 && !loading}
      <div class="empty-state">
        <p>{error ? 'Failed to read log file' : 'No log lines retrieved. Check file path and permissions.'}</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary, #15181e);
    color: var(--text-primary, #e2e8f0);
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    align-items: center;
    background: var(--bg-secondary, #1a1e27);
    flex-shrink: 0;
  }

  .preset-select {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    outline: none;
    max-width: 140px;
  }

  .path-input {
    flex: 1;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary, #e2e8f0);
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    outline: none;
  }

  .lines-group,
  .auto-refresh-group {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-secondary, #94a3b8);
  }

  .count-input {
    width: 60px;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 4px;
    padding: 3px 6px;
    color: var(--text-primary, #e2e8f0);
    font-size: 11px;
    text-align: center;
    outline: none;
  }

  .auto-select {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    border-radius: 4px;
    padding: 3px 6px;
    font-size: 11px;
    outline: none;
  }

  .action-btn {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.12));
    color: var(--text-secondary, #94a3b8);
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    color: #ffffff;
    background: var(--bg-hover, #232834);
  }

  .action-btn.active {
    background: rgba(99, 102, 241, 0.2);
    border-color: #6366f1;
    color: #818cf8;
    font-weight: 500;
  }

  .action-btn.primary {
    background: var(--accent-primary, #6366f1);
    border-color: var(--accent-primary, #6366f1);
    color: #ffffff;
    font-weight: 500;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-secondary, #1a1e27);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.05));
    flex-shrink: 0;
  }

  .search-box {
    display: flex;
    align-items: center;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 4px;
    padding: 0 8px;
    width: 280px;
    height: 24px;
  }

  .search-icon {
    font-size: 11px;
    color: var(--text-muted, #64748b);
    margin-right: 6px;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary, #e2e8f0);
    font-size: 11px;
    outline: none;
  }

  .clear-btn {
    background: none;
    border: none;
    color: var(--text-muted, #64748b);
    cursor: pointer;
    font-size: 10px;
  }

  .match-count {
    font-size: 11px;
    color: #38bdf8;
  }

  .spacer {
    flex: 1;
  }

  .total-count {
    font-size: 11px;
    color: var(--text-muted, #64748b);
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(239, 68, 68, 0.15);
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    color: #fca5a5;
    padding: 6px 12px;
    font-size: 12px;
  }

  .dismiss-btn {
    background: none;
    border: none;
    color: #fca5a5;
    cursor: pointer;
  }

  .log-content {
    flex: 1;
    overflow-y: auto;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    background: var(--bg-primary, #0f1218);
  }

  .log-line {
    display: flex;
    gap: 12px;
    padding: 1px 12px;
    line-height: 1.5;
  }

  .log-line:hover {
    background: var(--bg-hover, rgba(255, 255, 255, 0.04));
  }

  .log-line.highlight {
    background: rgba(99, 102, 241, 0.2);
  }

  .line-num {
    color: var(--text-muted, #475569);
    user-select: none;
    min-width: 40px;
    text-align: right;
    flex-shrink: 0;
    font-size: 11px;
  }

  .line-text {
    color: var(--text-primary, #cbd5e1);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .empty-state {
    text-align: center;
    color: var(--text-muted, #64748b);
    padding: 36px;
    font-size: 13px;
  }
</style>
