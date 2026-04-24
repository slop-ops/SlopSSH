<script lang="ts">
  import * as api from '$lib/api/invoke'

  let { sessionId }: { sessionId: string } = $props()

  let ports = $state<any[]>([])
  let loading = $state(false)
  let error = $state('')
  let filter = $state('')

  $effect(() => {
    refresh()
  })

  async function refresh() {
    loading = true
    error = ''
    try {
      const result = await api.remoteExec(
        sessionId,
        `ss -tlnp 2>/dev/null || netstat -tlnp 2>/dev/null`,
        15,
      )
      ports = parsePorts(result.stdout)
    } catch (e) {
      error = String(e)
      ports = []
    } finally {
      loading = false
    }
  }

  function parsePorts(output: string): any[] {
    return output
      .split('\n')
      .filter((l: string) => l.trim())
      .slice(1)
      .map((line: string) => {
        const parts = line.trim().split(/\s+/)
        if (parts.length >= 4) {
          const local = parts[3] || ''
          const portMatch = local.match(/:(\d+)$/)
          return {
            proto: parts[0] || '',
            state: parts[1] || '',
            recvQ: parts[2] || '0',
            sendQ: parts[3] ? '' : '0',
            local: local,
            port: portMatch ? portMatch[1] : '',
            process: parts.slice(5).join(' ') || '-',
          }
        }
        return null
      })
      .filter(Boolean) as any[]
  }

  let filtered = $derived(
    ports.filter(
      (p) =>
        !filter ||
        p.local.includes(filter) ||
        p.port.includes(filter) ||
        p.process.toLowerCase().includes(filter.toLowerCase()),
    ),
  )
</script>

<div class="port-viewer">
  <div class="toolbar">
    <input
      type="text"
      bind:value={filter}
      placeholder="Filter ports..."
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
          <th>Proto</th>
          <th>State</th>
          <th>Local Address</th>
          <th>Port</th>
          <th>Process</th>
        </tr>
      </thead>
      <tbody>
        {#each filtered as p (p.local)}
          <tr>
            <td class="mono">{p.proto}</td>
            <td><span class="state-badge">{p.state}</span></td>
            <td class="mono">{p.local}</td>
            <td class="mono port">{p.port}</td>
            <td class="process">{p.process}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if filtered.length === 0 && !loading}
      <div class="empty">{ports.length === 0 ? 'No port data' : 'No matches'}</div>
    {/if}
  </div>
</div>

<style>
  .port-viewer {
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

  .port {
    color: #61afef;
    font-weight: 600;
  }

  .state-badge {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    background: #2e303a;
    color: #9ca3af;
  }

  .process {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .empty {
    text-align: center;
    color: #6b7280;
    padding: 24px;
    font-size: 13px;
  }
</style>
