<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

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
      placeholder={t('tools.filterPorts')}
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
      <div class="empty">{ports.length === 0 ? t('tools.noPortData') : t('tools.noMatches')}</div>
    {/if}
  </div>
</div>

<style>
  .port-viewer {
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

  .port {
    color: var(--info);
    font-weight: 600;
  }

  .state-badge {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .process {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .empty {
    text-align: center;
    color: var(--text-tertiary);
    padding: 24px;
    font-size: 13px;
  }
</style>
