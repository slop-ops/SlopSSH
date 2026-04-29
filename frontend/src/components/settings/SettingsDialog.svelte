<script lang="ts">
  import { onDestroy } from 'svelte'
  import * as api from '$lib/api/invoke'
  import { setTheme, persistTheme, setTerminalSettings } from '$lib/stores/theme'
  import { loadLocale, t } from '$lib/utils/i18n'
  import type { Settings, EditorInfo, PluginInfo } from '$lib/types'

  let { open = $bindable() }: { open: boolean } = $props()

  let settings = $state<Settings | null>(null)
  let loading = $state(false)
  let saving = $state(false)
  let error = $state('')
  let success = $state('')
  let activeTab = $state('general')
  let successTimer: ReturnType<typeof setTimeout> | undefined

  const tabs = [
    { id: 'general', get label() { return t('settings.general') } },
    { id: 'terminal', get label() { return t('settings.terminal') } },
    { id: 'files', get label() { return t('settings.fileBrowser') } },
    { id: 'connection', get label() { return t('settings.connection') } },
    { id: 'editor', get label() { return t('settings.editor') } },
    { id: 'plugins', get label() { return t('settings.plugins') } },
  ]

  let detectedEditors: EditorInfo[] = $state([])
  let plugins: PluginInfo[] = $state([])
  let pluginLoading = $state(false)
  let selectedPluginId = $state<string | null>(null)
  let pluginSettingsMap = $state<Record<string, Record<string, string>>>({})
  let newSettingKey = $state('')
  let newSettingValue = $state('')

  $effect(() => {
    if (open) {
      loadSettings()
      loadEditors()
      loadPlugins()
    }
  })

  async function loadEditors() {
    try {
      detectedEditors = await api.detectEditors()
    } catch (e) {
      error = String(e)
    }
  }

  async function loadPlugins() {
    pluginLoading = true
    try {
      await api.pluginDiscover()
      plugins = await api.pluginList()
    } catch (e) {
      error = String(e)
    } finally {
      pluginLoading = false
    }
  }

  async function togglePlugin(pluginId: string, enabled: boolean) {
    try {
      await api.pluginSetEnabled(pluginId, enabled)
      plugins = await api.pluginList()
    } catch (e) {
      error = String(e)
    }
  }

  async function removePlugin(pluginId: string) {
    try {
      await api.pluginRemove(pluginId)
      if (selectedPluginId === pluginId) selectedPluginId = null
      plugins = await api.pluginList()
    } catch (e) {
      error = String(e)
    }
  }

  async function selectPlugin(pluginId: string) {
    selectedPluginId = pluginId
    try {
      pluginSettingsMap[pluginId] = await api.pluginGetAllSettings(pluginId)
      pluginSettingsMap = { ...pluginSettingsMap }
    } catch (e) {
      error = String(e)
    }
  }

  async function savePluginSetting() {
    if (!selectedPluginId || !newSettingKey.trim()) return
    try {
      await api.pluginSetSetting(selectedPluginId, newSettingKey.trim(), newSettingValue)
      newSettingKey = ''
      newSettingValue = ''
      await selectPlugin(selectedPluginId)
    } catch (e) {
      error = String(e)
    }
  }

  async function deletePluginSetting(key: string) {
    if (!selectedPluginId) return
    try {
      await api.pluginSetSetting(selectedPluginId, key, '')
      await selectPlugin(selectedPluginId)
    } catch (e) {
      error = String(e)
    }
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
      if (settings.language) {
        await loadLocale(settings.language)
      }
      setTerminalSettings({
        font_family: settings.font_family,
        font_size: settings.font_size,
        terminal_scrollback: settings.terminal_scrollback,
        terminal_copy_on_select: settings.terminal_copy_on_select,
      })
      success = t('settings.saved')
      if (successTimer) clearTimeout(successTimer)
      successTimer = setTimeout(() => (success = ''), 2000)
    } catch (e) {
      error = String(e)
    } finally {
      saving = false
    }
  }

  onDestroy(() => {
    if (successTimer) clearTimeout(successTimer)
  })
</script>

