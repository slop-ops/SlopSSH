import { describe, it, expect } from 'vitest'
import type {
  SessionInfo,
  Settings,
  DirEntry,
  TransferRequest,
  TransferProgress,
  PortForwardRule,
  SshKeyInfo,
  ConnectResult,
} from '$lib/types'

describe('types', () => {
  describe('SessionInfo', () => {
    it('creates a valid SessionInfo', () => {
      const session: SessionInfo = {
        id: 'test-id',
        name: 'Test Server',
        host: 'example.com',
        port: 22,
        username: 'admin',
        auth_type: 'password',
        password_key: 'cred-1',
        private_key_path: null,
        passphrase_key: null,
        proxy_type: 'none',
        proxy_host: null,
        proxy_port: null,
        proxy_user: null,
        proxy_password_key: null,
        jump_hosts: [],
        x11_forwarding: false,
        remote_command: null,
        start_directory: null,
        encoding: 'utf-8',
        folder_id: null,
        last_connected: null,
      }
      expect(session.id).toBe('test-id')
      expect(session.host).toBe('example.com')
      expect(session.auth_type).toBe('password')
    })
  })

  describe('Settings', () => {
    it('creates a valid Settings', () => {
      const settings: Settings = {
        language: 'en',
        theme: 'dark',
        font_family: 'JetBrains Mono',
        font_size: 14,
        terminal_scrollback: 10000,
        terminal_copy_on_select: true,
        show_hidden_files: false,
        default_edit_command: 'nano',
        external_editor: '',
        confirm_before_delete: true,
        confirm_before_overwrite: true,
        transfer_parallel_count: 4,
        connection_timeout_secs: 30,
        keep_alive_interval_secs: 60,
        enable_compression: false,
        keyboard_shortcuts: '',
        log_level: 'info',
      }
      expect(settings.font_size).toBe(14)
      expect(settings.theme).toBe('dark')
    })
  })

  describe('DirEntry', () => {
    it('creates a valid file entry', () => {
      const entry: DirEntry = {
        name: 'test.txt',
        path: '/home/user/test.txt',
        attributes: {
          size: 1024,
          file_type: 'File',
          modified: Date.now(),
          permissions: 644,
          uid: 1000,
          gid: 1000,
        },
      }
      expect(entry.attributes.file_type).toBe('File')
      expect(entry.attributes.size).toBe(1024)
    })

    it('creates a valid directory entry', () => {
      const entry: DirEntry = {
        name: 'docs',
        path: '/home/user/docs',
        attributes: {
          size: 4096,
          file_type: 'Directory',
          modified: null,
          permissions: 755,
          uid: null,
          gid: null,
        },
      }
      expect(entry.attributes.file_type).toBe('Directory')
    })
  })

  describe('TransferProgress', () => {
    it('creates a valid transfer progress', () => {
      const progress: TransferProgress = {
        id: 't1',
        bytes_transferred: 512,
        total_bytes: 1024,
        status: 'InProgress',
        error: null,
        speed_bps: 1234.5,
      }
      expect(progress.status).toBe('InProgress')
      expect(progress.speed_bps).toBe(1234.5)
    })

    it('handles failed transfer', () => {
      const progress: TransferProgress = {
        id: 't2',
        bytes_transferred: 0,
        total_bytes: 100,
        status: 'Failed',
        error: 'Connection reset',
        speed_bps: 0,
      }
      expect(progress.error).toBe('Connection reset')
    })
  })

  describe('PortForwardRule', () => {
    it('creates local forward rule', () => {
      const rule: PortForwardRule = {
        id: 'pf1',
        bind_host: '127.0.0.1',
        bind_port: 8080,
        target_host: '10.0.0.1',
        target_port: 80,
        direction: 'local',
      }
      expect(rule.direction).toBe('local')
    })

    it('creates remote forward rule', () => {
      const rule: PortForwardRule = {
        id: 'pf2',
        bind_host: '0.0.0.0',
        bind_port: 9090,
        target_host: 'localhost',
        target_port: 3000,
        direction: 'remote',
      }
      expect(rule.direction).toBe('remote')
    })
  })

  describe('SshKeyInfo', () => {
    it('creates key info', () => {
      const key: SshKeyInfo = {
        path: '/home/user/.ssh/id_ed25519',
        name: 'id_ed25519',
        key_type: 'OpenSSH',
        fingerprint: 'SHA256:abc',
        has_public_key: true,
      }
      expect(key.has_public_key).toBe(true)
      expect(key.fingerprint).toBe('SHA256:abc')
    })
  })

  describe('ConnectResult', () => {
    it('creates trusted result', () => {
      const result: ConnectResult = {
        session_id: 's1',
        host_key_status: 'Trusted',
        host_key_fingerprint: 'SHA256:xyz',
      }
      expect(result.host_key_status).toBe('Trusted')
    })

    it('creates unknown result', () => {
      const result: ConnectResult = {
        session_id: 's2',
        host_key_status: 'Unknown',
        host_key_fingerprint: null,
      }
      expect(result.host_key_fingerprint).toBeNull()
    })
  })
})
