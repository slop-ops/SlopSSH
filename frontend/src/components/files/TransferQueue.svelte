<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { listen } from '@tauri-apps/api/event'

  interface TransferProgress {
    id: string
    bytes_transferred: number
    total_bytes: number
    status: string
    error: string | null
    speed_bps: number
  }

  let transfers = $state<TransferProgress[]>([])
  let expanded = $state(true)

  $effect(() => {
    loadTransfers()
    const poll = setInterval(loadTransfers, 2000)
    return () => clearInterval(poll)
  })

  async function loadTransfers() {
    try {
      const list = await api.transferList()
      transfers = Array.isArray(list) ? list : []
    } catch {
      transfers = []
    }
  }

  async function cancelTransfer(id: string) {
    try {
      await api.transferCancel(id)
      await loadTransfers()
    } catch (e) {
      console.error('Cancel failed:', e)
    }
  }

  async function clearCompleted() {
    try {
      await api.transferClearCompleted()
      await loadTransfers()
    } catch (e) {
      console.error('Clear failed:', e)
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const units = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
  }

  function formatSpeed(bps: number): string {
    return formatBytes(bps) + '/s'
  }

  function percent(t: TransferProgress): number {
    if (t.total_bytes === 0) return 0
    return Math.round((t.bytes_transferred / t.total_bytes) * 100)
  }

  let activeCount = $derived(
    transfers.filter((t) => t.status === 'InProgress' || t.status === 'Queued').length,
  )
  let completedCount = $derived(transfers.filter((t) => t.status === 'Completed').length)
</script>

{#if transfers.length > 0}
  <div class="transfer-queue" class:collapsed={!expanded}>
    <div class="header" onclick={() => (expanded = !expanded)}>
      <span class="title">
        Transfers
        {#if activeCount > 0}
          <span class="badge">{activeCount}</span>
        {/if}
      </span>
      <div class="header-actions">
        {#if completedCount > 0}
          <button class="clear-btn" onclick={(e: Event) => { e.stopPropagation(); clearCompleted() }}>Clear</button>
        {/if}
        <span class="expand-icon">{expanded ? 'v' : '^'}</span>
      </div>
    </div>

    {#if expanded}
      <div class="transfer-list">
        {#each transfers as t (t.id)}
          <div class="transfer-item" class:active={t.status === 'InProgress'}>
            <div class="transfer-info">
              <span class="transfer-id">{t.id.slice(0, 8)}</span>
              <span class="transfer-status" class:success={t.status === 'Completed'} class:failed={t.status === 'Failed'}>
                {t.status}
              </span>
            </div>
            <div class="progress-bar-container">
              <div class="progress-bar" style:width="{percent(t)}%"></div>
            </div>
            <div class="transfer-details">
              <span>{formatBytes(t.bytes_transferred)} / {formatBytes(t.total_bytes)}</span>
              {#if t.status === 'InProgress' && t.speed_bps > 0}
                <span class="speed">{formatSpeed(t.speed_bps)}</span>
              {/if}
            </div>
            {#if t.status === 'InProgress' || t.status === 'Queued'}
              <button class="cancel-btn" onclick={() => cancelTransfer(t.id)}>x</button>
            {/if}
            {#if t.error}
              <div class="transfer-error">{t.error}</div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .transfer-queue {
    border-top: 1px solid #2e303a;
    background: #16171d;
    max-height: 200px;
    display: flex;
    flex-direction: column;
  }

  .transfer-queue.collapsed {
    max-height: 36px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 12px;
    cursor: pointer;
    border-bottom: 1px solid #2e303a;
    flex-shrink: 0;
  }

  .title {
    font-size: 11px;
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .badge {
    background: #4a90d9;
    color: #fff;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 8px;
    text-transform: none;
    letter-spacing: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .clear-btn {
    background: none;
    border: none;
    color: #6b7280;
    font-size: 11px;
    cursor: pointer;
    padding: 2px 4px;
  }

  .clear-btn:hover {
    color: #e0e0e0;
  }

  .expand-icon {
    color: #6b7280;
    font-size: 10px;
  }

  .transfer-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .transfer-item {
    padding: 4px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    position: relative;
  }

  .transfer-item.active {
    background: #2a2a3e;
  }

  .transfer-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .transfer-id {
    font-size: 11px;
    color: #e0e0e0;
    font-family: 'JetBrains Mono', monospace;
  }

  .transfer-status {
    font-size: 10px;
    color: #6b7280;
  }

  .transfer-status.success {
    color: #98c379;
  }

  .transfer-status.failed {
    color: #e06c75;
  }

  .progress-bar-container {
    height: 3px;
    background: #2e303a;
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: #4a90d9;
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .transfer-details {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: #6b7280;
  }

  .speed {
    color: #98c379;
  }

  .cancel-btn {
    position: absolute;
    right: 8px;
    top: 4px;
    background: none;
    border: none;
    color: #6b7280;
    cursor: pointer;
    font-size: 10px;
    padding: 2px 4px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .transfer-item:hover .cancel-btn {
    opacity: 1;
  }

  .cancel-btn:hover {
    color: #e06c75;
  }

  .transfer-error {
    font-size: 10px;
    color: #e06c75;
    padding: 2px 0;
  }
</style>
