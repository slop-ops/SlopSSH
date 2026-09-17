<script lang="ts">
  import FilePane from './FilePane.svelte'
  import FileEditor from './FileEditor.svelte'
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'
  import type { FileItem } from '$lib/types'

  let { sessionId }: { sessionId: string } = $props()

  let remotePane: any = $state()
  let localPane: any = $state()
  let isDualPane = $state(true)
  let editingFilePath = $state<string | null>(null)
  let statusMessage = $state('')

  async function handleTransferFromLocalToRemote(item: FileItem, targetRemoteDir: string) {
    statusMessage = `Uploading ${item.name}...`
    const sep = targetRemoteDir.endsWith('/') ? '' : '/'
    const destPath = `${targetRemoteDir}${sep}${item.name}`
    try {
      await api.transferUpload(
        crypto.randomUUID(),
        sessionId,
        item.path,
        destPath,
        item.size
      )
      statusMessage = `Upload complete: ${item.name}`
      setTimeout(() => { statusMessage = '' }, 3000)
      remotePane?.loadDir(targetRemoteDir)
    } catch (e) {
      statusMessage = `Upload failed: ${e}`
    }
  }

  async function handleTransferFromRemoteToLocal(item: FileItem, targetLocalDir: string) {
    statusMessage = `Downloading ${item.name}...`
    const sep = targetLocalDir.endsWith('/') ? '' : '/'
    const destPath = `${targetLocalDir}${sep}${item.name}`
    try {
      await api.transferDownload(
        crypto.randomUUID(),
        sessionId,
        item.path,
        destPath,
        item.size
      )
      statusMessage = `Download complete: ${item.name}`
      setTimeout(() => { statusMessage = '' }, 3000)
      localPane?.loadDir(targetLocalDir)
    } catch (e) {
      statusMessage = `Download failed: ${e}`
    }
  }

  function handleQuickUpload() {
    const selected = localPane?.getSelectedFile()
    if (!selected) {
      alert('Select a file in the Local pane first to upload')
      return
    }
    const remoteDir = remotePane?.getCurrentPath() || '/'
    handleTransferFromLocalToRemote(selected, remoteDir)
  }

  function handleQuickDownload() {
    const selected = remotePane?.getSelectedFile()
    if (!selected) {
      alert('Select a file in the Remote pane first to download')
      return
    }
    const localDir = localPane?.getCurrentPath() || '/'
    handleTransferFromRemoteToLocal(selected, localDir)
  }

  function handleRemoteFileOpen(file: FileItem) {
    editingFilePath = file.path
  }
</script>