{#if open && settings}
  <div class="backdrop" onclick={(e) => { if (e.target === e.currentTarget) open = false }} role="dialog" aria-modal="true" aria-label={t('settings.title')}>
    <div class="dialog" role="document">
      <div class="dialog-header">
        <h3>{t('settings.title')}</h3>
        <button class="close-btn" onclick={() => (open = false)}>x</button>
      </div>

      <div class="dialog-body">
        <div class="tabs" role="tablist">
          {#each tabs as tab}
            <button
              class="tab"
              class:active={activeTab === tab.id}
              onclick={() => (activeTab = tab.id)}
              role="tab"
              aria-selected={activeTab === tab.id}
              aria-controls="tabpanel-{tab.id}"
            >
              {tab.label}
            </button>
          {/each}
        </div>

        <div class="content" role="tabpanel" id="tabpanel-{activeTab}">
          {#if activeTab === 'general'}
            <div class="field">
              <label>{t('settings.language')}</label>
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
              <label>{t('settings.theme')}</label>
              <select bind:value={settings.theme}>
                <option value="dark">{t('settings.dark')}</option>
                <option value="light">{t('settings.light')}</option>
              </select>
            </div>
            <div class="field">
              <label>{t('settings.logLevel')}</label>
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
              <label>{t('settings.fontFamily')}</label>
              <input type="text" bind:value={settings.font_family} />
            </div>
            <div class="field">
              <label>{t('settings.fontSize')}</label>
              <input type="number" bind:value={settings.font_size} min="8" max="32" />
            </div>
            <div class="field">
              <label>{t('settings.scrollback')}</label>
              <input type="number" bind:value={settings.terminal_scrollback} min="100" max="100000" />
            </div>
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.terminal_copy_on_select} />
                {t('settings.copyOnSelect')}
              </label>
            </div>
          {/if}

          {#if activeTab === 'files'}
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.show_hidden_files} />
                {t('settings.showHidden')}
              </label>
            </div>
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.confirm_before_delete} />
                {t('settings.confirmDelete')}
              </label>
            </div>
            <div class="field">
              <label class="checkbox-label">
                <input type="checkbox" bind:checked={settings.confirm_before_overwrite} />
                {t('settings.confirmOverwrite')}
              </label>
            </div>
            <div class="field">
              <label>{t('settings.defaultEditor')}</label>
              <input type="text" bind:value={settings.default_edit_command} />
            </div>
            <div class="field">
              <label>{t('settings.parallelTransfers')}</label>
              <input type="number" bind:value={settings.transfer_parallel_count} min="1" max="16" />
            </div>
          {/if}

          {#if activeTab === 'connection'}
            <div class="field">
              <label>{t('settings.timeout')}</label>
              <input type="number" bind:value={settings.connection_timeout_secs} min="5" max="300" />
            </div>
            <div class="field">
              <label>{t('settings.keepAlive')}</label>
              <input type="number" bind:value={settings.keep_alive_interval_secs} min="0" max="600" />
            </div>
          {/if}

          {#if activeTab === 'editor'}
            <div class="field">
              <label>{t('settings.externalEditor')}</label>
              <input type="text" bind:value={settings.external_editor} placeholder={t('settings.autoDetected')} />
              <span class="hint">Leave empty for auto-detection. Enter command name or full path.</span>
            </div>
            {#if detectedEditors.length > 0}
              <div class="field">
                <label>{t('settings.detectedEditors')}</label>
                <div class="editor-list">
                  {#each detectedEditors as editor}
                    <button
                      class="editor-item"
                      class:selected={settings.external_editor === editor.command}
                      onclick={() => { if (settings) settings.external_editor = editor.command }}
                    >
                      <span class="editor-name">{editor.name}</span>
                      <span class="editor-path">{editor.path || editor.command}</span>
                    </button>
                  {/each}
                </div>
              </div>
            {:else}
              <div class="field">
                <span class="hint">{t('settings.noEditors')}</span>
              </div>
            {/if}
          {/if}

          {#if activeTab === 'plugins'}
            <div class="plugin-container">
              {#if pluginLoading}
                <div class="hint">{t('settings.loadingPlugins')}</div>
              {:else if plugins.length === 0}
                <div class="field">
                  <span class="hint">{t('settings.noPlugins')}</span>
                </div>
                <button class="cancel-btn" onclick={loadPlugins}>{t('settings.rescan')}</button>
              {:else}
                <div class="plugin-layout">
                  <div class="plugin-sidebar">
                    {#each plugins as plugin}
                      <button
                        class="plugin-item"
                        class:selected={selectedPluginId === plugin.id}
                        onclick={() => selectPlugin(plugin.id)}
                      >
                        <div class="plugin-item-header">
                          <span class="plugin-name">{plugin.name}</span>
                          <label class="toggle-label">
                            <input
                              type="checkbox"
                              checked={plugin.enabled}
                              onchange={() => togglePlugin(plugin.id, !plugin.enabled)}
                            />
                          </label>
                        </div>
                        <span class="plugin-version">v{plugin.version}</span>
                      </button>
                    {/each}
                    <button class="cancel-btn" style="margin-top: 8px; width: 100%;" onclick={loadPlugins}>{t('settings.rescan')}</button>
                  </div>
                  <div class="plugin-detail">
                    {#if selectedPluginId}
                      {@const selected = plugins.find((p) => p.id === selectedPluginId)}
                      {#if selected}
                        <div class="plugin-detail-header">
                          <div>
                            <h4 class="plugin-detail-name">{selected.name}</h4>
                            <span class="hint">{t('settings.by', { author: selected.author || 'Unknown' })} &middot; v{selected.version}</span>
                          </div>
                          <button class="danger-btn" onclick={() => removePlugin(selected.id)}>{t('settings.remove')}</button>
                        </div>
                        {#if selected.description}
                          <p class="plugin-desc">{selected.description}</p>
                        {/if}
                        <div class="plugin-capabilities">
                          <label>{t('settings.capabilities')}</label>
                          <div class="cap-list">
                            {#each (selected.capabilities || []) as cap}
                              <span class="cap-badge">{cap}</span>
                            {/each}
                          </div>
                        </div>
                        <div class="plugin-settings-section">
                          <label>{t('settings.pluginSettings')}</label>
                          {#if pluginSettingsMap[selectedPluginId]}
                            {#each Object.entries(pluginSettingsMap[selectedPluginId]) as [key, value]}
                              {#if value}
                                <div class="plugin-setting-row">
                                  <span class="setting-key">{key}</span>
                                  <span class="setting-value">{value}</span>
                                  <button class="icon-btn" onclick={() => deletePluginSetting(key)}>x</button>
                                </div>
                              {/if}
                            {/each}
                          {:else}
                            <span class="hint">{t('settings.noSettings')}</span>
                          {/if}
                          <div class="add-setting-row">
                            <input
                              type="text"
                              placeholder={t('settings.key')}
                              bind:value={newSettingKey}
                            />
                            <input
                              type="text"
                              placeholder={t('settings.value')}
                              bind:value={newSettingValue}
                            />
                            <button class="save-btn" onclick={savePluginSetting} disabled={!newSettingKey.trim()}>{t('settings.add')}</button>
                          </div>
                        </div>
                      {/if}
                    {:else}
                      <div class="hint">{t('settings.selectPlugin')}</div>
                    {/if}
                  </div>
                </div>
              {/if}
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
            {saving ? t('settings.saving') : t('settings.save')}
          </button>
          <button class="cancel-btn" onclick={() => (open = false)}>{t('settings.close')}</button>
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

  .plugin-container {
    min-height: 300px;
  }

  .plugin-layout {
    display: flex;
    gap: 16px;
  }

  .plugin-sidebar {
    width: 180px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
  }

  .plugin-item {
    background: var(--bg-input);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 8px;
    cursor: pointer;
    color: var(--text-primary);
    font-family: inherit;
    text-align: left;
    transition: border-color 0.15s;
  }

  .plugin-item:hover {
    border-color: var(--border-active);
  }

  .plugin-item.selected {
    border-color: var(--accent);
    background: var(--accent-bg);
  }

  .plugin-item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .plugin-name {
    font-size: 12px;
    font-weight: 500;
  }

  .plugin-version {
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .toggle-label {
    cursor: pointer;
  }

  .plugin-detail {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .plugin-detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .plugin-detail-name {
    margin: 0;
    font-size: 14px;
    color: var(--text-primary);
  }

  .plugin-desc {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 0;
  }

  .plugin-capabilities {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .plugin-capabilities label {
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .cap-list {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .cap-badge {
    font-size: 10px;
    padding: 2px 8px;
    border-radius: 3px;
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .plugin-settings-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .plugin-settings-section label {
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .plugin-setting-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    background: var(--bg-input);
    border-radius: 4px;
  }

  .setting-key {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    min-width: 80px;
  }

  .setting-value {
    font-size: 12px;
    color: var(--text-secondary);
    flex: 1;
  }

  .icon-btn {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 12px;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--error);
  }

  .add-setting-row {
    display: flex;
    gap: 4px;
    margin-top: 4px;
  }

  .add-setting-row input {
    background: var(--bg-input);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--text-primary);
    font-size: 12px;
    outline: none;
    flex: 1;
  }

  .add-setting-row input:focus {
    border-color: var(--border-active);
  }

  .danger-btn {
    background: transparent;
    border: 1px solid var(--error);
    color: var(--error);
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
  }

  .danger-btn:hover {
    background: var(--error-bg);
  }
</style>
