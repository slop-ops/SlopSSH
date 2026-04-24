<script lang="ts">
  import * as api from '$lib/api/invoke'

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
      placeholder="Filter processes..."
      class="filter-input"
    />
    <button class="action-btn" onclick={refresh} disabled={loading}>
      {loading ? '...' : 'Refresh'}
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
              <button class="kill-btn" onclick={() => killProcess(p.pid)}>Kill</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if filtered.length === 0 && !loading}
      <div class="empty">{processes.length === 0 ? 'No process data' : 'No matches'}</div>
    {/if}
  </div>
</div>

<style>
  .process-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1f2b;
  }

  .toolbar {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid #2e303a;
    align-items: center;
  }

  .filter-input {
    flex: 1;
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 4px;
    padding: 5px 10px;
    color: #e0e0e0;
    font-size: 12px;
    outline: none;
  }

  .filter-input:focus {
    border-color: #4a90d9;
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

  .error {
    background: #e06c7522;
    color: #e06c75;
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
    background: #16171d;
    color: #9ca3af;
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
    color: #e0e0e0;
  }

  td {
    padding: 4px 10px;
    color: #e0e0e0;
    border-bottom: 1px solid #2e303a11;
  }

  tr:hover td {
    background: #2a2a3e;
  }

  .mono {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .stat-badge {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    background: #2e303a;
    color: #9ca3af;
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
    border: 1px solid #e06c7544;
    color: #e06c75;
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
    background: #e06c7522;
  }

  .empty {
    text-align: center;
    color: #6b7280;
    padding: 24px;
    font-size: 13px;
  }
</style>
