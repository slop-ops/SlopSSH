<script lang="ts">
  import FileList from './FileList.svelte'
  import ContextMenu from '../common/ContextMenu.svelte'
  import FileEditor from './FileEditor.svelte'
  import * as api from '$lib/api/invoke'

  let { sessionId }: { sessionId: string } = $props()

  let currentPath = $state('')
  let entries = $state<any[]>([])
  let loading = $state(false)
  let error = $state('')
  let pathInput = $state('')
  let editingPath = $state(false)
  let dragOverState = $state(false)
  let contextMenu = $state<{ x: number; y: number; entry: any } | null>(null)
  let editingFile = $state<string | null>(null)

  $effect(() => {
    if (sessionId) loadHome()
  })

  async function loadHome() {
    loading = true
    error = ''
    try {
      const home = await api.sftpHome(sessionId)
      currentPath = home
      pathInput = home
      await loadDir(home)
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function loadDir(path: string) {
    loading = true
    error = ''
    try {
      entries = await api.sftpListDir(sessionId, path)
      currentPath = path
      pathInput = path
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function navigate(entry: any) {
    if (entry.isDir) {
      loadDir(entry.path)
    }
  }

  function goUp() {
    if (!currentPath || currentPath === '/') return
    const parts = currentPath.split('/').filter(Boolean)
    parts.pop()
    const parent = parts.length === 0 ? '/' : '/' + parts.join('/')
    loadDir(parent)
  }

  function submitPath() {
    editingPath = false
    if (pathInput && pathInput !== currentPath) {
      loadDir(pathInput)
    }
  }

  async function createDirectory() {
    const name = prompt('Directory name:')
    if (!name) return
    const path = currentPath === '/' ? `/${name}` : `${currentPath}/${name}`
    try {
      await api.sftpMkdir(sessionId, path)
      await loadDir(currentPath)
    } catch (e) {
      error = String(e)
    }
  }

  async function deleteEntry(entry: any) {
    if (!confirm(`Delete ${entry.name}?`)) return
    try {
      await api.sftpRemove(sessionId, entry.path)
      await loadDir(currentPath)
    } catch (e) {
      error = String(e)
    }
  }

  async function renameEntry(entry: any) {
    const newName = prompt('New name:', entry.name)
    if (!newName || newName === entry.name) return
    const newPath = currentPath === '/' ? `/${newName}` : `${currentPath}/${newName}`
    try {
      await api.sftpRename(sessionId, entry.path, newPath)
      await loadDir(currentPath)
    } catch (e) {
      error = String(e)
    }
  }

  function getBreadcrumbs(): string[] {
    if (currentPath === '/') return ['/']
    return currentPath.split('/').filter(Boolean)
  }

  function navigateToBreadcrumb(index: number) {
    const parts = getBreadcrumbs()
    if (index === 0 && parts[0] === '/') {
      loadDir('/')
      return
    }
    const path = '/' + parts.slice(0, index + 1).join('/')
    loadDir(path)
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault()
    e.stopPropagation()
    dragOverState = true
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault()
    e.stopPropagation()
    dragOverState = false
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault()
    e.stopPropagation()
    dragOverState = false

    if (!e.dataTransfer) return

    const files = e.dataTransfer.files
    if (!files || files.length === 0) return

    for (let i = 0; i < files.length; i++) {
      const file = files[i]
      if (!file) continue

      try {
        const arrayBuffer = await file.arrayBuffer()
        const bytes = new Uint8Array(arrayBuffer)
        let binary = ''
        for (let j = 0; j < bytes.length; j++) {
          binary += String.fromCharCode(bytes[j])
        }
        const base64 = btoa(binary)

        const remotePath =
          currentPath === '/' ? `/${file.name}` : `${currentPath}/${file.name}`

        await api.transferUpload(
          crypto.randomUUID(),
          sessionId,
          `__drag__:${base64}`,
          remotePath,
          file.size,
        )
      } catch (e) {
        error = String(e)
      }
    }

    await loadDir(currentPath)
  }

  function handleFileDragStart(entry: any, e: DragEvent) {
    if (!e.dataTransfer) return
    e.dataTransfer.setData('text/plain', JSON.stringify({
      type: 'remote-file',
      sessionId,
      path: entry.path,
      name: entry.name,
      isDir: entry.isDir,
      size: entry.size,
    }))
    e.dataTransfer.effectAllowed = 'copy'
  }

  function handleContextMenu(entry: any, e: MouseEvent) {
    e.preventDefault()
    contextMenu = { x: e.clientX, y: e.clientY, entry }
  }

  function handleContextMenuAction(action: string) {
    if (!contextMenu) return
    const entry = contextMenu.entry
    contextMenu = null

    switch (action) {
      case 'open':
        navigate(entry)
        break
      case 'rename':
        renameEntry(entry)
        break
      case 'delete':
        deleteEntry(entry)
        break
      case 'mkdir':
        createDirectory()
        break
      case 'refresh':
        loadDir(currentPath)
        break
      case 'extract':
        extractArchive(entry)
        break
      case 'archive':
        archiveEntry(entry)
        break
      case 'edit':
        editingFile = entry.path
        break
    }
  }

  async function extractArchive(entry: any) {
    const dirName = entry.name.replace(/\.(tar\.gz|tar\.bz2|tgz|tar|zip)$/, '')
    const targetPath = currentPath === '/' ? `/${dirName}` : `${currentPath}/${dirName}`
    try {
      await api.archiveExtract(sessionId, entry.path, targetPath)
      await loadDir(currentPath)
    } catch (e) {
      error = String(e)
    }
  }

  async function archiveEntry(entry: any) {
    const format = entry.isDir ? 'tar.gz' : 'tar.gz'
    const archiveName = `${entry.name}.tar.gz`
    const archivePath = currentPath === '/' ? `/${archiveName}` : `${currentPath}/${archiveName}`
    try {
      await api.archiveCreate(sessionId, archivePath, [entry.path], format)
      await loadDir(currentPath)
    } catch (e) {
      error = String(e)
    }
  }

  function isArchive(name: string): boolean {
    return /\.(tar\.gz|tar\.bz2|tgz|tar|zip)$/i.test(name)
  }

  function getContextMenuItems() {
    if (!contextMenu?.entry) {
      return [
        { label: 'New Folder', action: 'mkdir' },
        { label: '', separator: true },
        { label: 'Refresh', action: 'refresh' },
      ]
    }
    const entry = contextMenu.entry
    return [
      ...(entry.isDir ? [{ label: 'Open', action: 'open' }] : [{ label: 'Edit', action: 'edit' }]),
      { label: 'Rename', action: 'rename' },
      { label: 'Delete', action: 'delete' },
      { label: '', separator: true },
      ...(isArchive(entry.name) ? [{ label: 'Extract Here', action: 'extract' }] : []),
      ...(entry.isDir ? [{ label: 'Archive (tar.gz)', action: 'archive' }] : []),
      { label: '', separator: true },
      { label: 'New Folder', action: 'mkdir' },
    ]
  }
</script>

  <div
    class="file-browser"
    class:drag-over={dragOverState}
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
    oncontextmenu={(e) => {
      e.preventDefault()
      contextMenu = { x: e.clientX, y: e.clientY, entry: null }
    }}
  >
  <div class="address-bar">
    {#if editingPath}
      <form onsubmit={(e: Event) => { e.preventDefault(); submitPath() }} class="path-form">
        <input type="text" bind:value={pathInput} />
      </form>
    {:else}
      <div class="breadcrumbs" onclick={() => (editingPath = true)} role="navigation">
        {#each getBreadcrumbs() as part, i}
          {#if i > 0}
            <span class="separator">/</span>
          {/if}
          <button class="breadcrumb" onclick={() => navigateToBreadcrumb(i)}>{part}</button>
        {/each}
      </div>
    {/if}
    <div class="address-actions">
      <button class="icon-btn" onclick={goUp} title="Go up">..</button>
      <button class="icon-btn" onclick={() => loadDir(currentPath)} title="Refresh">R</button>
      <button class="icon-btn" onclick={createDirectory} title="New folder">+</button>
    </div>
  </div>

  {#if dragOverState}
    <div class="drop-overlay">
      <div class="drop-message">Drop files here to upload</div>
    </div>
  {/if}

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="file-content">
    {#if loading}
      <div class="loading">Loading...</div>
    {:else if entries.length === 0}
      <div class="empty">Empty directory</div>
    {:else}
      <FileList
        {entries}
        onNavigate={navigate}
        onDelete={deleteEntry}
        onRename={renameEntry}
        onDragStart={handleFileDragStart}
        onContextMenu={handleContextMenu}
      />
    {/if}
  </div>
</div>

{#if contextMenu}
  <ContextMenu
    items={getContextMenuItems()}
    x={contextMenu.x}
    y={contextMenu.y}
    onaction={handleContextMenuAction}
    onclose={() => (contextMenu = null)}
  />
{/if}

{#if editingFile}
  <FileEditor
    sessionId={sessionId}
    filePath={editingFile}
    onclose={() => {
      editingFile = null
      loadDir(currentPath)
    }}
  />
{/if}

<style>
  .file-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary);
    position: relative;
  }

  .file-browser.drag-over {
    outline: 2px dashed var(--accent);
    outline-offset: -2px;
  }

  .address-bar {
    display: flex;
    align-items: center;
    padding: 6px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-primary);
    gap: 8px;
  }

  .breadcrumbs {
    display: flex;
    align-items: center;
    flex: 1;
    gap: 2px;
    cursor: text;
    padding: 4px 8px;
    border-radius: 4px;
    min-height: 28px;
    overflow-x: auto;
  }

  .breadcrumbs:hover {
    background: var(--bg-hover);
  }

  .breadcrumb {
    background: none;
    border: none;
    color: var(--accent-text);
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    padding: 2px 4px;
    border-radius: 3px;
  }

  .breadcrumb:hover {
    background: var(--accent-bg);
  }

  .separator {
    color: var(--text-tertiary);
    font-size: 12px;
  }

  .path-form {
    flex: 1;
  }

  .path-form input {
    width: 100%;
    background: var(--bg-input);
    border: 1px solid var(--border-active);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: monospace;
    outline: none;
  }

  .address-actions {
    display: flex;
    gap: 4px;
  }

  .icon-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    width: 26px;
    height: 26px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .drop-overlay {
    position: absolute;
    inset: 40px 0 0 0;
    background: var(--accent-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    pointer-events: none;
  }

  .drop-message {
    color: var(--accent-text);
    font-size: 16px;
    font-weight: 500;
    padding: 16px 32px;
    border: 2px dashed var(--accent);
    border-radius: 8px;
  }

  .error {
    background: var(--error-bg);
    color: var(--error);
    padding: 6px 12px;
    font-size: 12px;
  }

  .file-content {
    flex: 1;
    overflow-y: auto;
  }

  .loading,
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    font-size: 13px;
  }
</style>
