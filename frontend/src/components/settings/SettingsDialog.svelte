<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { setTheme, persistTheme } from '$lib/stores/theme'

  let { open = $bindable() }: { open: boolean } = $props()

  let settings = $state<any>(null)
  let loading = $state(false)
  let saving = $state(false)
  let error = $state('')
  let success = $state('')
  let activeTab = $state('general')

  const tabs = [
    { id: 'general', label: 'General' },
    { id: 'terminal', label: 'Terminal' },
    { id: 'files', label: 'File Browser' },
    { id: 'connection', label: 'Connection' },
    { id: 'editor', label: 'Editor' },
  ]

  let detectedEditors: any[] = $state([])

  $effect(() => {
    if (open) {
      loadSettings()
      loadEditors()
    }
  })

  async function loadEditors() {
    try {
      detectedEditors = await api.detectEditors()
    } catch {}
  }

  async function loadSettings() {
    loading = true
    try {
      settings = await api.getSettings()
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function save() {
    if (!settings) return
    saving = true
    error = ''
    success = ''
    try {
      await api.saveSettings(settings)
      if (settings.theme) {
        setTheme(settings.theme)
        persistTheme(settings.theme)
      }
      success = 'Settings saved'
      setTimeout(() => (success = ''), 2000)
    } catch (e) {
      error = String(e)
    } finally {
      saving = false
    }
  }
</script>

{#if open && settings}
  <div class="backdrop" onclick={(e) => { if (e.target === e.currentTarget) open = false }}>
    <div class="dialog">
      <div class="dialog-header">
        <h3>Settings</h3>
        <button class="close-btn" onclick={() => (open = false)}>x</button>
      </div>

      <div class="dialog-body">
        <div class="tabs">
          {#each tabs as tab}
            <button
              class="tab"
              class:active={activeTab === tab.id}
              onclick={() => (activeTab = tab.id)}
            >
              {tab.label}
            </button>
          {/each}
        </div>

        <div class="content">
          {#if activeTab === 'general'}
            <div class="field">
              <label>Language</label>
              <select bind:value={settings.language}>
                <option value="en">English</option>
                <option value="es">Spanish</option>
                <option value="ru">Russian</option>
                <option value="fr">French</option>
                <option value="de">German</option>
                <option value="pt">Portuguese</option>
                <option value="cn">Chinese</option>
              </select>
            </div>
            <div class="field">
              <label>Theme</label>
              <select bind:value={settings.theme}>
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>
            <div class="field">
              <label>Log Level</label>
              <select bind:value={settings.log_level}>
                <option value="trace">Trace</option>
                <option value="debug">Debug</option>
                <option value="info">Info</option>
                <option value="warn">Warning</option>
                <option value="error">Error</option>
              </select>
            </div>
          {/if}

          {#if activeTab === 'terminal'}
            <div class="field">
              <label>Font Family</label>
              <input type="text" bind:value={settings.font_family} />
            </div>
            <div class="field">
              <label>Font Size</label>
              <input type="number" bind:value={settings.font_size} min="8" max="32" />
            </div>
            <div class="field">
              <label>Scrollback Lines</label>
              <input type="number" bind:value={settings.terminal_scrollback} min="100" max="100000" />
            </div>
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.terminal_copy_on_select} />
                Copy on select
              </label>
            </div>
          {/if}

          {#if activeTab === 'files'}
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.show_hidden_files} />
                Show hidden files
              </label>
            </div>
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.confirm_before_delete} />
                Confirm before delete
              </label>
            </div>
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.confirm_before_overwrite} />
                Confirm before overwrite
              </label>
            </div>
            <div class="field">
              <label>Default Editor</label>
              <input type="text" bind:value={settings.default_edit_command} />
            </div>
            <div class="field">
              <label>Parallel Transfers</label>
              <input type="number" bind:value={settings.transfer_parallel_count} min="1" max="16" />
            </div>
          {/if}

          {#if activeTab === 'connection'}
            <div class="field">
              <label>Connection Timeout (seconds)</label>
              <input type="number" bind:value={settings.connection_timeout_secs} min="5" max="300" />
            </div>
            <div class="field">
              <label>Keep-Alive Interval (seconds)</label>
              <input type="number" bind:value={settings.keep_alive_interval_secs} min="0" max="600" />
            </div>
          {/if}

          {#if activeTab === 'editor'}
            <div class="field">
              <label>External Editor</label>
              <input type="text" bind:value={settings.external_editor} placeholder="Auto-detected" />
              <span class="hint">Leave empty for auto-detection. Enter command name or full path.</span>
            </div>
            {#if detectedEditors.length > 0}
              <div class="field">
                <label>Detected Editors</label>
                <div class="editor-list">
                  {#each detectedEditors as editor}
                    <button
                      class="editor-item"
                      class:selected={settings.external_editor === editor.command}
                      onclick={() => (settings.external_editor = editor.command)}
                    >
                      <span class="editor-name">{editor.name}</span>
                      <span class="editor-path">{editor.path || editor.command}</span>
                    </button>
                  {/each}
                </div>
              </div>
            {:else}
              <div class="field">
                <span class="hint">No editors detected on this system.</span>
              </div>
            {/if}
          {/if}
        </div>

        {#if error}
          <div class="error">{error}</div>
        {/if}
        {#if success}
          <div class="success">{success}</div>
        {/if}

        <div class="actions">
          <button class="save-btn" onclick={save} disabled={saving}>
            {saving ? 'Saving...' : 'Save'}
          </button>
          <button class="cancel-btn" onclick={() => (open = false)}>Close</button>
        </div>
      </div>
    </div>
  </div>
{/if}

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
    width: 520px;
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

  .dialog-body {
    padding: 16px;
    overflow-y: auto;
  }

  .tabs {
    display: flex;
    gap: 2px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border-primary);
  }

  .tab {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 8px 16px;
    cursor: pointer;
    font-size: 12px;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab:hover {
    color: var(--text-primary);
  }

  .tab.active {
    color: var(--accent-text);
    border-bottom-color: var(--accent);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field label {
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .field input[type="text"],
  .field input[type="number"],
  .field select {
    background: var(--bg-input);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 6px 10px;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
  }

  .field input:focus,
  .field select:focus {
    border-color: var(--border-active);
  }

  .checkbox-label {
    display: flex !important;
    flex-direction: row !important;
    align-items: center;
    gap: 8px;
    text-transform: none !important;
    font-size: 13px !important;
    color: var(--text-primary) !important;
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    accent-color: var(--accent);
  }

  .error {
    background: var(--error-bg);
    color: var(--error);
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 12px;
    margin-top: 8px;
  }

  .success {
    background: var(--success-bg);
    color: var(--success);
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 12px;
    margin-top: 8px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }

  .save-btn {
    background: var(--accent);
    border: none;
    color: var(--text-inverse);
    padding: 6px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .save-btn:hover {
    background: var(--accent-hover);
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .cancel-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 6px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .hint {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .editor-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .editor-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--bg-input);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 6px 10px;
    cursor: pointer;
    color: var(--text-primary);
    font-size: 12px;
    font-family: inherit;
    text-align: left;
    transition: border-color 0.15s;
  }

  .editor-item:hover {
    border-color: var(--border-active);
  }

  .editor-item.selected {
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  .editor-name {
    font-weight: 500;
  }

  .editor-path {
    font-size: 10px;
    color: var(--text-tertiary);
  }
</style>
