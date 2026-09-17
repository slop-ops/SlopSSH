<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import { getCached, setCache } from '$lib/utils/toolCache'

  interface PartitionEntry {
    filesystem: string
    size: string
    used: string
    avail: string
    usePercent: number
    mount: string
  }

  interface FolderEntry {
    size: string
    path: string
  }

  let { sessionId }: { sessionId: string } = $props()

  // State
  let partitions = $state<PartitionEntry[]>([])
  let folderUsage = $state<FolderEntry[]>([])
  let loadingPartitions = $state(false)
  let loadingFolders = $state(false)
  let error = $state('')
  let selectedPath = $state('/')
  let activeTab = $state<'partitions' | 'folders'>('partitions')
  let pathFilter = $state('')

  $effect(() => {
    if (sessionId) {
      const cachedParts = getCached<PartitionEntry[]>(`${sessionId}:partitions`)
      if (cachedParts && cachedParts.length > 0) {
        partitions = cachedParts
      } else {
        refreshPartitions()
      }
    }
  })

  async function refreshPartitions() {
    if (!sessionId || loadingPartitions) return
    loadingPartitions = true
    error = ''
    try {
      const result = await api.remoteExec(sessionId, 'df -hP 2>/dev/null', 5)
      partitions = parseDfOutput(result.stdout)
      setCache(`${sessionId}:partitions`, partitions)
    } catch (e) {
      error = String(e)
    } finally {
      loadingPartitions = false
    }
  }

  function parseDfOutput(output: string): PartitionEntry[] {
    return output
      .split('\n')
      .filter((l) => l.trim())
      .slice(1) // skip header
      .map((line) => {
        const parts = line.trim().split(/\s+/)
        if (parts.length >= 6) {
          const pctStr = parts[4].replace('%', '')
          return {
            filesystem: parts[0],
            size: parts[1],
            used: parts[2],
            avail: parts[3],
            usePercent: parseInt(pctStr, 10) || 0,
            mount: parts[5],
          }
        }
        return null
      })
      .filter((p): p is PartitionEntry => p !== null)
  }

  async function analyzeFolder(path: string) {
    if (!sessionId || loadingFolders) return
    selectedPath = path
    activeTab = 'folders'
    loadingFolders = true
    error = ''
    try {
      // 5-second explicit timeout for folder drilldown
      const safePath = path.replace(/"/g, '\\"')
      const cmd = `du -h --max-depth=1 "${safePath}" 2>/dev/null | sort -rh | head -50`
      const result = await api.remoteExec(sessionId, cmd, 5)
      folderUsage = result.stdout
        .split('\n')
        .filter((l) => l.trim())
        .map((line) => {
          const match = line.match(/^([\d.]+[KMGTPE]?)\s+(.+)$/)
          if (match) {
            return { size: match[1], path: match[2] }
          }
          return null
        })
        .filter((d): d is FolderEntry => d !== null)
      setCache(`${sessionId}:folders:${path}`, folderUsage)
    } catch (e) {
      error = `Folder scan for "${path}" timed out or failed: ${e}`
      folderUsage = []
    } finally {
      loadingFolders = false
    }
  }

  function drillDown(entry: FolderEntry) {
    if (entry.path !== selectedPath) {
      analyzeFolder(entry.path)
    }
  }

  function navigateUp() {
    if (selectedPath === '/' || !selectedPath) return
    const segments = selectedPath.split('/').filter(Boolean)
    segments.pop()
    const parent = '/' + segments.join('/')
    analyzeFolder(parent || '/')
  }

  function parseSizeToBytes(sizeStr: string): number {
    const match = sizeStr.match(/^([\d.]+)([KMGTPE]?)$/)
    if (!match) return 0
    const val = parseFloat(match[1])
    const unit = match[2]
    const multipliers: Record<string, number> = {
      '': 1,
      K: 1024,
      M: 1024 ** 2,
      G: 1024 ** 3,
      T: 1024 ** 4,
      P: 1024 ** 5,
      E: 1024 ** 6,
    }
    return val * (multipliers[unit] || 1)
  }

  function folderBarWidth(sizeStr: string): string {
    if (folderUsage.length === 0) return '0%'
    // entry 0 is usually the folder itself, entry 1 is the largest subfolder
    const maxEntry = folderUsage[1] || folderUsage[0]
    const maxBytes = parseSizeToBytes(maxEntry?.size || '0')
    if (maxBytes === 0) return '0%'
    const curBytes = parseSizeToBytes(sizeStr)
    return Math.min(100, Math.max(3, (curBytes / maxBytes) * 100)) + '%'
  }

  let filteredPartitions = $derived(
    partitions.filter(
      (p) =>
        !pathFilter ||
        p.mount.toLowerCase().includes(pathFilter.toLowerCase()) ||
        p.filesystem.toLowerCase().includes(pathFilter.toLowerCase()),
    ),
  )

  let filteredFolders = $derived(
    folderUsage.filter(
      (f) => !pathFilter || f.path.toLowerCase().includes(pathFilter.toLowerCase()),
    ),
  )
</script>

<div class="disk-analyzer" role="region" aria-label="Disk analyzer">
  <!-- Top Navigation & Actions -->
  <div class="toolbar">
    <div class="tab-toggle-group">
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === 'partitions'}
        onclick={() => (activeTab = 'partitions')}
      >
        <span>💾 Partitions ({partitions.length})</span>
      </button>
      <button
        type="button"
        class="tab-btn"
        class:active={activeTab === 'folders'}
        onclick={() => {
          activeTab = 'folders'
          if (folderUsage.length === 0) analyzeFolder(selectedPath)
        }}
      >
        <span>📁 Folder Drilldown</span>
      </button>
    </div>

    {#if activeTab === 'folders'}
      <div class="folder-nav-group">
        <button
          type="button"
          class="nav-btn"
          onclick={navigateUp}
          disabled={selectedPath === '/' || loadingFolders}
          title="Go Up"
        >
          ⬆ Up
        </button>
        <input
          type="text"
          bind:value={selectedPath}
          class="path-input"
          onkeydown={(e) => { if (e.key === 'Enter') analyzeFolder(selectedPath) }}
          placeholder="/path/to/analyze"
        />
        <button
          type="button"
          class="nav-btn primary"
          onclick={() => analyzeFolder(selectedPath)}
          disabled={loadingFolders}
        >
          {loadingFolders ? 'Scanning...' : 'Scan (5s max)'}
        </button>
      </div>
    {/if}

    <div class="spacer"></div>

    <input
      type="text"
      bind:value={pathFilter}
      placeholder="Filter..."
      class="filter-input"
    />

    <button
      type="button"
      class="action-btn"
      onclick={() => (activeTab === 'partitions' ? refreshPartitions() : analyzeFolder(selectedPath))}
      disabled={loadingPartitions || loadingFolders}
      title="Refresh"
    >
      ↻
    </button>
  </div>

  {#if error}
    <div class="error-banner">
      <span>⚠️ {error}</span>
      <button class="dismiss-btn" onclick={() => (error = '')}>✕</button>
    </div>
  {/if}

  <div class="content-container">
    {#if activeTab === 'partitions'}
      <!-- Partitions Table (df -hP) -->
      <table class="data-table">
        <thead>
          <tr>
            <th>Mount Point</th>
            <th>Filesystem</th>
            <th class="num-col">Total</th>
            <th class="num-col">Used</th>
            <th class="num-col">Avail</th>
            <th class="bar-col">Usage</th>
            <th class="action-col">Action</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredPartitions as p (p.mount)}
            <tr>
              <td class="mount-cell">
                <span class="mount-icon">💾</span>
                <strong>{p.mount}</strong>
              </td>
              <td class="fs-cell text-muted mono">{p.filesystem}</td>
              <td class="mono num-col">{p.size}</td>
              <td class="mono num-col">{p.used}</td>
              <td class="mono num-col text-avail">{p.avail}</td>
              <td class="bar-col">
                <div class="usage-bar-track">
                  <div
                    class="usage-bar-fill"
                    class:level-high={p.usePercent > 85}
                    class:level-med={p.usePercent >= 70 && p.usePercent <= 85}
                    class:level-low={p.usePercent < 70}
                    style:width="{p.usePercent}%"
                  ></div>
                </div>
                <span class="pct-label mono">{p.usePercent}%</span>
              </td>
              <td class="action-col">
                <button
                  type="button"
                  class="drill-btn"
                  onclick={() => analyzeFolder(p.mount)}
                  title={`Inspect folder usage in ${p.mount}`}
                >
                  Drilldown
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>

      {#if filteredPartitions.length === 0 && !loadingPartitions}
        <div class="empty-state">No partitions match your filter</div>
      {/if}
    {:else}
      <!-- Folder Drilldown Table (du -h --max-depth=1) -->
      <div class="drilldown-header">
        <span class="drilldown-title">Folder: <code>{selectedPath}</code></span>
        {#if loadingFolders}
          <span class="loading-tag">⏳ Running capped 5-second scan...</span>
        {/if}
      </div>

      <table class="data-table">
        <thead>
          <tr>
            <th class="size-col">Size</th>
            <th class="folder-col">Path</th>
            <th class="bar-col">Relative Size</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredFolders as f (f.path)}
            <tr
              class="clickable-row"
              class:is-root={f.path === selectedPath}
              onclick={() => drillDown(f)}
            >
              <td class="mono size-col font-bold">{f.size}</td>
              <td class="folder-col">
                <span class="folder-icon">{f.path === selectedPath ? '📂' : '📁'}</span>
                <span class="folder-path mono">{f.path}</span>
              </td>
              <td class="bar-col">
                <div class="usage-bar-track">
                  <div
                    class="usage-bar-fill level-folder"
                    style:width={folderBarWidth(f.size)}
                  ></div>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>

      {#if filteredFolders.length === 0 && !loadingFolders}
        <div class="empty-state">
          <p>No folder usage data. Click "Scan" above to analyze <code>{selectedPath}</code>.</p>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .disk-analyzer {
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
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-secondary, #1a1e27);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    flex-shrink: 0;
  }

  .tab-toggle-group {
    display: flex;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 6px;
    padding: 2px;
    gap: 2px;
  }

  .tab-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary, #94a3b8);
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .tab-btn:hover {
    color: #ffffff;
  }

  .tab-btn.active {
    background: var(--accent-primary, #6366f1);
    color: #ffffff;
  }

  .folder-nav-group {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    max-width: 500px;
  }

  .nav-btn {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    white-space: nowrap;
  }

  .nav-btn.primary {
    background: var(--accent-primary, #6366f1);
    border-color: var(--accent-primary, #6366f1);
    color: #ffffff;
    font-weight: 500;
  }

  .nav-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .path-input {
    flex: 1;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-family: 'JetBrains Mono', monospace;
    outline: none;
  }

  .spacer {
    flex: 1;
  }

  .filter-input {
    width: 140px;
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    outline: none;
  }

  .action-btn {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #e2e8f0);
    width: 28px;
    height: 28px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
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

  .content-container {
    flex: 1;
    overflow: auto;
  }

  .drilldown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-secondary, #1a1e27);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.05));
    font-size: 12px;
  }

  .drilldown-title code {
    color: #93c5fd;
    font-family: 'JetBrains Mono', monospace;
  }

  .loading-tag {
    color: #fbbf24;
    font-size: 11px;
  }

  .data-table {
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
  }

  td {
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.05));
  }

  .clickable-row {
    cursor: pointer;
  }

  .clickable-row:hover td {
    background: var(--bg-hover, rgba(255, 255, 255, 0.04));
  }

  .clickable-row.is-root td {
    background: rgba(99, 102, 241, 0.06);
  }

  .mount-cell {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .mount-icon {
    font-size: 13px;
  }

  .mono {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
  }

  .font-bold {
    font-weight: 600;
  }

  .text-muted {
    color: var(--text-muted, #64748b);
  }

  .text-avail {
    color: #34d399;
  }

  .num-col {
    width: 75px;
    text-align: right;
  }

  .size-col {
    width: 85px;
    text-align: right;
  }

  .bar-col {
    width: 180px;
  }

  .usage-bar-track {
    background: rgba(255, 255, 255, 0.08);
    height: 8px;
    border-radius: 4px;
    overflow: hidden;
    display: inline-block;
    width: 120px;
    vertical-align: middle;
    margin-right: 8px;
  }

  .usage-bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.3s ease;
  }

  .level-low {
    background: #10b981;
  }

  .level-med {
    background: #f59e0b;
  }

  .level-high {
    background: #ef4444;
  }

  .level-folder {
    background: #6366f1;
  }

  .pct-label {
    font-size: 11px;
    color: var(--text-secondary, #94a3b8);
  }

  .action-col {
    width: 80px;
    text-align: right;
  }

  .drill-btn {
    background: transparent;
    border: 1px solid rgba(99, 102, 241, 0.4);
    color: #818cf8;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 10px;
    cursor: pointer;
  }

  .drill-btn:hover {
    background: rgba(99, 102, 241, 0.15);
    border-color: #6366f1;
  }

  .folder-col {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .folder-icon {
    font-size: 12px;
  }

  .folder-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-state {
    text-align: center;
    color: var(--text-muted, #64748b);
    padding: 36px;
    font-size: 13px;
  }
</style>
