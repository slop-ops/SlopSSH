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
    background: #1e1f2b;
    border: 1px solid #2e303a;
    border-radius: 8px;
    width: 90%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #2e303a;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 14px;
    color: #e0e0e0;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: #6b7280;
    cursor: pointer;
    font-size: 14px;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .dialog-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
  }
</style>
