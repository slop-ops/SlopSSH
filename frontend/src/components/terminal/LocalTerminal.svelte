<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Terminal } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import { WebglAddon } from '@xterm/addon-webgl'
  import { listen } from '@tauri-apps/api/event'
  import { darkTheme, lightTheme } from '$lib/terminal/themes'
  import { getTheme, getTerminalSettings } from '$lib/stores/theme.svelte'
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
  let contextmenuHandler: ((e: MouseEvent) => void) | undefined = $state()

  // Write batching for performance
  let writeQueue: string[] = []
  let writeScheduled = false

  function scheduleWrite(data: string) {
    writeQueue.push(data)
    if (!writeScheduled) {
      writeScheduled = true
      requestAnimationFrame(flushWrites)
    }
  }

  function flushWrites() {
    if (writeQueue.length > 0 && terminal) {
      const batch = writeQueue.join('')
      writeQueue = []
      writeScheduled = false
      terminal.write(batch)
    } else {
      writeScheduled = false
    }
  }

  function encodeBase64(str: string): string {
    const bytes = new TextEncoder().encode(str)
    let binary = ''
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i])
    }
    return btoa(binary)
  }

  function decodeBase64(b64: string): string {
    const binary = atob(b64)
    const bytes = new Uint8Array(binary.length)
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i)
    }
    return new TextDecoder().decode(bytes)
  }

  onMount(async () => {
    if (!terminalEl) return

    const settings = getTerminalSettings()
    const theme = getTheme() === 'light' ? lightTheme : darkTheme
    terminal = new Terminal({
      theme,
      fontFamily: settings.font_family || 'JetBrains Mono, monospace',
      fontSize: settings.font_size || 14,
      cursorBlink: true,
      scrollback: settings.terminal_scrollback || 10000,
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
      console.warn('WebGL addon failed, falling back to DOM renderer')
    }

    // Custom keyboard shortcuts
    terminal.attachCustomKeyEventHandler((e: KeyboardEvent) => {
      // Ctrl+Backspace → delete last word (sends Ctrl+W)
      if (e.ctrlKey && e.key === 'Backspace') {
        if (e.type === 'keydown') {
          terminal?.write('\x17')
        }
        return false
      }
      // Ctrl+Delete → delete word forward
      if (e.ctrlKey && e.key === 'Delete') {
        if (e.type === 'keydown') {
          terminal?.write('\x1b[3;5~')
        }
        return false
      }
      return true
    })

    terminal.open(terminalEl)
    fitAddon.fit()

    terminal.onData(async (data: string) => {
      if (!connected) return
      try {
        const encoded = encodeBase64(data)
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
      if (terminal?.hasSelection() && getTerminalSettings().terminal_copy_on_select) {
        const selection = terminal.getSelection()
        if (selection) {
          navigator.clipboard.writeText(selection).catch(() => {})
        }
      }
    })

    contextmenuHandler = (e) => {
      e.preventDefault()
      navigator.clipboard.readText().then((text) => {
        if (text && connected) {
          const encoded = encodeBase64(text)
          api.localTerminalWrite(channelId, encoded).catch(console.error)
        }
      }).catch(() => {})
    }
    terminalEl.addEventListener('contextmenu', contextmenuHandler)

    unlisten = await listen<string>(`terminal-output-${channelId}`, (event) => {
      const decoded = decodeBase64(event.payload)
      scheduleWrite(decoded)
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
    if (!terminalEl || terminalEl.offsetParent === null) return
    fitAddon?.fit()
  }

  onDestroy(() => {
    connected = false
    unlisten?.()
    if (contextmenuHandler && terminalEl) {
      terminalEl.removeEventListener('contextmenu', contextmenuHandler)
    }
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

  // Reactive theme switching
  $effect(() => {
    const themeName = getTheme()
    if (terminal) {
      terminal.options.theme = themeName === 'light' ? lightTheme : darkTheme
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
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .terminal-container {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
