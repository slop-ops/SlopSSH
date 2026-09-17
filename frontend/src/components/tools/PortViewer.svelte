<script lang="ts">
  import { onDestroy } from 'svelte'
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import { getCached, setCache } from '$lib/utils/toolCache'

  interface PortEntry {
    proto: string
    state: string
    localIp: string
    port: string
    peerIp: string
    peerPort: string
    processName: string
    pid: string
  }

  let { sessionId }: { sessionId: string } = $props()

  let ports = $state<PortEntry[]>([])
  let loading = $state(false)
  let error = $state('')
  let filter = $state('')
  let protoFilter = $state<'ALL' | 'TCP' | 'UDP'>('ALL')
  let stateFilter = $state<'ALL' | 'LISTEN' | 'ESTABLISHED'>('ALL')
  let autoRefreshSecs = $state<number>(0) // 0 = paused
  let refreshTimer: ReturnType<typeof setInterval> | null = null

  // Kill PID modal
  let killTarget = $state<{ pid: string; process: string; port: string } | null>(null)
  let killing = $state(false)

  $effect(() => {
    if (sessionId) {
      const cached = getCached<PortEntry[]>(`${sessionId}:ports`)
      if (cached && cached.length > 0) {
        ports = cached
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
    if (autoRefreshSecs > 0 && sessionId) {
      refreshTimer = setInterval(() => {
        if (!loading) refresh()
      }, autoRefreshSecs * 1000)
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
      const cmd = `ss -tulpn 2>/dev/null || netstat -tulpn 2>/dev/null || lsof -i -P -n 2>/dev/null`
      const result = await api.remoteExec(sessionId, cmd, 8)
      ports = parsePortOutput(result.stdout)
      setCache(`${sessionId}:ports`, ports)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function parsePortOutput(output: string): PortEntry[] {
    const lines = output.split('\n').filter((l) => l.trim())
    if (lines.length === 0) return []

    const firstLine = lines[0].trim()
    if (firstLine.startsWith('Netid') || firstLine.startsWith('State')) {
      return parseSs(lines)
    } else if (firstLine.startsWith('Active Internet') || firstLine.startsWith('Proto')) {
      return parseNetstat(lines)
    } else if (firstLine.startsWith('COMMAND') && firstLine.includes('PID')) {
      return parseLsof(lines)
    }

    // Default heuristic
    return parseSs(lines)
  }

  function parseSs(lines: string[]): PortEntry[] {
    const entries: PortEntry[] = []
    for (const line of lines) {
      const trimmed = line.trim()
      if (trimmed.startsWith('Netid') || trimmed.startsWith('Recv-Q')) continue
      const parts = trimmed.split(/\s+/)
      // Typical: [Netid, State, Recv-Q, Send-Q, Local Address:Port, Peer Address:Port, Process?]
      if (parts.length >= 5) {
        const proto = parts[0].toUpperCase()
        const state = parts[1]
        const local = parts[4] || ''
        const peer = parts[5] || ''
        const processPart = parts.slice(6).join(' ')

        const { ip: localIp, port } = splitIpPort(local)
        const { ip: peerIp, port: peerPort } = splitIpPort(peer)
        const { processName, pid } = extractProcessInfo(processPart)

        entries.push({
          proto,
          state,
          localIp,
          port,
          peerIp,
          peerPort,
          processName,
          pid,
        })
      }
    }
    return entries
  }

  function parseNetstat(lines: string[]): PortEntry[] {
    const entries: PortEntry[] = []
    for (const line of lines) {
      const trimmed = line.trim()
      if (trimmed.startsWith('Active') || trimmed.startsWith('Proto')) continue
      const parts = trimmed.split(/\s+/)
      // Proto Recv-Q Send-Q Local_Address Foreign_Address State? PID/Program
      if (parts.length >= 4) {
        const proto = parts[0].toUpperCase()
        const local = parts[3] || ''
        const peer = parts[4] || ''
        let state = 'LISTEN'
        let procStr = ''

        if (proto.includes('UDP')) {
          state = 'UNCONN'
          procStr = parts.slice(5).join(' ')
        } else {
          state = parts[5] || 'LISTEN'
          procStr = parts.slice(6).join(' ')
        }

        const { ip: localIp, port } = splitIpPort(local)
        const { ip: peerIp, port: peerPort } = splitIpPort(peer)
        const { processName, pid } = extractProcessInfo(procStr)

        entries.push({
          proto,
          state,
          localIp,
          port,
          peerIp,
          peerPort,
          processName,
          pid,
        })
      }
    }
    return entries
  }

  function parseLsof(lines: string[]): PortEntry[] {
    const entries: PortEntry[] = []
    for (const line of lines) {
      const trimmed = line.trim()
      if (trimmed.startsWith('COMMAND')) continue
      const parts = trimmed.split(/\s+/)
      // COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
      if (parts.length >= 9) {
        const processName = parts[0]
        const pid = parts[1]
        const proto = (parts[7] || 'TCP').toUpperCase()
        const namePart = parts[8] || ''
        const stateMatch = line.match(/\((LISTEN|ESTABLISHED)\)/)
        const state = stateMatch ? stateMatch[1] : (proto.includes('UDP') ? 'UNCONN' : 'LISTEN')

        const cleanName = namePart.replace(/\(.*?\)/, '').trim()
        const { ip: localIp, port } = splitIpPort(cleanName)

        entries.push({
          proto,
          state,
          localIp,
          port,
          peerIp: '*',
          peerPort: '*',
          processName,
          pid,
        })
      }
    }
    return entries
  }

  function splitIpPort(addressWithPort: string): { ip: string; port: string } {
    if (!addressWithPort || addressWithPort === '*') return { ip: '*', port: '*' }

    // Handle IPv6 like [::1]:80 or :::80
    const lastColon = addressWithPort.lastIndexOf(':')
    if (lastColon !== -1) {
      const ip = addressWithPort.substring(0, lastColon) || '*'
      const port = addressWithPort.substring(lastColon + 1)
      return { ip, port }
    }
    return { ip: addressWithPort, port: '' }
  }

  function extractProcessInfo(raw: string): { processName: string; pid: string } {
    if (!raw || raw === '-') return { processName: '-', pid: '' }

    // Match ss format: users:(("sshd",pid=1234,fd=3))
    const ssPid = raw.match(/pid=(\d+)/)
    const ssName = raw.match(/"([^"]+)"/)
    if (ssPid || ssName) {
      return {
        processName: ssName ? ssName[1] : '-',
        pid: ssPid ? ssPid[1] : '',
      }
    }

    // Match netstat format: 1234/nginx
    const netstatMatch = raw.match(/^(\d+)\/(.+)$/)
    if (netstatMatch) {
      return {
        pid: netstatMatch[1],
        processName: netstatMatch[2],
      }
    }

    return { processName: raw, pid: '' }
  }

  async function killProcess(pid: string, sig: number = 15) {
    if (!sessionId) return
    killing = true
    try {
      await api.remoteExec(sessionId, `kill -${sig} ${pid} 2>&1`, 5)
      killTarget = null
      await refresh()
    } catch (e) {
      error = String(e)
    } finally {
      killing = false
    }
  }

  let filteredPorts = $derived(
    ports.filter((p) => {
      // Protocol filter
      if (protoFilter !== 'ALL' && !p.proto.includes(protoFilter)) return false

      // State filter
      if (stateFilter === 'LISTEN' && !p.state.includes('LISTEN')) return false
      if (stateFilter === 'ESTABLISHED' && !p.state.includes('ESTAB')) return false

      // Text search
      if (!filter) return true
      const q = filter.toLowerCase()
      return (
        p.port.includes(q) ||
        p.localIp.toLowerCase().includes(q) ||
        p.processName.toLowerCase().includes(q) ||
        p.pid.includes(q) ||
        p.proto.toLowerCase().includes(q)
      )
    }),
  )
</script>

<div class="port-viewer" role="region" aria-label="Port viewer">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="filter-group">
      <span class="label">Proto:</span>
      <div class="btn-toggle-group">
        <button
          type="button"
          class="toggle-btn"
          class:active={protoFilter === 'ALL'}
          onclick={() => (protoFilter = 'ALL')}
        >
          All
        </button>
        <button
          type="button"
          class="toggle-btn"
          class:active={protoFilter === 'TCP'}
          onclick={() => (protoFilter = 'TCP')}
        >
          TCP
        </button>
        <button
          type="button"
          class="toggle-btn"
          class:active={protoFilter === 'UDP'}
          onclick={() => (protoFilter = 'UDP')}
        >
          UDP
        </button>
      </div>
    </div>

    <div class="filter-group">
      <span class="label">State:</span>
      <div class="btn-toggle-group">
        <button
          type="button"
          class="toggle-btn"
          class:active={stateFilter === 'ALL'}
          onclick={() => (stateFilter = 'ALL')}
        >
          All
        </button>
        <button
          type="button"
          class="toggle-btn"
          class:active={stateFilter === 'LISTEN'}
          onclick={() => (stateFilter = 'LISTEN')}
        >
          Listen
        </button>
        <button
          type="button"
          class="toggle-btn"
          class:active={stateFilter === 'ESTABLISHED'}
          onclick={() => (stateFilter = 'ESTABLISHED')}
        >
          Estab
        </button>
      </div>
    </div>

    <div class="search-wrap">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        bind:value={filter}
        placeholder="Filter port, IP, process..."
        class="search-input"
      />
      {#if filter}
        <button class="clear-btn" onclick={() => (filter = '')}>✕</button>
      {/if}
    </div>

    <div class="auto-refresh-wrap">
      <span class="label">Auto:</span>
      <select bind:value={autoRefreshSecs} class="auto-select">
        <option value={0}>Paused</option>
        <option value={5}>5s</option>
        <option value={10}>10s</option>
      </select>
    </div>

    <button
      type="button"
      class="action-btn"
      onclick={refresh}
      disabled={loading}
      title="Refresh listening ports"
    >
      {#if loading}
        <span class="spin">↻</span>
      {:else}
        <span>↻</span> Refresh
      {/if}
    </button>

    <div class="count-badge">
      <span>{filteredPorts.length}</span> ports
    </div>
  </div>

  {#if error}
    <div class="error-banner">
      <span>⚠️ {error}</span>
      <button class="dismiss-btn" onclick={() => (error = '')}>✕</button>
    </div>
  {/if}

  <!-- Data Table -->
  <div class="table-container">
    <table class="port-table">
      <thead>
        <tr>
          <th class="col-proto">Proto</th>
          <th class="col-port">Port</th>
          <th class="col-ip">Local Address</th>
          <th class="col-peer">Foreign Address</th>
          <th class="col-state">State</th>
          <th class="col-proc">Process</th>
          <th class="col-pid">PID</th>
          <th class="col-action"></th>
        </tr>
      </thead>
      <tbody>
        {#each filteredPorts as p (p.proto + p.localIp + p.port + p.pid)}
          <tr>
            <td>
              <span class="proto-tag" class:tcp={p.proto.includes('TCP')} class:udp={p.proto.includes('UDP')}>
                {p.proto}
              </span>
            </td>
            <td class="mono port-cell">
              <strong>{p.port || '-'}</strong>
            </td>
            <td class="mono ip-cell">{p.localIp}:{p.port}</td>
            <td class="mono peer-cell text-muted">{p.peerIp}:{p.peerPort}</td>
            <td>
              <span
                class="state-tag"
                class:state-listen={p.state.includes('LISTEN')}
                class:state-estab={p.state.includes('ESTAB')}
              >
                {p.state}
              </span>
            </td>
            <td class="proc-cell" title={p.processName}>
              <strong>{p.processName}</strong>
            </td>
            <td class="mono text-muted">{p.pid || '-'}</td>
            <td class="action-cell">
              {#if p.pid}
                <button
                  type="button"
                  class="kill-btn"
                  onclick={() => killTarget = { pid: p.pid, process: p.processName, port: p.port }}
                  title={`Stop process ${p.processName} (PID ${p.pid})`}
                >
                  Kill
                </button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if filteredPorts.length === 0 && !loading}
      <div class="empty-state">
        <p>{ports.length === 0 ? 'No open or listening ports found' : 'No ports match your current filter'}</p>
      </div>
    {/if}
  </div>
</div>

<!-- Kill Confirmation Dialog -->
{#if killTarget}
  <div class="modal-backdrop" onclick={() => (killTarget = null)} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal-dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h3>Terminate Port Process</h3>
        <button class="close-btn" onclick={() => (killTarget = null)}>✕</button>
      </div>
      <div class="modal-body">
        <p>Are you sure you want to stop the process listening on port <strong>{killTarget.port}</strong>?</p>
        <div class="info-box">
          <div><strong>Process:</strong> <code>{killTarget.process}</code></div>
          <div><strong>PID:</strong> <span class="mono">{killTarget.pid}</span></div>
        </div>
      </div>
      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" onclick={() => (killTarget = null)} disabled={killing}>
          Cancel
        </button>
        <button
          type="button"
          class="btn btn-warning"
          onclick={() => killProcess(killTarget!.pid, 15)}
          disabled={killing}
        >
          SIGTERM (15)
        </button>
        <button
          type="button"
          class="btn btn-danger"
          onclick={() => killProcess(killTarget!.pid, 9)}
          disabled={killing}
        >
          Force Kill (9)
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .port-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary, #15181e);
    color: var(--text-primary, #e2e8f0);
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: var(--bg-secondary, #1a1e27);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    flex-shrink: 0;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
  }

  .label {
    color: var(--text-secondary, #94a3b8);
  }

  .btn-toggle-group {
    display: flex;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 5px;
    padding: 2px;
    gap: 2px;
  }

  .toggle-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary, #94a3b8);
    padding: 2px 7px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .toggle-btn:hover {
    color: #ffffff;
  }

  .toggle-btn.active {
    background: var(--accent-primary, #6366f1);
    color: #ffffff;
  }

  .search-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 5px;
    padding: 0 8px;
    height: 26px;
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

  .auto-refresh-wrap {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
  }

  .auto-select {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 11px;
    outline: none;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.12));
    color: var(--text-primary, #e2e8f0);
    padding: 3px 9px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 11px;
    height: 26px;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--bg-hover, #232834);
    border-color: var(--accent-primary, #6366f1);
  }

  .count-badge {
    font-size: 11px;
    color: var(--text-muted, #64748b);
    white-space: nowrap;
  }

  .count-badge span {
    color: var(--text-secondary, #94a3b8);
    font-weight: 600;
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

  .table-container {
    flex: 1;
    overflow: auto;
  }

  .port-table {
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
    position: sticky;
    top: 0;
    z-index: 2;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    user-select: none;
    white-space: nowrap;
  }

  td {
    padding: 5px 10px;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.05));
    white-space: nowrap;
  }

  tr:hover td {
    background: var(--bg-hover, rgba(255, 255, 255, 0.04));
  }

  .mono {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .col-proto { width: 60px; }
  .col-port { width: 75px; }
  .col-ip { width: 180px; }
  .col-peer { width: 160px; }
  .col-state { width: 90px; }
  .col-proc { max-width: 180px; }
  .col-pid { width: 70px; }
  .col-action { width: 60px; text-align: right; }

  .proto-tag {
    font-size: 10px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.08);
  }

  .proto-tag.tcp {
    background: rgba(99, 102, 241, 0.2);
    color: #818cf8;
  }

  .proto-tag.udp {
    background: rgba(245, 158, 11, 0.2);
    color: #fbbf24;
  }

  .port-cell {
    color: #38bdf8;
  }

  .text-muted {
    color: var(--text-muted, #64748b);
  }

  .state-tag {
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-secondary, #94a3b8);
  }

  .state-tag.state-listen {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    font-weight: 600;
  }

  .state-tag.state-estab {
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
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
    padding: 36px;
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

  /* Modal */
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
    width: 420px;
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
    font-size: 13px;
  }

  .info-box {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    padding: 10px 12px;
    font-size: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    background: rgba(0, 0, 0, 0.15);
  }

  .btn {
    padding: 6px 12px;
    border-radius: 5px;
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

  .btn-warning {
    background: #d97706;
    color: #ffffff;
  }

  .btn-danger {
    background: #dc2626;
    color: #ffffff;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
