<script lang="ts">
  import FileList from './FileList.svelte'
  import * as api from '$lib/api/invoke'

  let { sessionId }: { sessionId: string } = $props()

  let currentPath = $state('')
  let entries = $state<any[]>([])
  let loading = $state(false)
  let error = $state('')
  let pathInput = $state('')
  let editingPath = $state(false)

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
</script>

<div class="file-browser">
  <div class="address-bar">
    {#if editingPath}
      <form onsubmit={(e: Event) => { e.preventDefault(); submitPath() }} class="path-form">
        <input type="text" bind:value={pathInput} autofocus />
      </form>
    {:else}
      <div class="breadcrumbs" onclick={() => (editingPath = true)}>
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
      <button class="icon-btn" onclick={loadDir.bind(null, currentPath)} title="Refresh">R</button>
      <button class="icon-btn" onclick={createDirectory} title="New folder">+</button>
    </div>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="file-content">
    {#if loading}
      <div class="loading">Loading...</div>
    {:else if entries.length === 0}
      <div class="empty">Empty directory</div>
    {:else}
      <FileList {entries} onNavigate={navigate} onDelete={deleteEntry} onRename={renameEntry} />
    {/if}
  </div>
</div>

<style>
  .file-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1f2b;
  }

  .address-bar {
    display: flex;
    align-items: center;
    padding: 6px 8px;
    background: #16171d;
    border-bottom: 1px solid #2e303a;
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
    background: #2a2a3e;
  }

  .breadcrumb {
    background: none;
    border: none;
    color: #4a90d9;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    padding: 2px 4px;
    border-radius: 3px;
  }

  .breadcrumb:hover {
    background: #4a90d922;
  }

  .separator {
    color: #6b7280;
    font-size: 12px;
  }

  .path-form {
    flex: 1;
  }

  .path-form input {
    width: 100%;
    background: #1a1a2e;
    border: 1px solid #4a90d9;
    border-radius: 4px;
    padding: 4px 8px;
    color: #e0e0e0;
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
    border: 1px solid #2e303a;
    color: #9ca3af;
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
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .error {
    background: #e06c7522;
    color: #e06c75;
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
    color: #6b7280;
    font-size: 13px;
  }
</style>
