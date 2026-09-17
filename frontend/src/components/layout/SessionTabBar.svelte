<script lang="ts">
  import { t } from '$lib/utils/i18n'

  export interface SessionTabItem {
    id: string
    title: string
    subtitle?: string
    status: 'connected' | 'reconnecting' | 'disconnected'
    isLocal?: boolean
  }

  let {
    sessions = [],
    activeSessionId = '',
    onSelectSession,
    onCloseSession,
    onNewSession,
    onOpenLocalTerminal,
  }: {
    sessions: SessionTabItem[]
    activeSessionId: string
    onSelectSession: (id: string) => void
    onCloseSession: (id: string) => void
    onNewSession: () => void
    onOpenLocalTerminal?: () => void
  } = $props()

  function handleKeyDown(e: KeyboardEvent) {
    if (e.altKey && !e.ctrlKey && !e.metaKey) {
      const num = parseInt(e.key, 10)
      if (!isNaN(num) && num >= 1 && num <= 9) {
        const targetIdx = num - 1
        if (targetIdx < sessions.length) {
          e.preventDefault()
          onSelectSession(sessions[targetIdx].id)
        }
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="session-tab-bar" role="tablist" aria-label="Session Tabs">
  <div class="tabs-scroll-container">
    {#each sessions as session, idx (session.id)}
      <div
        class="session-tab"
        class:active={activeSessionId === session.id}
        class:is-local={session.isLocal}
        class:disconnected={session.status === 'disconnected'}
        role="tab"
        tabindex={0}
        aria-selected={activeSessionId === session.id}
        onclick={() => onSelectSession(session.id)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelectSession(session.id) }}
        title={session.subtitle ? `${session.title} (${session.subtitle}) [Alt+${idx + 1}]` : `${session.title} [Alt+${idx + 1}]`}
      >
        <span
          class="status-indicator {session.isLocal ? 'local' : session.status}"
          aria-label={session.status}
        ></span>

        <span class="tab-label">
          <span class="tab-title">{session.title}</span>
          {#if session.subtitle}
            <span class="tab-subtitle">{session.subtitle}</span>
          {/if}
        </span>

        <button
          type="button"
          class="tab-close-btn"
          aria-label={t('sessionTab.closeSession')}
          onclick={(e) => {
            e.stopPropagation()
            onCloseSession(session.id)
          }}
        >
          &times;
        </button>
      </div>
    {/each}

    <div class="tab-actions">
      <button
        type="button"
        class="action-btn new-session-btn"
        onclick={onNewSession}
        title={t('toolbar.newSession')}
        aria-label={t('toolbar.newSession')}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>

      {#if onOpenLocalTerminal}
        <button
          type="button"
          class="action-btn local-terminal-btn"
          onclick={onOpenLocalTerminal}
          title={t('app.openLocalTerminal')}
          aria-label={t('app.openLocalTerminal')}
        >
          <span class="terminal-glyph">&gt;_</span>
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .session-tab-bar {
    display: flex;
    align-items: center;
    background: var(--bg-tertiary, #12141a);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    height: 38px;
    min-height: 38px;
    width: 100%;
    overflow: hidden;
    user-select: none;
  }

  .tabs-scroll-container {
    display: flex;
    align-items: flex-end;
    height: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    padding: 0 4px;
    gap: 2px;
    flex: 1;
  }

  .tabs-scroll-container::-webkit-scrollbar {
    display: none;
  }

  .session-tab {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 10px;
    background: var(--bg-secondary, rgba(255, 255, 255, 0.03));
    border: 1px solid transparent;
    border-bottom: none;
    border-top-left-radius: 6px;
    border-top-right-radius: 6px;
    color: var(--text-secondary, #94a3b8);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    max-width: 220px;
    transition: all 0.15s ease;
    position: relative;
  }

  .session-tab:hover {
    background: var(--bg-primary, rgba(255, 255, 255, 0.07));
    color: var(--text-primary, #f1f5f9);
  }

  .session-tab.active {
    background: var(--bg-primary, #1a1d24);
    color: var(--text-primary, #ffffff);
    border-color: var(--border-primary, rgba(255, 255, 255, 0.12));
    border-bottom-color: var(--bg-primary, #1a1d24);
    box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.25);
    z-index: 1;
  }

  .session-tab.active::after {
    content: '';
    position: absolute;
    top: 0;
    left: 4px;
    right: 4px;
    height: 2px;
    background: var(--accent-primary, #6366f1);
    border-radius: 2px 2px 0 0;
  }

  .session-tab.disconnected {
    opacity: 0.75;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-indicator.connected {
    background-color: #10b981;
    box-shadow: 0 0 6px rgba(16, 185, 129, 0.5);
  }

  .status-indicator.reconnecting {
    background-color: #f59e0b;
    box-shadow: 0 0 6px rgba(245, 158, 11, 0.5);
    animation: pulse 1.5s infinite;
  }

  .status-indicator.disconnected {
    background-color: #ef4444;
  }

  .status-indicator.local {
    background-color: #3b82f6;
    box-shadow: 0 0 6px rgba(59, 130, 246, 0.5);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.4; transform: scale(0.9); }
  }

  .tab-label {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-subtitle {
    font-size: 10px;
    color: var(--text-muted, #64748b);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-close-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--text-muted, #64748b);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0;
    transition: all 0.12s ease;
    margin-left: 2px;
  }

  .tab-close-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .tab-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: 6px;
    margin-bottom: 3px;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 5px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: var(--bg-secondary, rgba(255, 255, 255, 0.08));
    color: var(--text-primary, #ffffff);
    border-color: var(--border-primary, rgba(255, 255, 255, 0.1));
  }

  .terminal-glyph {
    font-family: monospace;
    font-size: 11px;
    font-weight: bold;
  }
</style>
