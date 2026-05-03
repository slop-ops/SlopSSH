<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import type { Snippet } from '$lib/types'

  let { onSend }: { onSend: (command: string) => void } = $props()

  let snippets = $state<Snippet[]>([])
  let search = $state('')
  let showForm = $state(false)
  let editingId = $state<string | null>(null)
  let formName = $state('')
  let formCommand = $state('')
  let formDescription = $state('')

  $effect(() => {
    loadSnippets()
  })

  async function loadSnippets() {
    try {
      const result = await api.listSnippets()
      snippets = Array.isArray(result) ? result : []
    } catch {
      snippets = []
    }
  }

  let filtered = $derived(
    snippets.filter(
      (s) =>
        !search ||
        s.name.toLowerCase().includes(search.toLowerCase()) ||
        s.command.toLowerCase().includes(search.toLowerCase()),
    ),
  )

  async function saveSnippet() {
    if (!formName.trim() || !formCommand.trim()) return
    try {
      if (editingId) {
        await api.updateSnippet({
          id: editingId,
          name: formName.trim(),
          command: formCommand.trim(),
          description: formDescription.trim() || null,
        })
      } else {
        await api.createSnippet({
          id: '',
          name: formName.trim(),
          command: formCommand.trim(),
          description: formDescription.trim() || null,
        })
      }
      resetForm()
      await loadSnippets()
    } catch (e) {
      console.error('Failed to save snippet:', e)
    }
  }

  async function deleteSnippet(id: string) {
    try {
      await api.deleteSnippet(id)
      await loadSnippets()
    } catch (e) {
      console.error('Failed to delete snippet:', e)
    }
  }

  function editSnippet(snippet: Snippet) {
    editingId = snippet.id
    formName = snippet.name
    formCommand = snippet.command
    formDescription = snippet.description || ''
    showForm = true
  }

  function resetForm() {
    editingId = null
    formName = ''
    formCommand = ''
    formDescription = ''
    showForm = false
  }
</script>

<div class="snippet-panel">
  <div class="panel-header">
    <h3>{t('terminal.snippets')}</h3>
    <button class="add-btn" onclick={() => { resetForm(); showForm = true }} aria-label="Add snippet">+</button>
  </div>

  <div class="search-bar">
    <input
      type="text"
      bind:value={search}
      placeholder={t('terminal.search')}
    />
  </div>

  {#if showForm}
    <div class="form">
      <input type="text" bind:value={formName} placeholder={t('terminal.snippetName')} />
      <textarea bind:value={formCommand} placeholder={t('terminal.snippetCommand')} rows="3"></textarea>
      <input type="text" bind:value={formDescription} placeholder={t('terminal.snippetDescription')} />
      <div class="form-actions">
        <button class="btn-save" onclick={saveSnippet}>
          {editingId ? t('terminal.update') : t('terminal.save')}
        </button>
        <button class="btn-cancel" onclick={resetForm}>{t('session.cancel')}</button>
      </div>
    </div>
  {/if}

  <div class="snippet-list">
    {#each filtered as snippet (snippet.id)}
      <div class="snippet-item">
        <div class="snippet-info" role="button" tabindex={0} onclick={() => onSend(snippet.command)} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSend(snippet.command) }}>
          <span class="snippet-name">{snippet.name}</span>
          <span class="snippet-cmd">{snippet.command}</span>
        </div>
        <div class="snippet-actions">
          <button class="act-btn" onclick={() => editSnippet(snippet)} aria-label="Edit snippet">E</button>
          <button class="act-btn del" onclick={() => deleteSnippet(snippet.id)} aria-label="Delete snippet">x</button>
        </div>
      </div>
    {/each}

    {#if filtered.length === 0}
      <div class="empty">
        {search ? t('terminal.noMatches') : t('terminal.noSnippets')}
      </div>
    {/if}
  </div>
</div>

<style>
  .snippet-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary);
    border-left: 1px solid var(--border-primary);
    width: 240px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-primary);
  }

  .panel-header h3 {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .add-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    width: 22px;
    height: 22px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .add-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .search-bar {
    padding: 8px 12px;
  }

  .search-bar input {
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 5px 8px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: inherit;
    outline: none;
    box-sizing: border-box;
  }

  .search-bar input:focus {
    border-color: var(--border-active);
  }

  .form {
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    border-bottom: 1px solid var(--border-primary);
  }

  .form input,
  .form textarea {
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 5px 8px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    outline: none;
    resize: vertical;
  }

  .form input:focus,
  .form textarea:focus {
    border-color: var(--border-active);
  }

  .form-actions {
    display: flex;
    gap: 6px;
  }

  .btn-save {
    flex: 1;
    background: var(--accent);
    border: none;
    color: #fff;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-family: inherit;
  }

  .btn-cancel {
    flex: 1;
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-family: inherit;
  }

  .snippet-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .snippet-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    cursor: pointer;
    transition: background 0.1s;
  }

  .snippet-item:hover {
    background: var(--bg-hover);
  }

  .snippet-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .snippet-name {
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 500;
  }

  .snippet-cmd {
    color: var(--text-tertiary);
    font-size: 10px;
    font-family: 'JetBrains Mono', monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .snippet-actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .snippet-item:hover .snippet-actions {
    opacity: 1;
  }

  .act-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .act-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .act-btn.del:hover {
    color: var(--error);
    background: var(--error-bg);
  }

  .empty {
    text-align: center;
    color: var(--text-tertiary);
    font-size: 12px;
    padding: 16px;
  }
</style>
