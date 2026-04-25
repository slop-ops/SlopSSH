import { invoke } from '@tauri-apps/api/core'

export async function getSettings(): Promise<any> {
  return invoke('get_settings')
}

export async function saveSettings(settings: any): Promise<void> {
  return invoke('save_settings', { settings })
}

export async function listSessions(): Promise<any> {
  return invoke('list_sessions')
}

export async function createSession(session: any): Promise<string> {
  return invoke<string>('create_session', { session })
}

export async function updateSession(session: any): Promise<void> {
  return invoke('update_session', { session })
}

export async function deleteSession(sessionId: string): Promise<void> {
  return invoke('delete_session', { sessionId })
}

export async function createFolder(name: string, parentId?: string): Promise<string> {
  return invoke<string>('create_folder', { name, parentId: parentId ?? null })
}

export async function sshConnect(sessionId: string, password?: string): Promise<{ sessionId: string; hostKeyStatus: string; hostKeyFingerprint: string | null }> {
  return invoke('ssh_connect', { sessionId, password: password ?? null })
}

export async function acceptHostKey(sessionId: string): Promise<void> {
  return invoke('accept_host_key', { sessionId })
}

export async function sshDisconnect(sessionId: string): Promise<void> {
  return invoke('ssh_disconnect', { sessionId })
}

export async function sshOpenShell(sessionId: string, channelId: string, cols: number, rows: number): Promise<void> {
  return invoke('ssh_open_shell', { sessionId, channelId, cols, rows })
}

export async function sshWriteShell(sessionId: string, channelId: string, data: string): Promise<void> {
  return invoke('ssh_write_shell', { sessionId, channelId, data })
}

export async function sshResizeShell(sessionId: string, channelId: string, cols: number, rows: number): Promise<void> {
  return invoke('ssh_resize_shell', { sessionId, channelId, cols, rows })
}

export async function sshCloseShell(sessionId: string, channelId: string): Promise<void> {
  return invoke('ssh_close_shell', { sessionId, channelId })
}

export async function sftpConnect(sessionId: string): Promise<void> {
  return invoke('sftp_connect', { sessionId })
}

export async function sftpDisconnect(sessionId: string): Promise<void> {
  return invoke('sftp_disconnect', { sessionId })
}

export async function sftpListDir(sessionId: string, path: string): Promise<any[]> {
  return invoke('sftp_list_dir', { sessionId, path })
}

export async function sftpMkdir(sessionId: string, path: string): Promise<void> {
  return invoke('sftp_mkdir', { sessionId, path })
}

export async function sftpRemove(sessionId: string, path: string): Promise<void> {
  return invoke('sftp_remove', { sessionId, path })
}

export async function sftpRename(sessionId: string, from: string, to: string): Promise<void> {
  return invoke('sftp_rename', { sessionId, from, to })
}

export async function sftpReadFile(sessionId: string, path: string): Promise<string> {
  return invoke<string>('sftp_read_file', { sessionId, path })
}

export async function sftpWriteFile(sessionId: string, path: string, data: string): Promise<void> {
  return invoke('sftp_write_file', { sessionId, path, data })
}

export async function sftpStat(sessionId: string, path: string): Promise<any> {
  return invoke('sftp_stat', { sessionId, path })
}

export async function sftpHome(sessionId: string): Promise<string> {
  return invoke<string>('sftp_home', { sessionId })
}

export async function getAppVersion(): Promise<string> {
  return invoke<string>('get_app_version')
}

export async function listSnippets(): Promise<any[]> {
  return invoke('list_snippets')
}

export async function createSnippet(snippet: any): Promise<string> {
  return invoke<string>('create_snippet', { snippet })
}

export async function updateSnippet(snippet: any): Promise<void> {
  return invoke('update_snippet', { snippet })
}

export async function deleteSnippet(snippetId: string): Promise<void> {
  return invoke('delete_snippet', { snippetId })
}

export async function transferUpload(
  transferId: string,
  sessionId: string,
  localPath: string,
  remotePath: string,
  fileSize: number,
): Promise<void> {
  return invoke('transfer_upload', { transferId, sessionId, localPath, remotePath, fileSize })
}

export async function transferDownload(
  transferId: string,
  sessionId: string,
  remotePath: string,
  localPath: string,
  fileSize: number,
): Promise<void> {
  return invoke('transfer_download', { transferId, sessionId, remotePath, localPath, fileSize })
}

export async function transferCancel(transferId: string): Promise<boolean> {
  return invoke<boolean>('transfer_cancel', { transferId })
}

export async function transferList(): Promise<any[]> {
  return invoke('transfer_list')
}

export async function transferClearCompleted(): Promise<void> {
  return invoke('transfer_clear_completed')
}

export async function remoteExec(
  sessionId: string,
  command: string,
  timeoutSecs?: number,
): Promise<{ stdout: string; exitCode: number }> {
  return invoke('remote_exec', { sessionId, command, timeoutSecs: timeoutSecs ?? null })
}

