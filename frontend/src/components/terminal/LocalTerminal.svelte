<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Terminal } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import { WebglAddon } from '@xterm/addon-webgl'
  import { listen } from '@tauri-apps/api/event'
  import { darkTheme } from '$lib/terminal/themes'
  import * as api from '$lib/api/invoke'
  import '@xterm/xterm/css/xterm.css'

  let {
    channelId = crypto.randomUUID(),
  }: {
    channelId?: string
  } = $props()

  let terminalEl: HTMLDivElement | undefined = $state()
  let terminal: Terminal | undefined = $state()
  let fitAddon: FitAddon | undefined = $state()
  let connected = $state(false)
  let unlisten: (() => void) | undefined = $state()

  onMount(async () => {
    if (!terminalEl) return

    terminal = new Terminal({
      theme: darkTheme,
      fontFamily: 'JetBrains Mono, monospace',
      fontSize: 14,
      cursorBlink: true,
      scrollback: 10000,
    })

    fitAddon = new FitAddon()
    terminal.loadAddon(fitAddon)

    try {
      const webglAddon = new WebglAddon()
      webglAddon.onContextLoss(() => {
        webglAddon.dispose()
      })
      terminal.loadAddon(webglAddon)
    } catch {}

    terminal.open(terminalEl)
    fitAddon.fit()

    terminal.onData(async (data: string) => {
      if (!connected) return
      try {
        const encoded = btoa(data)
        await api.localTerminalWrite(channelId, encoded)
      } catch (e) {
        console.error(e)
      }
    })

    terminal.onResize(({ cols, rows }) => {
      if (connected) {
        api.localTerminalResize(channelId, cols, rows).catch(console.error)
      }
    })

    terminal.onSelectionChange(() => {
      if (terminal?.hasSelection()) {
        const selection = terminal.getSelection()
        if (selection) {
          navigator.clipboard.writeText(selection).catch(() => {})
        }
      }
    })

    terminalEl.addEventListener('contextmenu', (e) => {
      e.preventDefault()
      navigator.clipboard.readText().then((text) => {
        if (text && connected) {
          const encoded = btoa(text)
          api.localTerminalWrite(channelId, encoded).catch(console.error)
        }
      }).catch(() => {})
    })

    unlisten = await listen<string>(`terminal-output-${channelId}`, (event) => {
      const decoded = atob(event.payload)
      terminal?.write(decoded)
    })

    const { cols, rows } = terminal
    try {
      await api.localTerminalOpen(channelId, cols, rows)
      connected = true
      terminal.focus()
    } catch (e) {
      terminal.writeln(`\r\n\x1b[31mError: ${e}\x1b[0m`)
    }
  })

  function handleResize() {
    fitAddon?.fit()
  }

  onDestroy(() => {
    connected = false
    unlisten?.()
    api.localTerminalClose(channelId).catch(() => {})
    terminal?.dispose()
  })

  $effect(() => {
    if (terminalEl && terminal) {
      const observer = new ResizeObserver(handleResize)
      observer.observe(terminalEl)
      return () => observer.disconnect()
    }
  })
</script>

<div class="terminal-wrapper">
  <div class="terminal-container" bind:this={terminalEl}></div>
</div>

<style>
  .terminal-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .terminal-container {
    width: 100%;
    height: 100%;
    min-height: 200px;
  }
</style>
