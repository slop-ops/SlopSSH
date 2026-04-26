<script lang="ts">
  import { importSshConfig, importSshConfigToFolder, listSessions } from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let preview: any[] = $state([])
  let loading = $state(false)
  let imported = $state(false)
  let error = $state('')

  async function previewConfig() {
    loading = true
    error = ''
    preview = []
    try {
      preview = await importSshConfig()
    } catch (e: any) {
      error = e.toString()
    }
    loading = false
  }

  async function doImport() {
    loading = true
    error = ''
    try {
      await importSshConfigToFolder()
      imported = true
      preview = []
    } catch (e: any) {
      error = e.toString()
    }
    loading = false
  }

  let { onClose }: { onClose: () => void } = $props()
</script>

<div class="dialog-overlay" onclick={onClose}>
  <div class="dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Import SSH config">
    <div class="dialog-header">
      <h2>{t('session.importSshConfig')}</h2>
      <button class="close-btn" onclick={onClose}>&times;</button>
    </div>

    <div class="dialog-body">
      {#if error}
        <div class="error">{error}</div>
      {/if}

      {#if imported}
        <div class="success">
          {t('session.importSuccess')}
        </div>
      {:else if preview.length === 0}
        <p class="description">
          {t('session.importDescription')}
        </p>
        <button class="btn-primary" onclick={previewConfig} disabled={loading}>
          {loading ? t('session.scanning') : t('session.scanConfig')}
        </button>
      {:else}
        <p class="description">
          {t('session.foundHosts', { count: String(preview.length) })}
        </p>
        <div class="preview-list">
          {#each preview as item}
            <div class="preview-item">
              <span class="host-name">{item.name || item.host}</span>
              <span class="host-detail">{item.username}@{item.host}:{item.port}</span>
              <span class="auth-badge">{item.auth_type}</span>
            </div>
          {/each}
        </div>
        <div class="actions">
          <button class="btn-secondary" onclick={() => (preview = [])}>{t('session.cancel')}</button>
          <button class="btn-primary" onclick={doImport} disabled={loading}>
            {loading ? t('session.importing') : t('session.importCount', { count: String(preview.length) })}
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .dialog {
    background: var(--bg-primary);
    border-radius: 8px;
    width: 500px;
    max-height: 80vh;
    overflow: auto;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }
  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }
  .dialog-header h2 {
    margin: 0;
    font-size: 16px;
    color: var(--text-primary);
  }
  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 20px;
    cursor: pointer;
    padding: 4px 8px;
  }
  .close-btn:hover {
    color: var(--text-primary);
  }
  .dialog-body {
    padding: 20px;
  }
  .description {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 16px;
  }
  .description code {
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 12px;
  }
  .btn-primary {
    padding: 8px 16px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }
  .btn-primary:disabled {
    opacity: 0.6;
  }
  .btn-secondary {
    padding: 8px 16px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }
  .preview-list {
    max-height: 300px;
    overflow-y: auto;
    margin-bottom: 16px;
  }
  .preview-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }
  .host-name {
    font-weight: 600;
    color: var(--text-primary);
    min-width: 120px;
  }
  .host-detail {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary);
    flex: 1;
  }
  .auth-badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--bg-secondary);
    color: var(--text-secondary);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .error {
    color: var(--error);
    font-size: 13px;
    padding: 8px;
    background: rgba(255, 0, 0, 0.1);
    border-radius: 4px;
    margin-bottom: 12px;
  }
  .success {
    color: #4caf50;
    font-size: 13px;
    padding: 12px;
    background: rgba(76, 175, 80, 0.1);
    border-radius: 4px;
    text-align: center;
  }
</style>