export async function listLocalKeys(): Promise<any[]> {
  return invoke('list_local_keys')
}

export async function listRemoteKeys(sessionId: string): Promise<any[]> {
  return invoke('list_remote_keys', { sessionId })
}

export async function generateKeyPair(
  algorithm: string,
  path: string,
  passphrase?: string,
): Promise<any> {
  return invoke('generate_key_pair', { algorithm, path, passphrase: passphrase ?? null })
}

export async function deployPublicKey(
  sessionId: string,
  publicKey: string,
): Promise<void> {
  return invoke('deploy_public_key', { sessionId, publicKey })
}

export async function readPublicKey(path: string): Promise<string> {
  return invoke<string>('read_public_key', { path })
}

export async function portForwardStart(
  sessionId: string,
  bindHost: string,
  bindPort: number,
  targetHost: string,
  targetPort: number,
  direction: 'local' | 'remote',
): Promise<string> {
  return invoke<string>('port_forward_start', {
    sessionId,
    bindHost,
    bindPort,
    targetHost,
    targetPort,
    direction,
  })
}

export async function portForwardStop(forwardId: string): Promise<void> {
  return invoke('port_forward_stop', { forwardId })
}

export async function portForwardList(): Promise<string[]> {
  return invoke('port_forward_list')
}

export async function importSshConfig(path?: string): Promise<any[]> {
  return invoke('import_ssh_config', { path: path ?? null })
}

export async function importSshConfigToFolder(path?: string): Promise<string> {
  return invoke<string>('import_ssh_config_to_folder', { path: path ?? null })
}

export async function credentialSave(
  sessionId: string,
  field: string,
  value: string,
): Promise<void> {
  return invoke('credential_save', { sessionId, field, value })
}

export async function credentialGet(
  sessionId: string,
  field: string,
): Promise<string | null> {
  return invoke<string | null>('credential_get', { sessionId, field })
}

export async function credentialDelete(
  sessionId: string,
  field: string,
): Promise<void> {
  return invoke('credential_delete', { sessionId, field })
}

export async function archiveCreate(
  sessionId: string,
  archivePath: string,
  sources: string[],
  format: string,
): Promise<void> {
  return invoke('archive_create', { sessionId, archivePath, sources, format })
}

export async function archiveExtract(
  sessionId: string,
  archivePath: string,
  targetDir: string,
): Promise<void> {
  return invoke('archive_extract', { sessionId, archivePath, targetDir })
}

export async function sftpUploadSudo(
  sessionId: string,
  remotePath: string,
  data: string,
): Promise<void> {
  return invoke('sftp_upload_sudo', { sessionId, remotePath, data })
}

export async function sftpDownloadSudo(
  sessionId: string,
  remotePath: string,
): Promise<string> {
  return invoke<string>('sftp_download_sudo', { sessionId, remotePath })
}

export async function localTerminalOpen(
  channelId: string,
  cols: number,
  rows: number,
): Promise<void> {
  return invoke('local_terminal_open', { channelId, cols, rows })
}

export async function localTerminalWrite(
  channelId: string,
  data: string,
): Promise<void> {
  return invoke('local_terminal_write', { channelId, data })
}

export async function localTerminalResize(
  channelId: string,
  cols: number,
  rows: number,
): Promise<void> {
  return invoke('local_terminal_resize', { channelId, cols, rows })
}

export async function localTerminalClose(channelId: string): Promise<void> {
  return invoke('local_terminal_close', { channelId })
}

export async function detectEditors(): Promise<any[]> {
  return invoke('detect_editors')
}

export async function openInEditor(filePath: string): Promise<void> {
  return invoke('open_in_editor', { filePath })
}

export async function pluginList(): Promise<any[]> {
  return invoke('plugin_list')
}

export async function pluginDiscover(): Promise<any[]> {
  return invoke('plugin_discover')
}

export async function pluginSetEnabled(pluginId: string, enabled: boolean): Promise<void> {
  return invoke('plugin_set_enabled', { pluginId, enabled })
}

export async function pluginRemove(pluginId: string): Promise<void> {
  return invoke('plugin_remove', { pluginId })
}

export async function pluginGetSetting(pluginId: string, key: string): Promise<string | null> {
  return invoke<string | null>('plugin_get_setting', { pluginId, key })
}

export async function pluginSetSetting(pluginId: string, key: string, value: string): Promise<void> {
  return invoke('plugin_set_setting', { pluginId, key, value })
}

export async function pluginGetAllSettings(pluginId: string): Promise<Record<string, string>> {
  return invoke('plugin_get_all_settings', { pluginId })
}

export async function pluginFireEvent(
  pluginId: string,
  eventType: string,
  payload: any,
): Promise<void> {
  return invoke('plugin_fire_event', { pluginId, eventType, payload })
}

export async function pluginShowNotification(
  pluginId: string,
  title: string,
  body: string,
): Promise<void> {
  return invoke('plugin_show_notification', { pluginId, title, body })
}
