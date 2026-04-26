<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Terminal } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import { WebglAddon } from '@xterm/addon-webgl'
  import { listen } from '@tauri-apps/api/event'
  import { darkTheme, lightTheme } from '$lib/terminal/themes'
  import { getTheme, getTerminalSettings } from '$lib/stores/theme'
  import * as api from '$lib/api/invoke'
  import '@xterm/xterm/css/xterm.css'

  let {
    sessionId,
    channelId = crypto.randomUUID(),
    onSendSnippet,
  }: {
    sessionId: string
    channelId?: string
    onSendSnippet?: (handler: (cmd: string) => void) => void
  } = $props()

  let terminalEl: HTMLDivElement | undefined = $state()
  let terminal: Terminal | undefined = $state()
  let fitAddon: FitAddon | undefined = $state()
  let connected = $state(false)
  let disconnected = $state(false)
  let error = $state('')
  let unlisten: (() => void) | undefined = $state()

  function getTerminalOpts() {
    const settings = getTerminalSettings()
    const theme = getTheme() === 'light' ? lightTheme : darkTheme
    return {
      theme,
      fontFamily: settings.font_family || 'JetBrains Mono, monospace',
      fontSize: settings.font_size || 14,
      cursorBlink: true,
      scrollback: settings.terminal_scrollback || 10000,
    }
  }

  onMount(async () => {
    if (!terminalEl) return

    terminal = new Terminal(getTerminalOpts())

    fitAddon = new FitAddon()
    terminal.loadAddon(fitAddon)

    try {
      const webglAddon = new WebglAddon()
      webglAddon.onContextLoss(() => {
        webglAddon.dispose()
      })
      terminal.loadAddon(webglAddon)
    } catch {
    }

    terminal.open(terminalEl)
    fitAddon.fit()

    terminal.onData(async (data: string) => {
      if (!connected) return
      try {
        const encoded = btoa(data)
        await api.sshWriteShell(sessionId, channelId, encoded)
      } catch (e) {
        handleDisconnect(String(e))
      }
    })

    terminal.onResize(({ cols, rows }) => {
      if (connected) {
        api.sshResizeShell(sessionId, channelId, cols, rows).catch(console.error)
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

    terminalEl.addEventListener('contextmenu', (e) => {
      e.preventDefault()
      navigator.clipboard.readText().then((text) => {
        if (text && connected) {
          const encoded = btoa(text)
          api.sshWriteShell(sessionId, channelId, encoded).catch(console.error)
        }
      }).catch(() => {})
    })

    unlisten = await listen<string>(`terminal-output-${channelId}`, (event) => {
      const decoded = atob(event.payload)
      terminal?.write(decoded)
    })

    const { cols, rows } = terminal
    try {
      await api.sshOpenShell(sessionId, channelId, cols, rows)
      connected = true
      disconnected = false
      terminal.focus()
    } catch (e) {
      error = String(e)
      terminal.writeln(`\r\n\x1b[31mError: ${e}\x1b[0m`)
    }

    onSendSnippet?.(sendCommand)
  })

  function sendCommand(cmd: string) {
    if (!connected || !terminal) return
    const encoded = btoa(cmd + '\n')
    api.sshWriteShell(sessionId, channelId, encoded).catch(console.error)
  }

  function handleDisconnect(msg: string) {
    connected = false
    disconnected = true
    error = msg
    terminal?.writeln(`\r\n\x1b[31m--- Disconnected: ${msg}\x1b[0m`)
  }

  async function reconnect() {
    if (!terminal) return
    disconnected = false
    error = ''
    terminal.writeln('\r\n\x1b[33m--- Reconnecting...\x1b[0m')
    const { cols, rows } = terminal
    try {
      await api.sshOpenShell(sessionId, channelId, cols, rows)
      connected = true
      terminal.focus()
    } catch (e) {
      handleDisconnect(String(e))
    }
  }

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

<div class="terminal-wrapper">
  <div class="terminal-container" bind:this={terminalEl}></div>
  {#if disconnected}
    <div class="disconnect-overlay">
      <div class="disconnect-content">
        <p class="disconnect-msg">Connection lost</p>
        {#if error}
          <p class="disconnect-error">{error}</p>
        {/if}
        <button class="reconnect-btn" onclick={reconnect}>Reconnect</button>
      </div>
    </div>
  {/if}
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

  .disconnect-overlay {
    position: absolute;
    inset: 0;
    background: rgba(26, 26, 46, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    backdrop-filter: blur(2px);
  }

  .disconnect-content {
    text-align: center;
  }

  .disconnect-msg {
    color: var(--error);
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 8px 0;
  }

  .disconnect-error {
    color: var(--text-secondary);
    font-size: 12px;
    margin: 0 0 16px 0;
    max-width: 300px;
    word-break: break-word;
  }

  .reconnect-btn {
    background: var(--accent);
    border: none;
    color: #fff;
    padding: 8px 24px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-family: inherit;
  }

  .reconnect-btn:hover {
    background: var(--accent-hover);
  }
</style>
