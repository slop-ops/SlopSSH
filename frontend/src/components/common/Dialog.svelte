<script lang="ts">
  let {
    open = $bindable(),
    title = '',
    width = '500px',
    children,
  }: {
    open: boolean
    title?: string
    width?: string
    children?: import('svelte').Snippet
  } = $props()

  function close() {
    open = false
  }

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) close()
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close()
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="backdrop" onclick={handleBackdrop} role="dialog" aria-modal="true">
    <div class="dialog" style:max-width={width}>
      {#if title}
        <div class="dialog-header">
          <h3>{title}</h3>
          <button class="close-btn" onclick={close}>x</button>
        </div>
      {/if}
      <div class="dialog-body">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: 8px;
    width: 90%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-primary);
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 14px;
    color: var(--text-primary);
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 14px;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .dialog-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
  }
</style>
