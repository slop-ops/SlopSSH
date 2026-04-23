<script lang="ts">
  import * as api from '$lib/api/invoke'

  let { onclose }: { onclose: () => void } = $props()

  let name = $state('')
  let host = $state('')
  let port = $state(22)
  let username = $state('')
  let authType = $state('password')
  let password = $state('')
  let keyPath = $state('')
  let saving = $state(false)
  let error = $state('')

  async function save() {
    if (!host.trim() || !username.trim()) {
      error = 'Host and username are required'
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

<div class="overlay" onclick={onclose} role="presentation">
  <div class="dialog" onclick={(e: Event) => e.stopPropagation()} role="dialog">
    <div class="dialog-header">
      <h2>New Session</h2>
      <button class="close-btn" onclick={onclose}>x</button>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <div class="form">
      <label>
        <span>Name</span>
        <input type="text" bind:value={name} placeholder="My Server" />
      </label>

      <div class="row">
        <label class="host">
          <span>Host</span>
          <input type="text" bind:value={host} placeholder="192.168.1.1" />
        </label>
        <label class="port">
          <span>Port</span>
          <input type="number" bind:value={port} min={1} max={65535} />
        </label>
      </div>

      <label>
        <span>Username</span>
        <input type="text" bind:value={username} placeholder="root" />
      </label>

      <label>
        <span>Authentication</span>
        <select bind:value={authType}>
          <option value="password">Password</option>
          <option value="public_key">Public Key</option>
        </select>
      </label>

      {#if authType === 'password'}
        <label>
          <span>Password</span>
          <input type="password" bind:value={password} placeholder="Enter password" />
        </label>
      {:else}
        <label>
          <span>Private Key Path</span>
          <input type="text" bind:value={keyPath} placeholder="~/.ssh/id_rsa" />
        </label>
      {/if}
    </div>

    <div class="dialog-footer">
      <button class="btn-cancel" onclick={onclose}>Cancel</button>
      <button class="btn-save" onclick={save} disabled={saving}>
        {saving ? 'Saving...' : 'Create'}
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
    background: #1e1f2b;
    border: 1px solid #2e303a;
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
    border-bottom: 1px solid #2e303a;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 16px;
    color: #e0e0e0;
  }

  .close-btn {
    background: none;
    border: none;
    color: #9ca3af;
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .error {
    background: #e06c7522;
    color: #e06c75;
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
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  input,
  select {
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 6px;
    padding: 8px 12px;
    color: #e0e0e0;
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.2s;
  }

  input:focus,
  select:focus {
    border-color: #4a90d9;
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
    border-top: 1px solid #2e303a;
  }

  .btn-cancel {
    background: transparent;
    border: 1px solid #2e303a;
    color: #9ca3af;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
  }

  .btn-cancel:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .btn-save {
    background: #4a90d9;
    border: none;
    color: #fff;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
  }

  .btn-save:hover {
    background: #3a7bc8;
  }

  .btn-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
