import { invoke } from '@tauri-apps/api/core'

export async function greet(name: string): Promise<string> {
  return invoke<string>('greet', { name })
}

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

export async function sshConnect(sessionId: string, password?: string): Promise<string> {
  return invoke<string>('ssh_connect', { sessionId, password: password ?? null })
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
