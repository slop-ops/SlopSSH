<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { t } from '$lib/utils/i18n'
  import type { SessionInfo } from '$lib/types'

  let {
    activeSession,
    activeSessionId,
    activeView = 'terminal',
  }: {
    activeSession?: { name?: string; host?: string; user?: string; port?: number } | null
    activeSessionId?: string | null
    activeView?: string
  } = $props()

  let currentTime = $state('')
  let timeTimer: ReturnType<typeof setInterval> | null = null

  function updateTime() {
    const now = new Date()
    currentTime = now.toLocaleTimeString(undefined, {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
  }

  onMount(() => {
    updateTime()
    timeTimer = setInterval(updateTime, 1000)
  })

  onDestroy(() => {
    if (timeTimer) clearInterval(timeTimer)
  })
</script>

<footer class="app-status-bar" role="status" aria-label="Status bar">
  <!-- Left: Connection / Host Status -->
  <div class="status-section left-section">
    {#if activeSessionId === '__local__'}
      <span class="status-dot dot-local"></span>
      <span class="session-info">Local Shell</span>
    {:else if activeSession}
      <span class="status-dot dot-online"></span>
      <span class="session-info mono">
        {activeSession.user || 'root'}@{activeSession.host}:{activeSession.port || 22}
      </span>
      {#if activeSession.name}
        <span class="session-tag">({activeSession.name})</span>
      {/if}
    {:else}
      <span class="status-dot dot-idle"></span>
      <span class="session-info text-muted">{t('app.noActiveSession') || 'No active session'}</span>
    {/if}
  </div>

  <!-- Center: Current View & Context -->
  <div class="status-section center-section">
    <span class="view-indicator">
      View: <strong>{activeView.toUpperCase()}</strong>
    </span>
  </div>

  <!-- Right: System Specs & Time -->
  <div class="status-section right-section">
    <span class="item charset-item">UTF-8</span>
    <span class="item time-item mono">{currentTime}</span>
  </div>
</footer>

<style>
  .app-status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    background: var(--bg-secondary, #1a1e27);
    border-top: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    padding: 0 10px;
    font-size: 11px;
    color: var(--text-secondary, #94a3b8);
    flex-shrink: 0;
    user-select: none;
  }

  .status-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .dot-online {
    background: #10b981;
    box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
  }

  .dot-local {
    background: #3b82f6;
    box-shadow: 0 0 6px rgba(59, 130, 246, 0.6);
  }

  .dot-idle {
    background: #64748b;
  }

  .session-info {
    color: var(--text-primary, #e2e8f0);
    font-size: 11px;
  }

  .session-tag {
    color: var(--text-muted, #64748b);
    font-size: 10px;
  }

  .mono {
    font-family: 'JetBrains Mono', monospace;
  }

  .text-muted {
    color: var(--text-muted, #64748b);
  }

  .view-indicator {
    color: var(--text-muted, #64748b);
    font-size: 10px;
    letter-spacing: 0.5px;
  }

  .view-indicator strong {
    color: var(--accent-primary, #818cf8);
    font-weight: 600;
  }

  .right-section {
    gap: 12px;
  }

  .charset-item {
    color: var(--text-muted, #64748b);
    font-size: 10px;
    background: rgba(255, 255, 255, 0.04);
    padding: 1px 4px;
    border-radius: 3px;
  }

  .time-item {
    color: var(--text-secondary, #94a3b8);
  }
</style>
