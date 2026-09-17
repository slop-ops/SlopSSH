<script lang="ts">
  let {
    open = $bindable(false),
    title = 'Input Required',
    message = '',
    defaultValue = '',
    placeholder = '',
    confirmLabel = 'Submit',
    cancelLabel = 'Cancel',
    onconfirm,
    oncancel,
  }: {
    open?: boolean
    title?: string
    message?: string
    defaultValue?: string
    placeholder?: string
    confirmLabel?: string
    cancelLabel?: string
    onconfirm: (value: string) => void
    oncancel?: () => void
  } = $props()

  let inputValue = $state('')

  $effect(() => {
    if (open) {
      inputValue = defaultValue
    }
  })

  function handleConfirm() {
    if (!inputValue.trim()) return
    open = false
    onconfirm(inputValue.trim())
  }

  function handleCancel() {
    open = false
    oncancel?.()
  }
</script>

{#if open}
  <div class="modal-backdrop" onclick={handleCancel} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="modal-card"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="modal-header">
        <h3>{title}</h3>
        <button type="button" class="close-btn" onclick={handleCancel} aria-label="Close">✕</button>
      </div>

      <div class="modal-body">
        {#if message}
          <p class="message-text">{message}</p>
        {/if}
        <input
          type="text"
          bind:value={inputValue}
          {placeholder}
          class="prompt-input"
          onkeydown={(e) => {
            if (e.key === 'Enter') handleConfirm()
            if (e.key === 'Escape') handleCancel()
          }}
        />
      </div>

      <div class="modal-footer">
        <button type="button" class="btn btn-cancel" onclick={handleCancel}>
          {cancelLabel}
        </button>
        <button
          type="button"
          class="btn btn-primary"
          onclick={handleConfirm}
          disabled={!inputValue.trim()}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
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
    z-index: 10000;
  }

  .modal-card {
    background: var(--bg-secondary, #1a1e27);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.12));
    border-radius: 8px;
    width: 420px;
    max-width: 90vw;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
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
    color: var(--text-primary, #ffffff);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    font-size: 12px;
  }

  .modal-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .message-text {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary, #94a3b8);
  }

  .prompt-input {
    background: var(--bg-primary, #0f1218);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.15));
    border-radius: 5px;
    padding: 7px 10px;
    color: var(--text-primary, #ffffff);
    font-size: 13px;
    outline: none;
    width: 100%;
    box-sizing: border-box;
  }

  .prompt-input:focus {
    border-color: var(--accent-primary, #6366f1);
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
    padding: 6px 14px;
    border-radius: 5px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: all 0.15s ease;
  }

  .btn-cancel {
    background: var(--bg-tertiary, #232834);
    color: var(--text-secondary, #cbd5e1);
  }

  .btn-cancel:hover {
    background: #2d3342;
    color: #ffffff;
  }

  .btn-primary {
    background: var(--accent-primary, #6366f1);
    color: #ffffff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #4f46e5;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
