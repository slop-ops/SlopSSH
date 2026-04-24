<script lang="ts">
  interface MenuItem {
    label: string
    action?: string
    separator?: boolean
    disabled?: boolean
  }

  let {
    items,
    x,
    y,
    onaction,
    onclose,
  }: {
    items: MenuItem[]
    x: number
    y: number
    onaction: (action: string) => void
    onclose: () => void
  } = $props()

  let menuEl: HTMLDivElement | undefined = $state()

  $effect(() => {
    function handleClick(e: MouseEvent) {
      if (menuEl && !menuEl.contains(e.target as Node)) {
        onclose()
      }
    }
    function handleKey(e: KeyboardEvent) {
      if (e.key === 'Escape') onclose()
    }
    setTimeout(() => {
      document.addEventListener('mousedown', handleClick)
      document.addEventListener('keydown', handleKey)
    }, 0)
    return () => {
      document.removeEventListener('mousedown', handleClick)
      document.removeEventListener('keydown', handleKey)
    }
  })

  let adjustedX = $derived(x)
  let adjustedY = $derived(y)

  $effect(() => {
    if (!menuEl) return
    const rect = menuEl.getBoundingClientRect()
    if (rect.right > window.innerWidth) {
      adjustedX = x - rect.width
    }
    if (rect.bottom > window.innerHeight) {
      adjustedY = y - rect.height
    }
  })
</script>

<div
  class="context-menu"
  bind:this={menuEl}
  style="left: {adjustedX}px; top: {adjustedY}px"
>
  {#each items as item}
    {#if item.separator}
      <div class="separator"></div>
    {:else}
      <button
        class="menu-item"
        disabled={item.disabled}
        onclick={() => {
          if (item.action) {
            onaction(item.action)
            onclose()
          }
        }}
      >
        {item.label}
      </button>
    {/if}
  {/each}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 1000;
    background: var(--bg-elevated);
    border: 1px solid var(--border-primary);
    border-radius: 6px;
    padding: 4px 0;
    min-width: 180px;
    box-shadow: var(--shadow-lg);
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-primary);
    padding: 6px 16px;
    font-size: 13px;
    font-family: inherit;
    cursor: pointer;
  }

  .menu-item:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .menu-item:disabled {
    color: var(--text-tertiary);
    cursor: default;
  }

  .separator {
    height: 1px;
    background: var(--border-primary);
    margin: 4px 8px;
  }
</style>
