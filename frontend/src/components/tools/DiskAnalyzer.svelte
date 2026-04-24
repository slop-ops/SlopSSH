<script lang="ts">
  import * as api from '$lib/api/invoke'

  let { sessionId }: { sessionId: string } = $props()

  let diskUsage = $state<any[]>([])
  let loading = $state(false)
  let error = $state('')
  let scanPath = $state('/')
  let sortBy = $state<'size'>('size')

  $effect(() => {
    scan()
  })

  async function scan() {
    loading = true
    error = ''
    try {
      const result = await api.remoteExec(
        sessionId,
        `du -sh ${scanPath}/*/ ${scanPath}/*.??* 2>/dev/null | sort -rh | head -50`,
        30,
      )
      diskUsage = result.stdout
        .split('\n')
        .filter((l: string) => l.trim())
        .map((line: string) => {
          const match = line.match(/^([\d.]+[KMGTPE]?)\s+(.+)$/)
          if (match) {
            return { size: match[1], path: match[2] }
          }
          return null
        })
        .filter(Boolean) as any[]
    } catch (e) {
      error = String(e)
      diskUsage = []
    } finally {
      loading = false
    }
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

  let totalSize = $derived(
    diskUsage.reduce((sum, d) => sum + parseSizeToBytes(d.size), 0),
  )

  function formatTotal(bytes: number): string {
    if (bytes === 0) return '0 B'
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
  }

  function barWidth(sizeStr: string): string {
    if (diskUsage.length === 0) return '0%'
    const maxBytes = parseSizeToBytes(diskUsage[0]?.size || '0')
    if (maxBytes === 0) return '0%'
    return Math.max(2, (parseSizeToBytes(sizeStr) / maxBytes) * 100) + '%'
  }
</script>

<div class="disk-analyzer">
  <div class="toolbar">
    <input
      type="text"
      bind:value={scanPath}
      class="path-input"
      onkeydown={(e) => { if (e.key === 'Enter') scan() }}
    />
    <button class="action-btn" onclick={scan} disabled={loading}>
      {loading ? '...' : 'Scan'}
    </button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="summary">
    <span>{diskUsage.length} items</span>
    <span>Total: {formatTotal(totalSize)}</span>
  </div>

  <div class="disk-list">
    {#each diskUsage as item (item.path)}
      <div class="disk-item">
        <div class="disk-bar-container">
          <div class="disk-bar" style:width={barWidth(item.size)}></div>
        </div>
        <div class="disk-info">
          <span class="disk-size">{item.size}</span>
          <span class="disk-path" title={item.path}>{item.path}</span>
        </div>
      </div>
    {/each}

    {#if diskUsage.length === 0 && !loading}
      <div class="empty">
        {error ? '' : 'Enter a path and click Scan to analyze disk usage'}
      </div>
    {/if}
  </div>
</div>

<style>
  .disk-analyzer {
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

  .summary {
    display: flex;
    justify-content: space-between;
    padding: 6px 12px;
    font-size: 11px;
    color: #6b7280;
    border-bottom: 1px solid #2e303a;
  }

  .disk-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .disk-item {
    padding: 4px 12px;
  }

  .disk-item:hover {
    background: #2a2a3e;
  }

  .disk-bar-container {
    height: 6px;
    background: #2e303a;
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 2px;
  }

  .disk-bar {
    height: 100%;
    background: linear-gradient(90deg, #4a90d9, #61afef);
    border-radius: 3px;
    min-width: 2px;
  }

  .disk-info {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .disk-size {
    font-size: 11px;
    color: #61afef;
    font-family: 'JetBrains Mono', monospace;
    min-width: 50px;
  }

  .disk-path {
    font-size: 11px;
    color: #9ca3af;
    font-family: 'JetBrains Mono', monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    text-align: center;
    color: #6b7280;
    padding: 24px;
    font-size: 13px;
  }
</style>
