<script lang="ts">
  import { getToasts, dismissToast } from '$lib/stores/toast.svelte'

  let toasts = $derived(getToasts())
</script>

<div class="toast-container" role="region" aria-label="Notifications" aria-live="polite">
  {#each toasts as t (t.id)}
    <div class="toast-item {t.type}" role="alert">
      <div class="toast-icon">
        {#if t.type === 'success'}
          ✓
        {:else if t.type === 'error'}
          ✕
        {:else if t.type === 'warning'}
          ⚠
        {:else}
          ℹ
        {/if}
      </div>
      <div class="toast-text">{t.text}</div>
      <button
        type="button"
        class="toast-close-btn"
        onclick={() => dismissToast(t.id)}
        aria-label="Dismiss notification"
      >
        ✕
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 32px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 99999;
    pointer-events: none;
    max-width: 380px;
    width: calc(100% - 32px);
  }

  .toast-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-radius: 6px;
    font-size: 12px;
    line-height: 1.4;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    pointer-events: auto;
    animation: slideIn 0.2s ease-out;
    background: var(--bg-secondary, #1a1e27);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.12));
    color: var(--text-primary, #ffffff);
  }

  @keyframes slideIn {
    from {
      transform: translateY(16px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .toast-icon {
    font-size: 13px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .toast-text {
    flex: 1;
    word-break: break-word;
  }

  .toast-item.success {
    border-color: rgba(52, 211, 153, 0.5);
    background: #064e3b;
    color: #ecfdf5;
  }

  .toast-item.error {
    border-color: rgba(239, 68, 68, 0.5);
    background: #7f1d1d;
    color: #fef2f2;
  }

  .toast-item.warning {
    border-color: rgba(245, 158, 11, 0.5);
    background: #78350f;
    color: #fffbeb;
  }

  .toast-item.info {
    border-color: rgba(56, 189, 248, 0.5);
    background: #0c4a6e;
    color: #f0f9ff;
  }

  .toast-close-btn {
    background: none;
    border: none;
    color: inherit;
    opacity: 0.7;
    cursor: pointer;
    font-size: 11px;
    padding: 2px 4px;
    margin-left: 4px;
    border-radius: 3px;
    transition: opacity 0.15s ease;
  }

  .toast-close-btn:hover {
    opacity: 1;
    background: rgba(255, 255, 255, 0.1);
  }
</style>
