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

export async function getAppVersion(): Promise<string> {
  return invoke<string>('get_app_version')
}
