<script lang="ts">
  interface FileEntry {
    name: string
    path: string
    isDir: boolean
    isFile: boolean
    isSymlink: boolean
    size: number
    modified: number | null
  }

  let {
    entries,
    onNavigate,
    onDelete,
    onRename,
    onDragStart,
  }: {
    entries: FileEntry[]
    onNavigate: (entry: FileEntry) => void
    onDelete: (entry: FileEntry) => void
    onRename: (entry: FileEntry) => void
    onDragStart?: (entry: FileEntry, e: DragEvent) => void
  } = $props()

  let selectedPath = $state<string | null>(null)

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

  function getIcon(entry: FileEntry): string {
    if (entry.isDir) return 'D'
    if (entry.isSymlink) return 'L'
    return 'F'
  }

  function handleDoubleClick(entry: FileEntry) {
    if (entry.isDir) {
      onNavigate(entry)
    }
  }

  function handleDragStart(entry: FileEntry, e: DragEvent) {
    onDragStart?.(entry, e)
  }
</script>

<table class="file-list">
  <thead>
    <tr>
      <th class="col-icon"></th>
      <th class="col-name">Name</th>
      <th class="col-size">Size</th>
      <th class="col-modified">Modified</th>
      <th class="col-actions"></th>
    </tr>
  </thead>
  <tbody>
    {#each entries as entry (entry.path)}
      <tr
        class="file-row"
        class:selected={selectedPath === entry.path}
        draggable="true"
        onclick={() => (selectedPath = entry.path)}
        ondblclick={() => handleDoubleClick(entry)}
        ondragstart={(e) => handleDragStart(entry, e)}
      >
        <td class="col-icon">
          <span class="icon" class:dir={entry.isDir} class:file={entry.isFile}>
            {getIcon(entry)}
          </span>
        </td>
        <td class="col-name">
          <span class="name" class:dir-name={entry.isDir}>{entry.name}</span>
        </td>
        <td class="col-size">{entry.isDir ? '-' : formatSize(entry.size)}</td>
        <td class="col-modified">{formatDate(entry.modified)}</td>
        <td class="col-actions">
          <button class="action-btn" onclick={(e: MouseEvent) => { e.preventDefault(); onRename(entry) }} title="Rename">
            R
          </button>
          <button class="action-btn delete" onclick={(e: MouseEvent) => { e.preventDefault(); onDelete(entry) }} title="Delete">
            x
          </button>
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .file-list {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  th {
    text-align: left;
    padding: 6px 8px;
    color: var(--text-tertiary);
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border-primary);
    background: var(--bg-secondary);
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .col-icon {
    width: 28px;
    text-align: center;
  }

  .col-name {
    min-width: 200px;
  }

  .col-size {
    width: 80px;
    text-align: right;
  }

  .col-modified {
    width: 160px;
  }

  .col-actions {
    width: 60px;
    text-align: right;
  }

  .file-row {
    cursor: pointer;
    transition: background 0.1s;
  }

  .file-row:hover {
    background: var(--bg-hover);
  }

  .file-row.selected {
    background: var(--accent-bg);
  }

  td {
    padding: 4px 8px;
    border-bottom: 1px solid var(--border-subtle);
    white-space: nowrap;
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
  }

  .icon.dir {
    background: var(--accent-bg);
    color: var(--accent-text);
  }

  .icon.file {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .name {
    color: var(--text-primary);
  }

  .dir-name {
    color: var(--accent-text);
    font-weight: 500;
  }

  .col-size {
    color: var(--text-secondary);
    text-align: right;
  }

  .col-modified {
    color: var(--text-tertiary);
  }

  .action-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
  }

  .file-row:hover .action-btn {
    opacity: 1;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .action-btn.delete:hover {
    color: var(--error);
    background: var(--error-bg);
  }
</style>
