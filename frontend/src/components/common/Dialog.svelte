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

  let dialogEl: HTMLDivElement | undefined = $state()
  let previousFocus: HTMLElement | null = null

  function close() {
    open = false
  }

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) close()
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close()
  }

  $effect(() => {
    if (open && dialogEl) {
      previousFocus = document.activeElement as HTMLElement

      const el = dialogEl
      const focusable = el.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      )
      const first = focusable[0]
      const last = focusable[focusable.length - 1]

      first?.focus()

      function trapFocus(e: KeyboardEvent) {
        if (e.key !== 'Tab') return
        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault()
            last?.focus()
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault()
            first?.focus()
          }
        }
      }

      el.addEventListener('keydown', trapFocus)
      return () => el.removeEventListener('keydown', trapFocus)
    }
  })

  $effect(() => {
    if (!open && previousFocus) {
      previousFocus.focus()
      previousFocus = null
    }
  })
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="backdrop" onclick={handleBackdrop} onkeydown={(e) => { if (e.key === 'Escape') close() }} role="dialog" aria-modal="true" tabindex={-1}>
    <div class="dialog" style:max-width={width} bind:this={dialogEl}>
      {#if title}
        <div class="dialog-header">
          <h3>{title}</h3>
          <button class="close-btn" onclick={close} aria-label="Close dialog">x</button>
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
