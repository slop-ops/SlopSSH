<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let { sessionId }: { sessionId: string } = $props()

  let processes = $state<any[]>([])
  let loading = $state(false)
  let error = $state('')
  let filter = $state('')
  let sortBy = $state('pid')
  let sortDir = $state<'asc' | 'desc'>('asc')

  $effect(() => {
    refresh()
  })

  async function refresh() {
    loading = true
    error = ''
    try {
      const result = await api.remoteExec(
        sessionId,
        'ps -eo pid,ppid,user,%cpu,%mem,vsz,rss,tty,stat,start,time,comm --no-headers 2>/dev/null || ps aux 2>/dev/null',
        15,
      )
      processes = parsePsOutput(result.stdout)
    } catch (e) {
      error = String(e)
      processes = []
    } finally {
      loading = false
    }
  }

  function parsePsOutput(output: string): any[] {
    return output
      .split('\n')
      .filter((l: string) => l.trim())
      .map((line: string) => {
        const parts = line.trim().split(/\s+/)
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
            command: parts.slice(11).join(' '),
          }
        }
        return null
      })
      .filter(Boolean)
  }

  async function killProcess(pid: string) {
    if (!confirm(`Kill process ${pid}?`)) return
    try {
      await api.remoteExec(sessionId, `kill -9 ${pid}`, 5)
      await refresh()
    } catch (e) {
      error = String(e)
    }
  }

  let filtered = $derived(
    processes.filter(
      (p) =>
        !filter ||
        p.command.toLowerCase().includes(filter.toLowerCase()) ||
        p.pid.includes(filter) ||
        p.user.toLowerCase().includes(filter.toLowerCase()),
    ),
  )

  let sorted = $derived(() => {
    const list = [...filtered]
    list.sort((a, b) => {
      const av = a[sortBy] ?? ''
      const bv = b[sortBy] ?? ''
      const cmp = typeof av === 'number' ? av - bv : String(av).localeCompare(String(bv))
      return sortDir === 'asc' ? cmp : -cmp
    })
    return list
  })

  function toggleSort(col: string) {
    if (sortBy === col) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc'
    } else {
      sortBy = col
      sortDir = 'asc'
    }
  }
</script>

<div class="process-viewer">
  <div class="toolbar">
    <input
      type="text"
      bind:value={filter}
      placeholder={t('tools.filterProcesses')}
      class="filter-input"
    />
    <button class="action-btn" onclick={refresh} disabled={loading}>
      {loading ? '...' : t('tools.refresh')}
    </button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="table-container">
    <table>
      <thead>
        <tr>
          <th class="sortable" onclick={() => toggleSort('pid')}>PID</th>
          <th class="sortable" onclick={() => toggleSort('user')}>User</th>
          <th class="sortable" onclick={() => toggleSort('cpu')}>CPU%</th>
          <th class="sortable" onclick={() => toggleSort('mem')}>MEM%</th>
          <th class="sortable" onclick={() => toggleSort('stat')}>Stat</th>
          <th class="sortable" onclick={() => toggleSort('command')}>Command</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each sorted() as p (p.pid)}
          <tr>
            <td class="mono">{p.pid}</td>
            <td>{p.user}</td>
            <td class="mono">{p.cpu}</td>
            <td class="mono">{p.mem}</td>
            <td><span class="stat-badge">{p.stat}</span></td>
            <td class="cmd" title={p.command}>{p.command}</td>
            <td>
              <button class="kill-btn" onclick={() => killProcess(p.pid)}>{t('tools.kill')}</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if filtered.length === 0 && !loading}
      <div class="empty">{processes.length === 0 ? t('tools.noProcessData') : t('tools.noMatches')}</div>
    {/if}
  </div>
</div>

<style>
  .process-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary);
  }

  .toolbar {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-primary);
    align-items: center;
  }

  .filter-input {
    flex: 1;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 5px 10px;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
  }

  .filter-input:focus {
    border-color: var(--border-active);
  }

  .action-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    background: var(--error-bg);
    color: var(--error);
    padding: 6px 12px;
    font-size: 12px;
  }

  .table-container {
    flex: 1;
    overflow: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  th {
    text-align: left;
    padding: 6px 10px;
    background: var(--bg-secondary);
    color: var(--text-secondary);
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    position: sticky;
    top: 0;
    z-index: 1;
  }

  th.sortable {
    cursor: pointer;
  }

  th.sortable:hover {
    color: var(--text-primary);
  }

  td {
    padding: 4px 10px;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border-primary);
  }

  tr:hover td {
    background: var(--bg-hover);
  }

  .mono {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .stat-badge {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .cmd {
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .kill-btn {
    background: none;
    border: 1px solid var(--error-bg);
    color: var(--error);
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 10px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  tr:hover .kill-btn {
    opacity: 1;
  }

  .kill-btn:hover {
    background: var(--error-bg);
  }

  .empty {
    text-align: center;
    color: var(--text-tertiary);
    padding: 24px;
    font-size: 13px;
  }
</style>
