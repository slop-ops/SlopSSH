<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import { showToast } from '$lib/stores/toast.svelte'
  import PromptDialog from '../common/PromptDialog.svelte'
  import ConfirmDialog from '../common/ConfirmDialog.svelte'
  import type { FileItem } from '$lib/types'

  let {
    mode = 'remote',
    sessionId = '',
    title = '',
    onFileOpen,
    onTransferRequest,
    onDragStartTransfer,
  }: {
    mode: 'remote' | 'local'
    sessionId?: string
    title?: string
    onFileOpen?: (file: FileItem) => void
    onTransferRequest?: (item: FileItem, targetDir: string) => void
    onDragStartTransfer?: (item: FileItem, e: DragEvent) => void
  } = $props()

  let currentPath = $state('')
  let pathInput = $state('')
  let editingPath = $state(false)
  let entries = $state<FileItem[]>([])
  let loading = $state(false)
  let error = $state('')
  let searchQuery = $state('')
  let showHidden = $state(false)
  let selectedPath = $state<string | null>(null)
  let dragOverSelf = $state(false)
  let contextMenu = $state<{ x: number; y: number; entry: FileItem | null } | null>(null)
  let drives = $state<string[]>([])
  let showDrivesDropdown = $state(false)

  let filteredEntries = $derived(
    entries
      .filter((e) => {
        if (!showHidden && e.name.startsWith('.')) return false
        if (!searchQuery) return true
        return e.name.toLowerCase().includes(searchQuery.toLowerCase())
      })
      .sort((a, b) => {
        if (a.isDir && !b.isDir) return -1
        if (!a.isDir && b.isDir) return 1
        return a.name.localeCompare(b.name)
      })
  )

  $effect(() => {
    if (mode === 'remote') {
      if (sessionId) loadRemoteHome()
    } else {
      loadLocalHome()
    }
  })

  async function loadRemoteHome() {
    loading = true
    error = ''
    try {
      try {
        await api.sftpConnect(sessionId)
      } catch {
        // already connected
      }
      const home = await api.sftpHome(sessionId)
      currentPath = home || '/'
      pathInput = currentPath
      await loadDir(currentPath)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function loadLocalHome() {
    loading = true
    error = ''
    try {
      const home = await api.localGetHome()
      currentPath = home || '/'
      pathInput = currentPath
      await loadDir(currentPath)
      try {
        drives = await api.localGetDrives()
      } catch {
        drives = []
      }
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  export async function loadDir(path: string) {
    loading = true
    error = ''
    try {
      if (mode === 'remote') {
        const raw = await api.sftpListDir(sessionId, path)
        entries = (raw as unknown as FileItem[]) || []
      } else {
        entries = (await api.localListDir(path)) || []
      }
      currentPath = path
      pathInput = path
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  export function getCurrentPath(): string {
    return currentPath
  }

  export function getSelectedFile(): FileItem | null {
    return entries.find(e => e.path === selectedPath) || null
  }

  function handleNavigate(entry: FileItem) {
    if (entry.isDir) {
      loadDir(entry.path)
    } else {
      onFileOpen?.(entry)
    }
  }

  function goUp() {
    if (!currentPath || currentPath === '/') return
    const parts = currentPath.split('/').filter(Boolean)
    parts.pop()
    const parent = parts.length === 0 ? '/' : '/' + parts.join('/')
    loadDir(parent)
  }

  function goRoot() {
    loadDir('/')
  }

  function goHome() {
    if (mode === 'remote') loadRemoteHome()
    else loadLocalHome()
  }

  function submitPath() {
    editingPath = false
    if (pathInput && pathInput !== currentPath) {
      loadDir(pathInput)
    }
  }

  let promptModal = $state<{
    open: boolean
    title: string
    message: string
    defaultValue: string
    placeholder: string
    onconfirm: (val: string) => void
  }>({
    open: false,
    title: '',
    message: '',
    defaultValue: '',
    placeholder: '',
    onconfirm: () => {},
  })

  let confirmModal = $state<{
    open: boolean
    title: string
    message: string
    isDanger: boolean
    onconfirm: () => void
  }>({
    open: false,
    title: '',
    message: '',
    isDanger: false,
    onconfirm: () => {},
  })

  function createDirectory() {
    promptModal = {
      open: true,
      title: 'New Folder',
      message: `Create new folder in ${currentPath}:`,
      defaultValue: '',
      placeholder: 'folder-name',
      onconfirm: async (name: string) => {
        const sep = currentPath.endsWith('/') ? '' : '/'
        const newPath = `${currentPath}${sep}${name}`
        try {
          if (mode === 'remote') {
            await api.sftpMkdir(sessionId, newPath)
          } else {
            await api.localMkdir(newPath)
          }
          showToast(`Created folder "${name}"`, 'success')
          await loadDir(currentPath)
        } catch (e) {
          showToast(`Failed to create folder: ${e}`, 'error')
        }
      },
    }
  }

  function deleteEntry(entry: FileItem) {
    confirmModal = {
      open: true,
      title: 'Delete Item',
      message: `Permanently delete "${entry.name}"? This action cannot be undone.`,
      isDanger: true,
      onconfirm: async () => {
        try {
          if (mode === 'remote') {
            await api.sftpRemove(sessionId, entry.path)
          } else {
            await api.localRemove(entry.path)
          }
          showToast(`Deleted "${entry.name}"`, 'success')
          await loadDir(currentPath)
        } catch (e) {
          showToast(`Failed to delete: ${e}`, 'error')
        }
      },
    }
  }

  function renameEntry(entry: FileItem) {
    promptModal = {
      open: true,
      title: 'Rename Item',
      message: `Enter new name for "${entry.name}":`,
      defaultValue: entry.name,
      placeholder: 'new-name',
      onconfirm: async (newName: string) => {
        if (newName === entry.name) return
        const sep = currentPath.endsWith('/') ? '' : '/'
        const newPath = `${currentPath}${sep}${newName}`
        try {
          if (mode === 'remote') {
            await api.sftpRename(sessionId, entry.path, newPath)
          } else {
            await api.localRename(entry.path, newPath)
          }
          showToast(`Renamed to "${newName}"`, 'success')
          await loadDir(currentPath)
        } catch (e) {
          showToast(`Failed to rename: ${e}`, 'error')
        }
      },
    }
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return '-'
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let i = 0
    let size = bytes
    while (size >= 1024 && i < units.length - 1) {
      size /= 1024
      i++
    }
    return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`
  }

  function formatDate(ts: number | null): string {
    if (!ts) return '-'
    return new Date(ts).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  function getBreadcrumbs(): { name: string; path: string }[] {
    if (!currentPath) return []
    const parts = currentPath.split('/').filter(Boolean)
    const crumbs = [{ name: '/', path: '/' }]
    let acc = ''
    for (const p of parts) {
      acc += '/' + p
      crumbs.push({ name: p, path: acc })
    }
    return crumbs
  }

  function handleDragStart(entry: FileItem, e: DragEvent) {
    if (!e.dataTransfer) return
    const payload = {
      type: mode,
      sessionId: mode === 'remote' ? sessionId : '',
      path: entry.path,
      name: entry.name,
      isDir: entry.isDir,
      size: entry.size,
    }
    e.dataTransfer.setData('text/plain', JSON.stringify(payload))
    e.dataTransfer.effectAllowed = 'copyMove'
    onDragStartTransfer?.(entry, e)
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault()
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'copy'
    }
    dragOverSelf = true
  }

  function handleDragLeave() {
    dragOverSelf = false
  }

  async function handleDrop(e: DragEvent, targetFolder?: FileItem) {
    e.preventDefault()
    e.stopPropagation()
    dragOverSelf = false

    const destDir = targetFolder && targetFolder.isDir ? targetFolder.path : currentPath

    if (e.dataTransfer) {
      const jsonStr = e.dataTransfer.getData('text/plain')
      if (jsonStr) {
        try {
          const item = JSON.parse(jsonStr)
          if (item && item.type !== mode) {
            onTransferRequest?.(item, destDir)
            return
          }
        } catch {
          // not json payload
        }
      }

      // External files dropped from OS file manager
      const files = e.dataTransfer.files
      if (files && files.length > 0) {
        for (let i = 0; i < files.length; i++) {
          const f = files[i]
          const arrayBuffer = await f.arrayBuffer()
          const bytes = new Uint8Array(arrayBuffer)
          let binary = ''
          for (let j = 0; j < bytes.length; j++) {
            binary += String.fromCharCode(bytes[j])
          }
          const base64 = btoa(binary)
          const sep = destDir.endsWith('/') ? '' : '/'
          const destFile = `${destDir}${sep}${f.name}`

          if (mode === 'remote') {
            await api.transferUpload(
              crypto.randomUUID(),
              sessionId,
              `__drag__:${base64}`,
              destFile,
              f.size
            )
          }
        }
        await loadDir(currentPath)
      }
    }
  }

  function handleContextMenu(entry: FileItem, e: MouseEvent) {
    e.preventDefault()
    selectedPath = entry.path
    contextMenu = { x: e.clientX, y: e.clientY, entry }
  }

  function closeContextMenu() {
    contextMenu = null
  }
</script>

<div
  class="file-pane {mode}"
  class:drag-over={dragOverSelf}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={(e) => handleDrop(e)}
  role="region"
  aria-label={title || mode}
>
  <!-- Top Navigation & Path Bar -->
  <div class="pane-header">
    <div class="pane-title-badge">
      {#if mode === 'remote'}
        <span class="badge remote-badge">&#9729; Remote</span>
      {:else}
        <span class="badge local-badge">&#128187; Local</span>
      {/if}
      {#if title}
        <span class="pane-title-text">{title}</span>
      {/if}
    </div>

    <div class="pane-nav-buttons">
      <button type="button" class="nav-btn" onclick={goUp} title="Go to parent directory (..)">&#x2191;</button>
      <button type="button" class="nav-btn" onclick={goHome} title="Go to Home directory (~)"><span class="home-icon">&#x2302;</span></button>
      <button type="button" class="nav-btn" onclick={goRoot} title="Go to Root (/)">/</button>

      {#if mode === 'local' && drives.length > 0}
        <div class="drives-dropdown-container">
          <button type="button" class="nav-btn drives-btn" onclick={() => (showDrivesDropdown = !showDrivesDropdown)} title="Select Drive">&#128190;</button>
          {#if showDrivesDropdown}
            <div class="drives-menu">
              {#each drives as drive}
                <button type="button" class="drive-item" onclick={() => { loadDir(drive); showDrivesDropdown = false }}>{drive}</button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <button type="button" class="nav-btn" onclick={() => loadDir(currentPath)} title="Refresh">&#x21BB;</button>
      <button type="button" class="nav-btn" class:active={showHidden} onclick={() => (showHidden = !showHidden)} title="Toggle hidden files">.*</button>
      <button type="button" class="nav-btn" onclick={createDirectory} title="New Folder">&#128193;+</button>
    </div>
  </div>

  <!-- Breadcrumbs & Path Input -->
  <div class="path-bar">
    {#if editingPath}
      <form onsubmit={(e) => { e.preventDefault(); submitPath(); }} class="path-form">
        <input
          type="text"
          class="path-input"
          bind:value={pathInput}
          onblur={() => (editingPath = false)}
        />
      </form>
    {:else}
      <div
        class="breadcrumbs-container"
        role="button"
        tabindex={0}
        ondblclick={() => (editingPath = true)}
        onkeydown={(e) => { if (e.key === 'Enter') editingPath = true; }}
        title="Double-click to edit path directly"
      >
        {#each getBreadcrumbs() as crumb}
          <button type="button" class="crumb-btn" onclick={() => loadDir(crumb.path)}>
            {crumb.name}
          </button>
          {#if crumb.name !== '/'}
            <span class="crumb-sep">/</span>
          {/if}
        {/each}
      </div>
      <button type="button" class="edit-path-btn" onclick={() => (editingPath = true)} title="Edit path">&#x270E;</button>
    {/if}
  </div>

  <!-- Search Filter Bar -->
  <div class="search-filter-bar">
    <span class="search-icon">&#x1F50D;</span>
    <input
      type="text"
      class="filter-input"
      placeholder="Filter files..."
      bind:value={searchQuery}
    />
    {#if searchQuery}
      <button type="button" class="clear-search-btn" onclick={() => (searchQuery = '')}>&times;</button>
    {/if}
    <span class="item-count">{filteredEntries.length} items</span>
  </div>

  <!-- File Table -->
  <div class="file-table-container">
    {#if loading}
      <div class="pane-loading">
        <span class="loading-spinner"></span>
        <span>Loading directory...</span>
      </div>
    {:else if error}
      <div class="pane-error">
        <span class="error-icon">&#9888;</span>
        <span>{error}</span>
        <button type="button" class="retry-btn" onclick={() => loadDir(currentPath)}>Retry</button>
      </div>
    {:else if filteredEntries.length === 0}
      <div class="pane-empty">
        <p>This directory is empty</p>
      </div>
    {:else}
      <table class="file-table" role="grid">
        <thead>
          <tr>
            <th class="th-icon"></th>
            <th class="th-name">{t('files.name')}</th>
            <th class="th-size">{t('files.size')}</th>
            <th class="th-modified">{t('files.modified')}</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredEntries as entry (entry.path)}
            <tr
              class="file-row"
              class:selected={selectedPath === entry.path}
              class:is-dir={entry.isDir}
              draggable="true"
              onclick={() => (selectedPath = entry.path)}
              ondblclick={() => handleNavigate(entry)}
              ondragstart={(e) => handleDragStart(entry, e)}
              ondrop={(e) => handleDrop(e, entry)}
              oncontextmenu={(e) => handleContextMenu(entry, e)}
            >
              <td class="td-icon">
                {#if entry.isDir}
                  <svg class="file-svg-icon dir-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                  </svg>
                {:else if entry.isSymlink}
                  <svg class="file-svg-icon link-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
                    <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
                  </svg>
                {:else if entry.name.endsWith('.tar.gz') || entry.name.endsWith('.zip') || entry.name.endsWith('.tgz')}
                  <svg class="file-svg-icon archive-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="21 8 21 21 3 21 3 8"/>
                    <rect x="1" y="3" width="22" height="5"/>
                    <line x1="10" y1="12" x2="14" y2="12"/>
                  </svg>
                {:else}
                  <svg class="file-svg-icon file-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                    <polyline points="14 2 14 8 20 8"/>
                  </svg>
                {/if}
              </td>
              <td class="td-name">
                <span class="file-name" title={entry.name}>{entry.name}</span>
              </td>
              <td class="td-size">{entry.isDir ? '-' : formatSize(entry.size)}</td>
              <td class="td-modified">{formatDate(entry.modified)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

{#if contextMenu && contextMenu.entry}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="context-menu-backdrop" onclick={closeContextMenu}>
    <div
      class="context-menu"
      style:top="{contextMenu.y}px"
      style:left="{contextMenu.x}px"
      onclick={(e) => e.stopPropagation()}
    >
      <button type="button" class="menu-item" onclick={() => { handleNavigate(contextMenu!.entry!); closeContextMenu(); }}>
        {contextMenu.entry.isDir ? 'Open Folder' : 'Open / Edit'}
      </button>

      <button type="button" class="menu-item" onclick={() => { onTransferRequest?.(contextMenu!.entry!, currentPath); closeContextMenu(); }}>
        {mode === 'remote' ? 'Download to Local' : 'Upload to Remote'}
      </button>

      <div class="menu-sep"></div>

      <button type="button" class="menu-item" onclick={() => { renameEntry(contextMenu!.entry!); closeContextMenu(); }}>
        Rename
      </button>

      <button type="button" class="menu-item" onclick={() => { navigator.clipboard.writeText(contextMenu!.entry!.path); closeContextMenu(); }}>
        Copy Path
      </button>

      <div class="menu-sep"></div>

      <button type="button" class="menu-item danger" onclick={() => { deleteEntry(contextMenu!.entry!); closeContextMenu(); }}>
        Delete
      </button>
    </div>
  </div>
{/if}

<PromptDialog
  bind:open={promptModal.open}
  title={promptModal.title}
  message={promptModal.message}
  defaultValue={promptModal.defaultValue}
  placeholder={promptModal.placeholder}
  onconfirm={promptModal.onconfirm}
/>

<ConfirmDialog
  bind:open={confirmModal.open}
  title={confirmModal.title}
  message={confirmModal.message}
  isDanger={confirmModal.isDanger}
  onconfirm={confirmModal.onconfirm}
/>

<style>
  .file-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    background: var(--bg-primary, #15181e);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    border-radius: 6px;
    overflow: hidden;
    position: relative;
  }

  .file-pane.drag-over {
    border-color: var(--accent-primary, #6366f1);
    box-shadow: inset 0 0 12px rgba(99, 102, 241, 0.25);
  }

  .pane-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    background: var(--bg-secondary, #1a1e27);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    gap: 8px;
    min-height: 36px;
    flex-shrink: 0;
  }

  .pane-title-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
  }

  .badge {
    padding: 2px 8px;
    border-radius: 4px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-size: 10px;
  }

  .remote-badge {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.3);
  }

  .local-badge {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    border: 1px solid rgba(59, 130, 246, 0.3);
  }

  .pane-title-text {
    color: var(--text-secondary, #94a3b8);
  }

  .pane-nav-buttons {
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .nav-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    font-size: 12px;
    transition: all 0.12s ease;
    padding: 0;
  }

  .nav-btn:hover {
    background: var(--bg-hover, rgba(255, 255, 255, 0.08));
    color: var(--text-primary, #ffffff);
    border-color: var(--border-primary, rgba(255, 255, 255, 0.1));
  }

  .nav-btn.active {
    background: var(--accent-primary, #6366f1);
    color: #ffffff;
  }

  .home-icon {
    font-size: 14px;
    line-height: 1;
  }

  .drives-dropdown-container {
    position: relative;
  }

  .drives-menu {
    position: absolute;
    top: 28px;
    right: 0;
    z-index: 50;
    background: var(--bg-secondary, #1e222b);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.15));
    border-radius: 6px;
    padding: 4px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    min-width: 80px;
  }

  .drive-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: var(--text-primary, #ffffff);
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px;
  }

  .drive-item:hover {
    background: var(--accent-primary, #6366f1);
  }

  .path-bar {
    display: flex;
    align-items: center;
    padding: 3px 8px;
    background: var(--bg-tertiary, #12141a);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.06));
    min-height: 28px;
    flex-shrink: 0;
  }

  .path-form {
    flex: 1;
    display: flex;
  }

  .path-input {
    flex: 1;
    background: var(--bg-primary, #15181e);
    border: 1px solid var(--accent-primary, #6366f1);
    color: var(--text-primary, #ffffff);
    border-radius: 3px;
    padding: 2px 6px;
    font-size: 11px;
    font-family: monospace;
    outline: none;
  }

  .breadcrumbs-container {
    flex: 1;
    display: flex;
    align-items: center;
    overflow-x: auto;
    scrollbar-width: none;
    font-size: 11px;
  }

  .breadcrumbs-container::-webkit-scrollbar {
    display: none;
  }

  .crumb-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    padding: 1px 3px;
    border-radius: 3px;
    font-size: 11px;
    white-space: nowrap;
  }

  .crumb-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary, #ffffff);
  }

  .crumb-sep {
    color: var(--text-muted, #475569);
    margin: 0 1px;
    font-size: 10px;
  }

  .edit-path-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, #64748b);
    cursor: pointer;
    font-size: 11px;
    padding: 2px 4px;
    border-radius: 3px;
    opacity: 0.6;
  }

  .edit-path-btn:hover {
    opacity: 1;
    color: var(--text-primary, #ffffff);
  }

  .search-filter-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--bg-primary, #15181e);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.04));
    font-size: 11px;
    flex-shrink: 0;
  }

  .search-icon {
    color: var(--text-muted, #64748b);
    font-size: 10px;
  }

  .filter-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary, #ffffff);
    font-size: 11px;
    outline: none;
  }

  .clear-search-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, #64748b);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
  }

  .item-count {
    font-size: 10px;
    color: var(--text-muted, #64748b);
  }

  .file-table-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
  }

  .file-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    table-layout: fixed;
  }

  .file-table th {
    text-align: left;
    padding: 6px 8px;
    font-weight: 500;
    color: var(--text-muted, #64748b);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    font-size: 11px;
    user-select: none;
  }

  .th-icon { width: 28px; }
  .th-name { width: auto; }
  .th-size { width: 80px; text-align: right; }
  .th-modified { width: 120px; text-align: right; }

  .file-row {
    cursor: pointer;
    user-select: none;
    transition: background 0.1s ease;
  }

  .file-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .file-row.selected {
    background: rgba(99, 102, 241, 0.18);
  }

  .file-row td {
    padding: 4px 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .td-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    padding-top: 6px;
  }

  .file-svg-icon {
    flex-shrink: 0;
  }

  .dir-icon {
    color: #f59e0b;
  }

  .link-icon {
    color: #a855f7;
  }

  .archive-icon {
    color: #ec4899;
  }

  .file-icon {
    color: #94a3b8;
  }

  .td-name {
    color: var(--text-primary, #f1f5f9);
  }

  .file-row.is-dir .td-name {
    font-weight: 500;
  }

  .td-size {
    text-align: right;
    color: var(--text-secondary, #94a3b8);
    font-family: monospace;
    font-size: 11px;
  }

  .td-modified {
    text-align: right;
    color: var(--text-muted, #64748b);
    font-size: 11px;
  }

  .pane-loading,
  .pane-empty,
  .pane-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: var(--text-muted, #64748b);
    font-size: 12px;
    padding: 24px;
    text-align: center;
  }

  .pane-error {
    color: #ef4444;
  }

  .retry-btn {
    margin-top: 6px;
    padding: 4px 12px;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
  }

  .context-menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
  }

  .context-menu {
    position: fixed;
    z-index: 1001;
    background: var(--bg-secondary, #1e222b);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.15));
    border-radius: 6px;
    padding: 4px;
    min-width: 150px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 10px;
    background: transparent;
    border: none;
    color: var(--text-primary, #ffffff);
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.1s;
  }

  .menu-item:hover {
    background: var(--accent-primary, #6366f1);
  }

  .menu-item.danger:hover {
    background: #ef4444;
  }

  .menu-sep {
    height: 1px;
    background: var(--border-primary, rgba(255, 255, 255, 0.08));
    margin: 4px 0;
  }
</style>
