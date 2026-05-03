<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import type { AuthType } from '$lib/types'

  let { onclose }: { onclose: () => void } = $props()

  let name = $state('')
  let host = $state('')
  let port = $state(22)
  let username = $state('')
  let authType = $state<AuthType>('password')
  let password = $state('')
  let keyPath = $state('')
  let saving = $state(false)
  let error = $state('')

  async function save() {
    if (!host.trim() || !username.trim()) {
      error = t('session.hostRequired')
      return
    }
    saving = true
    error = ''
    try {
      await api.createSession({
        name: name.trim() || host.trim(),
        host: host.trim(),
        port,
        username: username.trim(),
        auth_type: authType,
        password_key: authType === 'password' && password ? password : undefined,
        private_key_path: authType === 'public_key' && keyPath ? keyPath : undefined,
      })
      onclose()
    } catch (e) {
      error = String(e)
    } finally {
      saving = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose()
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay" onclick={onclose} onkeydown={(e) => { if (e.key === 'Escape') onclose() }} role="button" tabindex={0}>
  <div class="dialog" onclick={(e: Event) => e.stopPropagation()} onkeydown={(e: KeyboardEvent) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="New session" tabindex={-1}>
    <div class="dialog-header">
      <h2>{t('session.newSession')}</h2>
      <button class="close-btn" onclick={onclose}>x</button>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <div class="form">
      <label>
        <span>{t('session.name')}</span>
        <input type="text" bind:value={name} placeholder={t('session.name')} />
      </label>

      <div class="row">
        <label class="host">
          <span>{t('session.host')}</span>
          <input type="text" bind:value={host} placeholder="192.168.1.1" />
        </label>
        <label class="port">
          <span>{t('session.port')}</span>
          <input type="number" bind:value={port} min={1} max={65535} />
        </label>
      </div>

      <label>
        <span>{t('session.username')}</span>
        <input type="text" bind:value={username} placeholder={t('session.username')} />
      </label>

      <label>
        <span>{t('session.authType')}</span>
        <select bind:value={authType}>
          <option value="password">{t('session.password')}</option>
          <option value="public_key">{t('session.publicKey')}</option>
        </select>
      </label>

      {#if authType === 'password'}
        <label>
          <span>{t('session.password')}</span>
          <input type="password" bind:value={password} placeholder={t('session.enterPassword')} />
        </label>
      {:else}
        <label>
          <span>{t('session.keyPath')}</span>
          <input type="text" bind:value={keyPath} placeholder="~/.ssh/id_rsa" />
        </label>
      {/if}
    </div>

    <div class="dialog-footer">
      <button class="btn-cancel" onclick={onclose}>{t('session.cancel')}</button>
      <button class="btn-save" onclick={save} disabled={saving}>
        {saving ? t('session.saving') : t('session.create')}
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
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
    border-radius: 12px;
    width: 420px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-primary);
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
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .error {
    background: var(--error-bg);
    color: var(--error);
    padding: 8px 20px;
    font-size: 13px;
  }

  .form {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  label span {
    font-size: 12px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  input,
  select {
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: 6px;
    padding: 8px 12px;
    color: var(--text-primary);
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.2s;
  }

  input:focus,
  select:focus {
    border-color: var(--border-active);
  }

  .row {
    display: flex;
    gap: 12px;
  }

  .host {
    flex: 1;
  }

  .port {
    width: 100px;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid var(--border-primary);
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
  }

  .btn-cancel:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .btn-save {
    background: var(--accent);
    border: none;
    color: #fff;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
  }

  .btn-save:hover {
    background: var(--accent-hover);
  }

  .btn-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