<div class="dual-pane-browser" role="region" aria-label="SFTP File Browser">
  <!-- Top Toolbar -->
  <div class="browser-top-bar">
    <div class="view-toggle-group">
      <button
        type="button"
        class="toggle-btn"
        class:active={isDualPane}
        onclick={() => (isDualPane = true)}
        title="Dual-Pane Mode (Remote on Left, Local on Right)"
      >
        <span class="btn-icon">&#x25A4;</span> Dual-Pane
      </button>
      <button
        type="button"
        class="toggle-btn"
        class:active={!isDualPane}
        onclick={() => (isDualPane = false)}
        title="Single-Pane Mode (Remote Only)"
      >
        <span class="btn-icon">&#x25A1;</span> Remote Only
      </button>
    </div>

    {#if statusMessage}
      <span class="status-msg">{statusMessage}</span>
    {/if}

    <div class="spacer"></div>

    <div class="transfer-actions">
      {#if isDualPane}
        <button
          type="button"
          class="transfer-btn upload-btn"
          onclick={handleQuickUpload}
          title="Upload selected local file to remote directory"
        >
          <span>&#x2190; Upload</span>
        </button>
        <button
          type="button"
          class="transfer-btn download-btn"
          onclick={handleQuickDownload}
          title="Download selected remote file to local directory"
        >
          <span>Download &#x2192;</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Main Panes Area -->
  <div class="panes-container" class:dual={isDualPane}>
    <!-- Left: Remote SFTP File Pane -->
    <div class="pane-column remote-column">
      <FilePane
        bind:this={remotePane}
        mode="remote"
        {sessionId}
        title="Remote Server (SFTP)"
        onFileOpen={handleRemoteFileOpen}
        onTransferRequest={(item, targetDir) => handleTransferFromRemoteToLocal(item, targetDir)}
      />
    </div>

    <!-- Right: Local Filesystem Pane -->
    {#if isDualPane}
      <div class="center-transfer-divider">
        <button
          type="button"
          class="divider-action-btn"
          onclick={handleQuickUpload}
          title="Upload selected Local file to Remote (Left)"
        >
          &#x2190;
        </button>
        <button
          type="button"
          class="divider-action-btn"
          onclick={handleQuickDownload}
          title="Download selected Remote file to Local (Right)"
        >
          &#x2192;
        </button>
      </div>

      <div class="pane-column local-column">
        <FilePane
          bind:this={localPane}
          mode="local"
          title="Local Filesystem"
          onTransferRequest={(item, targetDir) => handleTransferFromLocalToRemote(item, targetDir)}
        />
      </div>
    {/if}
  </div>
</div>

{#if editingFilePath}
  <FileEditor
    {sessionId}
    filePath={editingFilePath}
    onclose={() => {
      editingFilePath = null
      remotePane?.loadDir(remotePane?.getCurrentPath())
    }}
  />
{/if}

<style>
  .dual-pane-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    background: var(--bg-primary, #15181e);
    overflow: hidden;
  }

  .browser-top-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    background: var(--bg-secondary, #1a1e27);
    border-bottom: 1px solid var(--border-primary, rgba(255, 255, 255, 0.08));
    min-height: 36px;
    flex-shrink: 0;
  }

  .view-toggle-group {
    display: flex;
    align-items: center;
    background: var(--bg-primary, rgba(0, 0, 0, 0.25));
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    border-radius: 5px;
    padding: 2px;
    gap: 2px;
  }

  .toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .toggle-btn:hover {
    color: var(--text-primary, #ffffff);
  }

  .toggle-btn.active {
    background: var(--accent-primary, #6366f1);
    color: #ffffff;
  }

  .btn-icon {
    font-size: 12px;
    line-height: 1;
  }

  .status-msg {
    font-size: 11px;
    color: #10b981;
    background: rgba(16, 185, 129, 0.12);
    border: 1px solid rgba(16, 185, 129, 0.25);
    padding: 2px 8px;
    border-radius: 4px;
  }

  .spacer {
    flex: 1;
  }

  .transfer-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .transfer-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.15s ease;
  }

  .upload-btn {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    border-color: rgba(16, 185, 129, 0.3);
  }

  .upload-btn:hover {
    background: #10b981;
    color: #ffffff;
  }

  .download-btn {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    border-color: rgba(59, 130, 246, 0.3);
  }

  .download-btn:hover {
    background: #3b82f6;
    color: #ffffff;
  }

  .panes-container {
    display: flex;
    flex: 1;
    min-height: 0;
    gap: 6px;
    padding: 6px;
    overflow: hidden;
  }

  .pane-column {
    flex: 1;
    min-width: 0;
    height: 100%;
    display: flex;
  }

  .center-transfer-divider {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 28px;
    flex-shrink: 0;
  }

  .divider-action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    background: var(--bg-secondary, #1a1e27);
    border: 1px solid var(--border-primary, rgba(255, 255, 255, 0.1));
    color: var(--text-secondary, #94a3b8);
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .divider-action-btn:hover {
    background: var(--accent-primary, #6366f1);
    color: #ffffff;
    border-color: var(--accent-primary, #6366f1);
    transform: scale(1.08);
  }
</style>
