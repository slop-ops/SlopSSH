<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { Terminal } from '@xterm/xterm'
  import { FitAddon } from '@xterm/addon-fit'
  import { WebglAddon } from '@xterm/addon-webgl'
  import { SearchAddon } from '@xterm/addon-search'
  import { listen } from '@tauri-apps/api/event'
  import { getTheme, getTerminalSettings, getActiveXtermTheme } from '$lib/stores/theme.svelte'
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import '@xterm/xterm/css/xterm.css'

  let {
    sessionId,
    channelId = crypto.randomUUID(),
    isActive = true,
    onSendSnippet,
    onDisconnect,
  }: {
    sessionId: string
    channelId?: string
    isActive?: boolean
    onSendSnippet?: (handler: (cmd: string) => void) => void
    onDisconnect?: () => void
  } = $props()

  let terminalEl: HTMLDivElement | undefined = $state()
  let terminal: Terminal | undefined = $state()
  let fitAddon: FitAddon | undefined = $state()
  let searchAddon: SearchAddon | undefined = $state()
  let connected = $state(false)
  let disconnected = $state(false)
  let error = $state('')
  let unlisten: (() => void) | undefined = $state()
  let contextmenuHandler: ((e: MouseEvent) => void) | undefined = $state()

  let showSearch = $state(false)
  let searchQuery = $state('')
  let searchMatchCase = $state(false)
  let searchRegex = $state(false)

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

  function getTerminalOpts() {
    const settings = getTerminalSettings()
    const theme = getActiveXtermTheme()
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

    searchAddon = new SearchAddon()
    terminal.loadAddon(searchAddon)

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
      // Ctrl+F -> open search
      if (e.ctrlKey && (e.key === 'f' || e.key === 'F')) {
        if (e.type === 'keydown') {
          showSearch = true
        }
        return false
      }
      // Ctrl+Backspace → delete last word (sends Ctrl+W which most shells interpret)
      if (e.ctrlKey && e.key === 'Backspace') {
        if (e.type === 'keydown') {
          terminal?.write('\x17')
        }
        return false
      }
      // Ctrl+Delete → delete word forward (sends Alt+D)
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

    contextmenuHandler = (e) => {
      e.preventDefault()
      navigator.clipboard.readText().then((text) => {
        if (text && connected) {
          const encoded = encodeBase64(text)
          api.sshWriteShell(sessionId, channelId, encoded).catch(console.error)
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
    const encoded = encodeBase64(cmd + '\n')
    api.sshWriteShell(sessionId, channelId, encoded).catch(console.error)
  }

  function handleDisconnect(msg: string) {
    connected = false
    disconnected = true
    error = msg
    terminal?.writeln(`\r\n\x1b[31m--- Disconnected: ${msg}\x1b[0m`)
    onDisconnect?.()
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
    if (!terminalEl || terminalEl.offsetParent === null) return
    fitAddon?.fit()
  }

  function handleSearchNext() {
    if (!searchAddon || !searchQuery) return
    searchAddon.findNext(searchQuery, { caseSensitive: searchMatchCase, regex: searchRegex })
  }

  function handleSearchPrev() {
    if (!searchAddon || !searchQuery) return
    searchAddon.findPrevious(searchQuery, { caseSensitive: searchMatchCase, regex: searchRegex })
  }

  function handleSearchClose() {
    showSearch = false
    searchQuery = ''
    searchAddon?.clearDecorations()
    terminal?.focus()
  }

  function zoomIn() {
    if (!terminal) return
    const current = terminal.options.fontSize ?? 14
    terminal.options.fontSize = Math.min(32, current + 1)
    fitAddon?.fit()
  }

  function zoomOut() {
    if (!terminal) return
    const current = terminal.options.fontSize ?? 14
    terminal.options.fontSize = Math.max(9, current - 1)
    fitAddon?.fit()
  }

  function clearTerminal() {
    terminal?.clear()
  }

  // Reactive effect when tab becomes active
  $effect(() => {
    if (isActive && terminal && fitAddon && terminalEl) {
      requestAnimationFrame(() => {
        if (!terminalEl || terminalEl.offsetParent === null) return
        fitAddon?.fit()
        terminal?.focus()
        if (connected && terminal) {
          const { cols, rows } = terminal
          api.sshResizeShell(sessionId, channelId, cols, rows).catch(console.error)
        }
      })
    }
  })

  onDestroy(() => {
    connected = false
    unlisten?.()
    if (contextmenuHandler && terminalEl) {
      terminalEl.removeEventListener('contextmenu', contextmenuHandler)
    }
    api.sshCloseShell(sessionId, channelId).catch(() => {})
    terminal?.dispose()
  })

  // ResizeObserver for fit
  $effect(() => {
    if (terminalEl && terminal) {
      const observer = new ResizeObserver(handleResize)
      observer.observe(terminalEl)
      return () => observer.disconnect()
    }
  })

  // Reactive theme switching
  $effect(() => {
    getTheme()
    if (terminal) {
      terminal.options.theme = getActiveXtermTheme()
    }
  })
</script>

<div class="terminal-wrapper">
  <div class="terminal-container" bind:this={terminalEl}></div>

  {#if showSearch}
    <div class="terminal-search-bar" role="search">
      <input
        type="text"
        class="search-input"
        placeholder="Find in terminal..."
        bind:value={searchQuery}
        oninput={handleSearchNext}
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            if (e.shiftKey) handleSearchPrev()
            else handleSearchNext()
          } else if (e.key === 'Escape') {
            handleSearchClose()
          }
        }}
      />
      <button class="search-btn" onclick={handleSearchPrev} title="Previous (Shift+Enter)">&#x25B2;</button>
      <button class="search-btn" onclick={handleSearchNext} title="Next (Enter)">&#x25BC;</button>
      <button
        class="search-btn toggle-btn"
        class:active={searchMatchCase}
        onclick={() => { searchMatchCase = !searchMatchCase; handleSearchNext(); }}
        title="Match Case"
      >Aa</button>
      <button
        class="search-btn toggle-btn"
        class:active={searchRegex}
        onclick={() => { searchRegex = !searchRegex; handleSearchNext(); }}
        title="Regular Expression"
      >.*</button>
      <button class="search-btn close-search" onclick={handleSearchClose} title="Close (Escape)">&times;</button>
    </div>
  {/if}

  <div class="terminal-quick-actions">
    <button class="quick-btn" onclick={() => (showSearch = !showSearch)} title="Search (Ctrl+F)">&#x1F50D;</button>
    <button class="quick-btn" onclick={zoomIn} title="Zoom In">+</button>
    <button class="quick-btn" onclick={zoomOut} title="Zoom Out">-</button>
    <button class="quick-btn" onclick={clearTerminal} title="Clear Terminal Buffer">&#x232B;</button>
  </div>

  {#if disconnected}
    <div class="disconnect-overlay">
      <div class="disconnect-content">
        <p class="disconnect-msg">{t('terminal.connectionLost')}</p>
        {#if error}
          <p class="disconnect-error">{error}</p>
        {/if}
        <button class="reconnect-btn" onclick={reconnect}>{t('terminal.reconnect')}</button>
      </div>
    </div>
  {/if}
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

  /* Search bar */
  .terminal-search-bar {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--bg-secondary, #1e222b);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.15));
    border-radius: 6px;
    padding: 4px 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }

  .search-input {
    background: var(--bg-primary, #14171d);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #ffffff);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    outline: none;
    width: 160px;
  }

  .search-input:focus {
    border-color: var(--accent-primary, #6366f1);
  }

  .search-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    border-radius: 4px;
    width: 24px;
    height: 24px;
    font-size: 11px;
    padding: 0;
    transition: all 0.12s ease;
  }

  .search-btn:hover {
    background: var(--bg-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-primary, #ffffff);
  }

  .search-btn.toggle-btn.active {
    background: var(--accent-primary, #6366f1);
    color: #ffffff;
  }

  .search-btn.close-search {
    font-size: 16px;
  }

  /* Quick Actions floating panel */
  .terminal-quick-actions {
    position: absolute;
    bottom: 12px;
    right: 12px;
    z-index: 5;
    display: flex;
    align-items: center;
    gap: 3px;
    background: var(--bg-secondary, rgba(20, 24, 33, 0.85));
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    padding: 2px 4px;
    backdrop-filter: blur(4px);
    opacity: 0.25;
    transition: opacity 0.2s ease;
  }

  .terminal-quick-actions:hover {
    opacity: 1;
  }

  .quick-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    border-radius: 4px;
    width: 22px;
    height: 22px;
    font-size: 12px;
    transition: all 0.12s ease;
  }

  .quick-btn:hover {
    background: var(--bg-primary, rgba(255, 255, 255, 0.12));
    color: var(--text-primary, #ffffff);
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
