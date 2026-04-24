<script lang="ts">
  import * as api from '$lib/api/invoke'

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
  ]

  $effect(() => {
    if (open) loadSettings()
  })

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
    background: #1e1f2b;
    border: 1px solid #2e303a;
    border-radius: 8px;
    width: 520px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #2e303a;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 14px;
    color: #e0e0e0;
  }

  .close-btn {
    background: none;
    border: none;
    color: #6b7280;
    cursor: pointer;
    font-size: 14px;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .dialog-body {
    padding: 16px;
    overflow-y: auto;
  }

  .tabs {
    display: flex;
    gap: 2px;
    margin-bottom: 16px;
    border-bottom: 1px solid #2e303a;
  }

  .tab {
    background: transparent;
    border: none;
    color: #9ca3af;
    padding: 8px 16px;
    cursor: pointer;
    font-size: 12px;
    border-bottom: 2px solid transparent;
    transition: color 0.15s, border-color 0.15s;
  }

  .tab:hover {
    color: #e0e0e0;
  }

  .tab.active {
    color: #4a90d9;
    border-bottom-color: #4a90d9;
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
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .field input[type="text"],
  .field input[type="number"],
  .field select {
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 4px;
    padding: 6px 10px;
    color: #e0e0e0;
    font-size: 13px;
    outline: none;
  }

  .field input:focus,
  .field select:focus {
    border-color: #4a90d9;
  }

  .checkbox-label {
    display: flex !important;
    flex-direction: row !important;
    align-items: center;
    gap: 8px;
    text-transform: none !important;
    font-size: 13px !important;
    color: #e0e0e0 !important;
    cursor: pointer;
  }

  .checkbox-label input[type="checkbox"] {
    accent-color: #4a90d9;
  }

  .error {
    background: #e06c7522;
    color: #e06c75;
    padding: 8px 12px;
    border-radius: 4px;
    font-size: 12px;
    margin-top: 8px;
  }

  .success {
    background: #98c37922;
    color: #98c379;
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
    background: #4a90d9;
    border: none;
    color: #fff;
    padding: 6px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .save-btn:hover {
    background: #3a7bc8;
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .cancel-btn {
    background: transparent;
    border: 1px solid #2e303a;
    color: #9ca3af;
    padding: 6px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .cancel-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }
</style>
