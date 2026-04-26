<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let { sessionId }: { sessionId: string } = $props()

  let sysInfo = $state('')
  let loading = $state(false)
  let error = $state('')

  $effect(() => {
    loadInfo()
  })

  async function loadInfo() {
    loading = true
    error = ''
    try {
      const result = await api.remoteExec(
        sessionId,
        `echo "=== OS ===" && cat /etc/os-release 2>/dev/null || uname -a && echo "=== KERNEL ===" && uname -r && echo "=== UPTIME ===" && uptime && echo "=== CPU ===" && lscpu 2>/dev/null | head -20 && echo "=== MEMORY ===" && free -h 2>/dev/null && echo "=== DISK ===" && df -h 2>/dev/null && echo "=== NETWORK ===" && hostname -I 2>/dev/null`,
        15,
      )
      sysInfo = result.stdout
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }
</script>

<div class="sysinfo-panel">
  <div class="toolbar">
    <button class="action-btn" onclick={loadInfo} disabled={loading}>
      {loading ? t('common.loading') : t('tools.refresh')}
    </button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="info-content">
    {#each sysInfo.split('\n') as line, i}
      <div class="info-line" class:header={line.startsWith('===') && line.endsWith('===')}>
        {#if line.startsWith('===') && line.endsWith('===')}
          <span class="section-title">{line.replace(/===/g, '').trim()}</span>
        {:else}
          <span class="line-text">{line}</span>
        {/if}
      </div>
    {/each}

    {#if !sysInfo && !loading && !error}
      <div class="empty">{t('tools.clickRefresh')}</div>
    {/if}
  </div>
</div>

<style>
  .sysinfo-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary);
  }

  .toolbar {
    display: flex;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-primary);
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

  .info-content {
    flex: 1;
    overflow: auto;
    padding: 8px 0;
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
  }

  .info-line {
    padding: 1px 12px;
  }

  .info-line:hover {
    background: var(--bg-hover);
  }

  .info-line.header {
    padding: 8px 12px 4px;
  }

  .section-title {
    color: var(--accent);
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .line-text {
    color: var(--text-primary);
    white-space: pre;
  }

  .empty {
    text-align: center;
    color: var(--text-tertiary);
    padding: 24px;
    font-size: 13px;
  }
</style>
