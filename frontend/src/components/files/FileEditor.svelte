<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let {
    sessionId,
    filePath,
    onclose,
  }: {
    sessionId: string
    filePath: string
    onclose: () => void
  } = $props()

  let content = $state('')
  let originalContent = $state('')
  let loading = $state(true)
  let saving = $state(false)
  let error = $state('')
  let modified = $state(false)
  let fileName = $derived(filePath.split('/').pop() || 'file')

  const MAX_FILE_SIZE = 10 * 1024 * 1024

  $effect(() => {
    loadFile()
  })

  async function loadFile() {
    loading = true
    error = ''
    try {
      const stat = await api.sftpStat(sessionId, filePath) as unknown as { attributes: { size: number } } | null
      const fileSize = stat?.attributes?.size ?? 0
      if (fileSize > MAX_FILE_SIZE) {
        const sizeMB = (fileSize / (1024 * 1024)).toFixed(1)
        error = `${t('files.fileTooLarge') || 'File too large to edit'} (${sizeMB} MB)`
        loading = false
        return
      }
      const base64 = await api.sftpReadFile(sessionId, filePath)
      const decoded = atob(base64)
      content = decoded
      originalContent = decoded
      modified = false
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function saveFile() {
    saving = true
    error = ''
    try {
      let binary = ''
      for (let i = 0; i < content.length; i++) {
        binary += String.fromCharCode(content.charCodeAt(i))
      }
      const base64 = btoa(binary)
      await api.sftpWriteFile(sessionId, filePath, base64)
      originalContent = content
      modified = false
    } catch (e) {
      error = String(e)
    } finally {
      saving = false
    }
  }

  function handleInput() {
    modified = content !== originalContent
  }

  async function handleClose() {
    if (modified) {
      const discard = confirm(t('files.unsaved'))
      if (!discard) return
    }
    onclose()
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault()
      if (modified) saveFile()
    }
    if (e.key === 'Escape') {
      handleClose()
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="backdrop" onclick={(e) => { if (e.target === e.currentTarget) handleClose() }}>
  <div class="dialog" role="dialog" aria-modal="true" aria-label="File editor">
    <div class="dialog-header">
      <h3>Edit: {fileName}</h3>
      <button class="close-btn" onclick={handleClose}>x</button>
    </div>

    <div class="editor-container">
      {#if loading}
        <div class="loading">{t('files.loadFile')}</div>
      {:else}
        {#if error}
          <div class="error">{error}</div>
        {/if}
        <div class="toolbar">
          <span class="filename">{filePath}</span>
          <div class="toolbar-actions">
            {#if modified}
              <span class="modified">{t('files.modified')}</span>
            {/if}
            <button class="save-btn" onclick={saveFile} disabled={!modified || saving}>
              {saving ? t('files.saving') : t('files.saveCtrl')}
            </button>
          </div>
        </div>
        <textarea
          class="editor"
          bind:value={content}
          oninput={handleInput}
          spellcheck="false"
        ></textarea>
      {/if}
    </div>
  </div>
</div>

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
    max-width: 800px;
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

  .editor-container {
    display: flex;
    flex-direction: column;
    height: 60vh;
    min-height: 400px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-primary);
  }

  .filename {
    font-family: monospace;
    font-size: 11px;
    color: var(--text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .modified {
    font-size: 11px;
    color: var(--warning);
  }

  .save-btn {
    background: var(--accent);
    color: var(--text-inverse);
    border: none;
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .save-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .editor {
    flex: 1;
    width: 100%;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: none;
    outline: none;
    resize: none;
    padding: 12px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    line-height: 1.5;
    tab-size: 4;
  }

  .loading,
  .error {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    font-size: 13px;
  }

  .error {
    color: var(--error);
    background: var(--error-bg);
    padding: 6px 12px;
    font-size: 12px;
  }
</style>
