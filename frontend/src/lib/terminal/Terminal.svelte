<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Terminal } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import { WebglAddon } from '@xterm/addon-webgl'
  import { listen } from '@tauri-apps/api/event'
  import { darkTheme } from '$lib/terminal/themes'
  import * as api from '$lib/api/invoke'
  import '@xterm/xterm/css/xterm.css'

  let { sessionId, channelId = crypto.randomUUID() } = $props()

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
    } catch {
      // WebGL not available, fall back to canvas
    }

    terminal.open(terminalEl)
    fitAddon.fit()

    terminal.onData(async (data: string) => {
      if (!connected) return
      try {
        const encoded = btoa(data)
        await api.sshWriteShell(sessionId, channelId, encoded)
      } catch (e) {
        console.error('Failed to write to shell:', e)
      }
    })

    terminal.onResize(({ cols, rows }) => {
      if (connected) {
        api.sshResizeShell(sessionId, channelId, cols, rows).catch(console.error)
      }
    })

    unlisten = await listen<string>(`terminal-output-${channelId}`, (event) => {
      const decoded = atob(event.payload)
      terminal?.write(decoded)
    })

    const { cols, rows } = terminal
    try {
      await api.sshOpenShell(sessionId, channelId, cols, rows)
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

<div class="terminal-container" bind:this={terminalEl}></div>

<style>
  .terminal-container {
    width: 100%;
    height: 100%;
    min-height: 200px;
  }
</style>
