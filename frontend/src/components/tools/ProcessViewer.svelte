<script lang="ts">
  import { onDestroy } from 'svelte'
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import { getCached, setCache } from '$lib/utils/toolCache'

  export interface ProcessEntry {
    pid: string
    ppid: string
    user: string
    cpu: string
    mem: string
    vsz: string
    rss: string
    tty: string
    stat: string
    start: string
    time: string
    command: string
  }

  let { sessionId }: { sessionId: string } = $props()

  let processes = $state<ProcessEntry[]>([])
  let loading = $state(false)
  let error = $state('')
  let filter = $state('')
  let sortBy = $state<keyof ProcessEntry>('cpu')
  let sortDir = $state<'asc' | 'desc'>('desc')
  let refreshRate = $state<number>(0) // 0 = paused, 3 = 3s, 5 = 5s, 10 = 10s
  let refreshTimer: ReturnType<typeof setInterval> | null = null

  // Kill modal state
  let targetProcess = $state<ProcessEntry | null>(null)
  let killSignal = $state<string>('15') // 15 = SIGTERM, 9 = SIGKILL, 1 = SIGHUP
  let killing = $state(false)

  $effect(() => {
    if (sessionId) {
      const cached = getCached<ProcessEntry[]>(`${sessionId}:processes`)
      if (cached && cached.length > 0) {
        processes = cached
      } else {
        refresh()
      }
    }
  })

  $effect(() => {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
    if (refreshRate > 0 && sessionId) {
      refreshTimer = setInterval(() => {
        if (!loading) refresh()
      }, refreshRate * 1000)
    }
  })

  onDestroy(() => {
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
  })

  async function refresh() {
    if (!sessionId || loading) return
    loading = true
    error = ''
    try {
      const result = await api.remoteExec(
        sessionId,
        'ps -eo pid,ppid,user,%cpu,%mem,vsz,rss,tty,stat,start,time,args --sort=-%cpu 2>/dev/null || ps -eo pid,ppid,user,%cpu,%mem,vsz,rss,tty,stat,start,time,args 2>/dev/null || ps aux 2>/dev/null',
        10,
      )
      processes = parsePsOutput(result.stdout)
      setCache(`${sessionId}:processes`, processes)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function parsePsOutput(output: string): ProcessEntry[] {
    return output
      .split('\n')
      .filter((l: string) => l.trim())
      .map((line: string) => {
        const trimmed = line.trim()
        if (trimmed.startsWith('PID') || trimmed.startsWith('USER')) return null
        const parts = trimmed.split(/\s+/)
        if (parts.length >= 11) {
          return {
            pid: parts[0],
            ppid: parts[1],
            user: parts[2],
            cpu: parts[3],
            mem: parts[4],
            vsz: parts[5],
            rss: parts[6],
            tty: parts[7],
            stat: parts[8],
            start: parts[9],
            time: parts[10],
            command: parts.slice(11).join(' ') || parts[10],
          }
        }
        return null
      })
      .filter((p): p is ProcessEntry => p !== null)
  }

  function openKillModal(p: ProcessEntry) {
    targetProcess = p
    killSignal = '15' // default to SIGTERM
  }

  function closeKillModal() {
    targetProcess = null
  }

  async function executeKill() {
    if (!targetProcess || !sessionId) return
    killing = true
    try {
      await api.remoteExec(sessionId, `kill -${killSignal} ${targetProcess.pid} 2>&1`, 5)
      targetProcess = null
      await refresh()
    } catch (e) {
      error = String(e)
    } finally {
      killing = false
    }
  }

  let filtered = $derived(
    processes.filter((p) => {
      if (!filter) return true
      const q = filter.toLowerCase()
      return (
        p.command.toLowerCase().includes(q) ||
        p.pid.includes(q) ||
        p.user.toLowerCase().includes(q)
      )
    }),
  )

  let sorted = $derived.by(() => {
    const list = [...filtered]
    const numericCols: (keyof ProcessEntry)[] = ['pid', 'ppid', 'cpu', 'mem', 'vsz', 'rss']
    const isNum = numericCols.includes(sortBy)

    list.sort((a, b) => {
      const av = a[sortBy] ?? ''
      const bv = b[sortBy] ?? ''
      if (isNum) {
        const an = parseFloat(av) || 0
        const bn = parseFloat(bv) || 0
        return sortDir === 'asc' ? an - bn : bn - an
      }
      const cmp = String(av).localeCompare(String(bv))
      return sortDir === 'asc' ? cmp : -cmp
    })
    return list
  })

  function toggleSort(col: keyof ProcessEntry) {
    if (sortBy === col) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc'
    } else {
      sortBy = col
      sortDir = ['cpu', 'mem', 'rss'].includes(col) ? 'desc' : 'asc'
    }
  }
</script>

<div class="process-viewer" role="region" aria-label="Process viewer">
  <div class="toolbar">
    <div class="search-wrap">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        bind:value={filter}
        placeholder={t('tools.filterProcesses') || 'Filter by name, PID, or user...'}
        class="filter-input"
      />
      {#if filter}
        <button class="clear-btn" onclick={() => (filter = '')} title="Clear search">✕</button>
      {/if}
    </div>

    <div class="auto-refresh-group">
      <span class="refresh-label">Auto:</span>
      <select bind:value={refreshRate} class="refresh-select" title="Auto-refresh interval">
        <option value={0}>Paused</option>
        <option value={3}>3s</option>
        <option value={5}>5s</option>
        <option value={10}>10s</option>
      </select>
    </div>

    <button class="action-btn" onclick={refresh} disabled={loading} title="Refresh process table">
      {#if loading}
        <span class="spin">↻</span> Loading...
      {:else}
        <span>↻</span> {t('tools.refresh') || 'Refresh'}
      {/if}
    </button>

    <div class="stats-badge">
      <span>{sorted.length}</span> / <span>{processes.length}</span> procs
    </div>
  </div>

  {#if error}
    <div class="error-banner">
      <span>⚠️ {error}</span>
      <button class="dismiss-btn" onclick={() => (error = '')}>✕</button>
    </div>
  {/if}

  <div class="table-container">
    <table>
      <thead>
        <tr>
          <th class="sortable col-pid" onclick={() => toggleSort('pid')}>
            PID {sortBy === 'pid' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable col-user" onclick={() => toggleSort('user')}>
            USER {sortBy === 'user' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable col-cpu" onclick={() => toggleSort('cpu')}>
            CPU% {sortBy === 'cpu' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable col-mem" onclick={() => toggleSort('mem')}>
            MEM% {sortBy === 'mem' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable col-rss" onclick={() => toggleSort('rss')}>
            RSS {sortBy === 'rss' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable col-stat" onclick={() => toggleSort('stat')}>
            STAT {sortBy === 'stat' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable col-time" onclick={() => toggleSort('time')}>
            TIME {sortBy === 'time' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="sortable col-cmd" onclick={() => toggleSort('command')}>
            COMMAND {sortBy === 'command' ? (sortDir === 'asc' ? '▲' : '▼') : ''}
          </th>
          <th class="col-action"></th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as p (p.pid)}
          <tr>
            <td class="mono pid-cell">{p.pid}</td>
            <td class="user-cell">{p.user}</td>
            <td class="mono num-cell" class:highlight-high={parseFloat(p.cpu) > 50}>{p.cpu}%</td>
            <td class="mono num-cell" class:highlight-high={parseFloat(p.mem) > 50}>{p.mem}%</td>
            <td class="mono num-cell text-muted">{p.rss}</td>
            <td><span class="stat-badge">{p.stat}</span></td>
            <td class="mono text-muted">{p.time}</td>
            <td class="cmd-cell" title={p.command}>{p.command}</td>
            <td class="action-cell">
              <button
                type="button"
                class="kill-btn"
                onclick={() => openKillModal(p)}
                title={`Terminate PID ${p.pid}`}
              >
                Kill
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if sorted.length === 0 && !loading}
      <div class="empty-state">
        <p>{processes.length === 0 ? 'No process data available' : 'No processes match your filter'}</p>
      </div>
    {/if}
  </div>
</div>

<!-- Kill Process Modal -->
{#if targetProcess}
  <div class="modal-backdrop" onclick={closeKillModal} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal-dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h3>Terminate Process</h3>
        <button class="close-btn" onclick={closeKillModal}>✕</button>
      </div>
      <div class="modal-body">
        <div class="process-info-box">
          <div><strong>PID:</strong> <span class="mono">{targetProcess.pid}</span></div>
          <div><strong>User:</strong> {targetProcess.user}</div>
          <div><strong>CPU / MEM:</strong> {targetProcess.cpu}% / {targetProcess.mem}%</div>
          <div class="cmd-preview"><strong>Command:</strong> <code>{targetProcess.command}</code></div>
        </div>

        <div class="signal-selector">
          <label class="signal-label" for="kill-signal-select">Signal to Send:</label>
          <select id="kill-signal-select" bind:value={killSignal} class="signal-select">
            <option value="15">SIGTERM (15) — Graceful Termination</option>
            <option value="9">SIGKILL (9) — Force Kill Immediately</option>
            <option value="1">SIGHUP (1) — Hangup / Reload Configuration</option>
            <option value="2">SIGINT (2) — Interrupt (Ctrl+C)</option>
          </select>
        </div>
      </div>
      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" onclick={closeKillModal} disabled={killing}>
          Cancel
        </button>
        <button
          type="button"
          class="btn btn-danger"
          onclick={executeKill}
          disabled={killing}
        >
          {killing ? 'Sending...' : `Send Signal`}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .process-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary, #15181e);
    color: var(--text-primary, #e2e8f0);
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    gap: 10px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    align-items: center;
    background: var(--bg-secondary, #1a1e27);
    flex-shrink: 0;
  }

  .search-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 6px;
    padding: 0 8px;
    height: 28px;
  }

  .search-icon {
    font-size: 11px;
    color: var(--text-muted, #64748b);
    margin-right: 6px;
  }

  .filter-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary, #e2e8f0);
    font-size: 12px;
    outline: none;
  }

  .clear-btn {
    background: none;
    border: none;
    color: var(--text-muted, #64748b);
    cursor: pointer;
    font-size: 10px;
    padding: 2px 4px;
  }

  .auto-refresh-group {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-secondary, #94a3b8);
  }

  .refresh-select {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    border-radius: 4px;
    padding: 3px 6px;
    font-size: 11px;
    outline: none;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.12));
    color: var(--text-primary, #e2e8f0);
    padding: 4px 10px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
    height: 28px;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--bg-hover, #232834);
    border-color: var(--accent-primary, #6366f1);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .stats-badge {
    font-size: 11px;
    color: var(--text-muted, #64748b);
    white-space: nowrap;
  }

  .stats-badge span {
    font-weight: 600;
    color: var(--text-secondary, #94a3b8);
  }

  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
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

  .table-container {
    flex: 1;
    overflow: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    text-align: left;
  }

  th {
    padding: 7px 10px;
    background: var(--bg-secondary, #1a1e27);
    color: var(--text-secondary, #94a3b8);
    font-weight: 600;
    font-size: 11px;
    letter-spacing: 0.5px;
    position: sticky;
    top: 0;
    z-index: 2;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    user-select: none;
    white-space: nowrap;
  }

  th.sortable {
    cursor: pointer;
  }

  th.sortable:hover {
    color: var(--text-primary, #ffffff);
  }

  td {
    padding: 4px 10px;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.05));
    white-space: nowrap;
  }

  tr:hover td {
    background: var(--bg-hover, rgba(255, 255, 255, 0.04));
  }

  .mono {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 11px;
  }

  .pid-cell {
    font-weight: 600;
    color: var(--accent-primary, #818cf8);
    width: 70px;
  }

  .user-cell {
    width: 90px;
  }

  .num-cell {
    width: 60px;
    text-align: right;
  }

  .text-muted {
    color: var(--text-muted, #64748b);
  }

  .highlight-high {
    color: #f87171;
    font-weight: 600;
  }

  .stat-badge {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary, #cbd5e1);
  }

  .cmd-cell {
    max-width: 400px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
    color: var(--text-primary, #e2e8f0);
  }

  .action-cell {
    width: 60px;
    text-align: right;
  }

  .kill-btn {
    background: transparent;
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #f87171;
    padding: 2px 7px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 10px;
    font-weight: 600;
    opacity: 0;
    transition: all 0.15s ease;
  }

  tr:hover .kill-btn {
    opacity: 1;
  }

  .kill-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    border-color: #ef4444;
  }

  .empty-state {
    text-align: center;
    color: var(--text-muted, #64748b);
    padding: 32px;
    font-size: 13px;
  }

  .spin {
    display: inline-block;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  /* Modal Dialog */
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-dialog {
    background: var(--bg-secondary, #1a1e27);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.15));
    border-radius: 8px;
    width: 460px;
    max-width: 90vw;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
  }

  .modal-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #f87171;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted, #64748b);
    cursor: pointer;
    font-size: 12px;
  }

  .modal-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .process-info-box {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    padding: 10px 12px;
    font-size: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .cmd-preview {
    word-break: break-all;
    max-height: 80px;
    overflow-y: auto;
  }

  .cmd-preview code {
    color: #93c5fd;
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .signal-selector {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .signal-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary, #94a3b8);
  }

  .signal-select {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.15));
    border-radius: 6px;
    color: var(--text-primary, #ffffff);
    padding: 8px 10px;
    font-size: 12px;
    outline: none;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 12px 16px;
    border-top: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    background: rgba(0, 0, 0, 0.15);
  }

  .btn {
    padding: 6px 14px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: all 0.15s ease;
  }

  .btn-secondary {
    background: var(--bg-primary, #232834);
    color: var(--text-secondary, #cbd5e1);
  }

  .btn-secondary:hover {
    background: #2d3342;
    color: #ffffff;
  }

  .btn-danger {
    background: #dc2626;
    color: #ffffff;
  }

  .btn-danger:hover {
    background: #b91c1c;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
