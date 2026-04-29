export type AuthType = 'password' | 'public_key' | 'keyboard_interactive' | 'none'
export type ProxyType = 'none' | 'http' | 'socks5'

export interface SessionInfo {
  id: string
  name: string
  host: string
  port: number
  username: string
  auth_type: AuthType
  password_key: string | null
  private_key_path: string | null
  passphrase_key: string | null
  proxy_type: ProxyType
  proxy_host: string | null
  proxy_port: number | null
  proxy_user: string | null
  proxy_password_key: string | null
  jump_hosts: string[]
  x11_forwarding: boolean
  remote_command: string | null
  start_directory: string | null
  encoding: string
  folder_id: string | null
  last_connected: string | null
}

export interface Settings {
  language: string
  theme: string
  font_family: string
  font_size: number
  terminal_scrollback: number
  terminal_copy_on_select: boolean
  show_hidden_files: boolean
  default_edit_command: string
  external_editor: string
  confirm_before_delete: boolean
  confirm_before_overwrite: boolean
  transfer_parallel_count: number
  connection_timeout_secs: number
  keep_alive_interval_secs: number
  enable_compression: boolean
  keyboard_shortcuts: string
  log_level: string
}

export type FileType = 'File' | 'Directory' | 'Symlink' | 'Other'

export interface FileAttributes {
  size: number
  file_type: FileType
  modified: number | null
  permissions: number | null
  uid: number | null
  gid: number | null
}

export interface DirEntry {
  name: string
  path: string
  attributes: FileAttributes
}

export interface Snippet {
  id: string
  name: string
  command: string
  description: string | null
}

export type TransferDirection = 'Upload' | 'Download'
export type TransferStatus = 'Queued' | 'InProgress' | 'Completed' | 'Failed' | 'Cancelled'
export type ConflictResolution = 'Overwrite' | 'Skip' | 'Rename' | 'Prompt'

export interface TransferRequest {
  id: string
  session_id: string
  direction: TransferDirection
  source_path: string
  dest_path: string
  file_size: number
  conflict_resolution: ConflictResolution
}

export interface TransferProgress {
  id: string
  bytes_transferred: number
  total_bytes: number
  status: TransferStatus
  error: string | null
  speed_bps: number
}

export type ForwardDirection = 'local' | 'remote'

export interface PortForwardRule {
  id: string
  bind_host: string
  bind_port: number
  target_host: string
  target_port: number
  direction: ForwardDirection
}

export type PluginCapability =
  | 'execute_command'
  | 'read_setting'
  | 'show_notification'
  | 'on_session_connect'
  | 'on_session_disconnect'
  | 'render_panel'

export type PanelContentType = 'html' | 'json' | 'markdown' | 'text'

export interface PluginManifest {
  id: string
  name: string
  version: string
  description: string | null
  author: string | null
  capabilities: PluginCapability[]
}

export interface PluginInfo extends PluginManifest {
  enabled: boolean
}

export interface PluginPanel {
  title: string
  content_type: PanelContentType
  content: string
}

export interface PluginEvent {
  event_type: string
  payload: unknown
}

export interface SshKeyInfo {
  path: string
  name: string
  key_type: string
  fingerprint: string | null
  has_public_key: boolean
}

export interface SessionFolder {
  id: string
  name: string
  folders: SessionFolder[]
  items: SessionInfo[]
}

export interface SshConfigHost {
  host_pattern: string
  host_name: string | null
  port: number | null
  user: string | null
  identity_file: string | null
  proxy_command: string | null
  proxy_jump: string | null
  forward_agent: boolean | null
  forward_x11: boolean | null
  remote_command: string | null
  extra_options: Record<string, string>
}

export interface ConnectResult {
  session_id: string
  host_key_status: string
  host_key_fingerprint: string | null
}

export interface EditorInfo {
  name: string
  command: string
  path: string | null
}

export interface RemoteExecResult {
  stdout: string
  exitCode: number
}

export interface SavedTab {
  session_id: string
  channel_id: string
  title: string
  is_local: boolean
}

export interface TabState {
  tabs: SavedTab[]
  active_tab_id: string | null
}
