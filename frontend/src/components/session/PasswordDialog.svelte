<script lang="ts">
  import { t } from '$lib/utils/i18n'

  let {
    sessionName,
    onsubmit,
    oncancel,
  }: {
    sessionName: string
    onsubmit: (password: string, save: boolean) => void
    oncancel: () => void
  } = $props()

  let password = $state('')
  let savePassword = $state(true)

  function handleSubmit(e?: Event) {
    e?.preventDefault()
    if (password) {
      onsubmit(password, savePassword)
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') oncancel()
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="overlay" onclick={oncancel} onkeydown={(e) => { if (e.key === 'Escape') oncancel() }} role="button" tabindex={0}>
  <div class="dialog" onclick={(e: Event) => e.stopPropagation()} onkeydown={(e: KeyboardEvent) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label={t('session.enterPassword')} tabindex={-1}>
    <div class="dialog-header">
      <h2>{t('session.enterPassword')}</h2>
      <button class="close-btn" onclick={oncancel}>x</button>
    </div>

    <form class="form" onsubmit={handleSubmit}>
      <p class="session-label">{sessionName}</p>
      <label>
        <span>{t('session.password')}</span>
        <input
          type="password"
          bind:value={password}
          placeholder={t('session.enterPassword')}
          autofocus
        />
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={savePassword} />
        <span>{t('session.savePassword')}</span>
      </label>
    </form>

    <div class="dialog-footer">
      <button class="btn-cancel" onclick={oncancel}>{t('session.cancel')}</button>
      <button class="btn-save" onclick={() => handleSubmit()} disabled={!password}>
        {t('session.connect')}
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
    width: 380px;
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

  .form {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .session-label {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
    font-weight: 500;
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

  input[type="password"] {
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

  input[type="password"]:focus {
    border-color: var(--border-active);
  }

  .checkbox-label {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .checkbox-label span {
    font-size: 13px;
    color: var(--text-secondary);
    text-transform: none;
    letter-spacing: normal;
  }

  .checkbox-label input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
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
