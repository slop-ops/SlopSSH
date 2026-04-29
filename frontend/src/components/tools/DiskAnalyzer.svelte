<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let { sessionId }: { sessionId: string } = $props()

  interface DiskEntry {
    size: string
    path: string
  }

  let diskUsage = $state<DiskEntry[]>([])
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
        .filter((d): d is DiskEntry => d !== null)
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

<div class="disk-analyzer" role="region" aria-label="Disk analyzer">
  <div class="toolbar">
    <input
      type="text"
      bind:value={scanPath}
      class="path-input"
      onkeydown={(e) => { if (e.key === 'Enter') scan() }}
    />
    <button class="action-btn" onclick={scan} disabled={loading}>
      {loading ? '...' : t('tools.scan')}
    </button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="summary">
    <span>{t('tools.items', { count: String(diskUsage.length) })}</span>
    <span>{t('tools.total', { size: formatTotal(totalSize) })}</span>
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
        {error ? '' : t('tools.enterPath')}
      </div>
    {/if}
  </div>
</div>

<style>
  .disk-analyzer {
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

  .path-input {
    flex: 1;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 5px 10px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    outline: none;
  }

  .path-input:focus {
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

  .summary {
    display: flex;
    justify-content: space-between;
    padding: 6px 12px;
    font-size: 11px;
    color: var(--text-tertiary);
    border-bottom: 1px solid var(--border-primary);
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
    background: var(--bg-hover);
  }

  .disk-bar-container {
    height: 6px;
    background: var(--bg-hover);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 2px;
  }

  .disk-bar {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--info));
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
    color: var(--info);
    font-family: 'JetBrains Mono', monospace;
    min-width: 50px;
  }

  .disk-path {
    font-size: 11px;
    color: var(--text-secondary);
    font-family: 'JetBrains Mono', monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    text-align: center;
    color: var(--text-tertiary);
    padding: 24px;
    font-size: 13px;
  }
</style>
