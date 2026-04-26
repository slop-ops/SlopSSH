<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let {
    sessionId,
  }: {
    sessionId: string
  } = $props()

  let localKeys = $state<any[]>([])
  let remoteKeys = $state<any[]>([])
  let loading = $state(false)
  let error = $state('')
  let showGenerate = $state(false)
  let genAlgorithm = $state('ed25519')
  let genName = $state('id_muon')
  let genPassphrase = $state('')
  let genPath = $state('')
  let generating = $state(false)
  let deploying = $state<string | null>(null)
  let successMsg = $state('')

  $effect(() => {
    loadKeys()
  })

  async function loadKeys() {
    loading = true
    error = ''
    try {
      const [local, remote] = await Promise.all([
        api.listLocalKeys(),
        sessionId ? api.listRemoteKeys(sessionId) : Promise.resolve([]),
      ])
      localKeys = local
      remoteKeys = remote
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  async function generateKey() {
    generating = true
    error = ''
    try {
      const homeDir = '~/.ssh'
      const path = genPath || `${homeDir}/${genName}`
      await api.generateKeyPair(genAlgorithm, path, genPassphrase || undefined)
      showGenerate = false
      genName = 'id_muon'
      genPassphrase = ''
      genPath = ''
      successMsg = 'Key generated successfully'
      setTimeout(() => (successMsg = ''), 3000)
      await loadKeys()
    } catch (e) {
      error = String(e)
    } finally {
      generating = false
    }
  }

  async function deployKey(keyPath: string) {
    deploying = keyPath
    error = ''
    try {
      const pubKey = await api.readPublicKey(keyPath)
      await api.deployPublicKey(sessionId, pubKey)
      successMsg = 'Key deployed to remote server'
      setTimeout(() => (successMsg = ''), 3000)
      await loadKeys()
    } catch (e) {
      error = String(e)
    } finally {
      deploying = null
    }
  }
</script>

<div class="key-manager">
  <div class="section-header">
    <h3>{t('tools.keyManager')}</h3>
    <div class="actions">
      <button class="btn" onclick={loadKeys}>{t('tools.refresh')}</button>
      <button class="btn btn-primary" onclick={() => (showGenerate = true)}>{t('tools.generateKey')}</button>
    </div>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if successMsg}
    <div class="success">{successMsg}</div>
  {/if}

  {#if loading}
    <div class="loading">{t('tools.loadingKeys')}</div>
  {:else}
    <div class="keys-section">
      <h4>{t('tools.localKeys')}</h4>
      {#if localKeys.length === 0}
        <div class="empty">{t('tools.noLocalKeys')}</div>
      {:else}
        <div class="key-list">
          {#each localKeys as key}
            <div class="key-item">
              <div class="key-info">
                <span class="key-name">{key.name}</span>
                <span class="key-type">{key.keyType}</span>
                {#if key.fingerprint}
                  <span class="key-fp" title={key.fingerprint}>{key.fingerprint.substring(0, 40)}...</span>
                {/if}
              </div>
              <div class="key-actions">
                {#if key.hasPublicKey && sessionId}
                  <button
                    class="btn btn-sm"
                    disabled={deploying === key.path}
                    onclick={() => deployKey(key.path)}
                  >
                    {deploying === key.path ? t('tools.deploying') : t('tools.deploy')}
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    {#if sessionId}
      <div class="keys-section">
        <h4>{t('tools.remoteKeys', { count: String(remoteKeys.length) })}</h4>
        {#if remoteKeys.length === 0}
          <div class="empty">{t('tools.noRemoteKeys')}</div>
        {:else}
          <div class="key-list">
            {#each remoteKeys as key}
              <div class="key-item">
                <div class="key-info">
                  <span class="key-name">{key.name}</span>
                  <span class="key-type">{key.keyType}</span>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  {/if}

  {#if showGenerate}
    <div class="dialog-overlay" onclick={() => (showGenerate = false)}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>{t('tools.generateTitle')}</h3>

        <div class="form-group">
          <label for="gen-algorithm">{t('tools.algorithm')}</label>
          <select id="gen-algorithm" bind:value={genAlgorithm}>
            <option value="ed25519">Ed25519</option>
            <option value="rsa">RSA 4096</option>
            <option value="ecdsa">ECDSA 521</option>
          </select>
        </div>

        <div class="form-group">
          <label for="gen-name">{t('tools.keyName')}</label>
          <input id="gen-name" type="text" bind:value={genName} placeholder="id_muon" />
        </div>

        <div class="form-group">
          <label for="gen-path">{t('tools.fullPath')}</label>
          <input id="gen-path" type="text" bind:value={genPath} placeholder="~/.ssh/id_muon" />
        </div>

        <div class="form-group">
          <label for="gen-passphrase">{t('tools.passphrase')}</label>
          <input id="gen-passphrase" type="password" bind:value={genPassphrase} />
        </div>

        <div class="dialog-actions">
          <button class="btn" onclick={() => (showGenerate = false)}>{t('common.cancel')}</button>
          <button class="btn btn-primary" disabled={generating} onclick={generateKey}>
            {generating ? t('tools.generating') : t('tools.generate')}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .key-manager {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
    overflow-y: auto;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .section-header h3 {
    margin: 0;
    font-size: 14px;
    color: var(--text-primary);
  }

  .actions {
    display: flex;
    gap: 6px;
  }

  .btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    transition: background 0.15s;
  }

  .btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-sm {
    padding: 2px 8px;
    font-size: 11px;
  }

  .keys-section h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .key-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .key-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--bg-tertiary);
    border-radius: 6px;
    border: 1px solid var(--border-primary);
  }

  .key-info {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .key-name {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .key-type {
    color: var(--accent);
    font-size: 11px;
    background: var(--accent-bg);
    padding: 1px 6px;
    border-radius: 3px;
  }

  .key-fp {
    color: var(--text-tertiary);
    font-size: 11px;
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .key-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .empty {
    color: var(--text-tertiary);
    font-size: 13px;
    padding: 12px;
    text-align: center;
  }

  .loading {
    color: var(--text-tertiary);
    text-align: center;
    padding: 24px;
  }

  .error {
    background: var(--error-bg);
    color: var(--error);
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
  }

  .success {
    background: var(--success-bg);
    color: var(--success);
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
  }

  .dialog-overlay {
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
    border-radius: 10px;
    padding: 20px;
    min-width: 360px;
    max-width: 480px;
  }

  .dialog h3 {
    margin: 0 0 16px 0;
    font-size: 15px;
    color: var(--text-primary);
  }

  .form-group {
    margin-bottom: 12px;
  }

  .form-group label {
    display: block;
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .form-group input,
  .form-group select {
    width: 100%;
    box-sizing: border-box;
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: 6px;
    padding: 6px 10px;
    color: var(--text-primary);
    font-size: 13px;
    font-family: inherit;
    outline: none;
  }

  .form-group input:focus,
  .form-group select:focus {
    border-color: var(--border-active);
  }

  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
</style>
