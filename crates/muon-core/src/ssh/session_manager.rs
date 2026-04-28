use std::collections::HashMap;
use std::sync::Arc;

use super::auth::AuthMethod;
use super::channel::ShellChannel;
use super::connection::{
    ClientHandler, ConnectionOptions, HostKeyCheckResult, RemoteForwardMap, SshConnection, SshError,
};
use crate::session::info::SessionInfo;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectResult {
    pub session_id: String,
    pub host_key_status: String,
    pub host_key_fingerprint: Option<String>,
}

struct PendingHostKey {
    host: String,
    port: u16,
    key_bytes: Vec<u8>,
    key_type: String,
}

struct ActiveSession {
    connection: SshConnection,
    shell_channels: HashMap<String, ShellChannel>,
    read_loop_handles: HashMap<String, tokio::task::JoinHandle<()>>,
    x11_display: Option<Arc<super::x11::X11Display>>,
    pending_host_key: Option<PendingHostKey>,
}

pub struct SessionManager {
    sessions: HashMap<String, ActiveSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub async fn connect(
        &mut self,
        session_info: SessionInfo,
        auth_method: AuthMethod,
        enable_compression: bool,
        jump_credentials: &HashMap<String, String>,
    ) -> Result<ConnectResult, SshError> {
        let options = ConnectionOptions {
            keep_alive_interval_secs: Some(60),
            keep_alive_max_count: 3,
            enable_compression,
            connection_timeout_secs: 30,
        };
        self.connect_with_options(session_info, auth_method, options, jump_credentials)
            .await
    }

    pub async fn connect_with_options(
        &mut self,
        session_info: SessionInfo,
        auth_method: AuthMethod,
        options: ConnectionOptions,
        jump_credentials: &HashMap<String, String>,
    ) -> Result<ConnectResult, SshError> {
        let id = session_info.id.clone();
        let remote_forwards: RemoteForwardMap = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

        let x11_display = if session_info.x11_forwarding {
            super::x11::X11Display::from_env().map(Arc::new)
        } else {
            None
        };

        let connection = if !session_info.jump_hosts.is_empty() {
            let jump_hosts: Vec<super::jump_host::JumpHost> = session_info
                .jump_hosts
                .iter()
                .filter_map(|jh| {
                    serde_json::from_str(jh)
                        .map_err(|e| {
                            tracing::warn!(jump_host = jh, error = %e, "Malformed jump host JSON skipped");
                            e
                        })
                        .ok()
                })
                .collect();

            if jump_hosts.is_empty() {
                SshConnection::connect_with_options(
                    session_info,
                    auth_method,
                    options,
                    remote_forwards.clone(),
                )
                .await?
            } else {
                let handle = super::jump_host::JumpHostTunnel::connect_via_jumps(
                    &session_info,
                    &auth_method,
                    &jump_hosts,
                    jump_credentials,
                )
                .await?;

                let host_key_check =
                    Arc::new(tokio::sync::Mutex::new(HostKeyCheckResult::default()));
                let mut conn = SshConnection {
                    handle: None,
                    session_info: crate::session::info::SessionInfo::default(),
                    connected: false,
                    remote_forwards: remote_forwards.clone(),
                    x11_display: x11_display.clone(),
                    host_key_check: host_key_check.clone(),
                };
                conn.handle = Some(Arc::new(handle));
                conn.session_info = session_info;
                conn.connected = true;
                conn
            }
        } else {
            SshConnection::connect_with_options(
                session_info,
                auth_method,
                options,
                remote_forwards.clone(),
            )
            .await?
        };

        let check_result = {
            let guard = connection.host_key_check.lock().await;
            guard.clone()
        };

        let pending_host_key = if check_result.status == "Unknown" {
            check_result.pending_key_bytes.as_ref().and_then(|kb| {
                check_result
                    .pending_key_type
                    .as_ref()
                    .map(|kt| PendingHostKey {
                        host: connection.session_info.host.clone(),
                        port: connection.session_info.port,
                        key_bytes: kb.clone(),
                        key_type: kt.clone(),
                    })
            })
        } else {
            None
        };

        let result = ConnectResult {
            session_id: id.clone(),
            host_key_status: check_result.status.clone(),
            host_key_fingerprint: check_result.fingerprint.clone(),
        };

        tracing::info!(
            session_id = %id,
            host = %connection.session_info.host,
            host_key_status = %check_result.status,
            "SSH session connected"
        );

        self.sessions.insert(
            id,
            ActiveSession {
                connection,
                shell_channels: HashMap::new(),
                read_loop_handles: HashMap::new(),
                x11_display,
                pending_host_key,
            },
        );
        Ok(result)
    }

    pub async fn accept_host_key(&mut self, session_id: &str) -> Result<(), SshError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(SshError::NotConnected)?;
        if let Some(pending) = session.pending_host_key.take() {
            super::host_keys::add_host_key_raw(
                &pending.host,
                pending.port,
                &pending.key_bytes,
                &pending.key_type,
            )
            .map_err(|e| SshError::HostKeyError(e.to_string()))?;
            tracing::info!(host = %pending.host, "Host key accepted and saved");
        }
        Ok(())
    }

    pub async fn disconnect(&mut self, session_id: &str) -> Result<(), SshError> {
        if let Some(mut session) = self.sessions.remove(session_id) {
            for (_, handle) in session.read_loop_handles.drain() {
                handle.abort();
            }
            for (_, channel) in session.shell_channels.drain() {
                let _ = channel.close().await;
            }
            session.connection.disconnect().await?;
        }
        tracing::info!(session_id = %session_id, "SSH session disconnected");
        Ok(())
    }

    pub async fn open_shell(
        &mut self,
        session_id: &str,
        channel_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), SshError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SshError::NotConnected)?;
        let handle = session.connection.handle().ok_or(SshError::NotConnected)?;

        let channel = if let Some(display) = &session.x11_display {
            ShellChannel::open_with_x11(handle, cols, rows, display).await?
        } else {
            ShellChannel::open(handle, cols, rows).await?
        };

        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(SshError::NotConnected)?;
        session
            .shell_channels
            .insert(channel_id.to_string(), channel);
        tracing::debug!(session_id, channel_id, cols, rows, "Shell channel opened");
        Ok(())
    }

    pub fn spawn_shell_read_loop<F>(
        &mut self,
        session_id: &str,
        channel_id: &str,
        on_data: F,
    ) -> Result<(), SshError>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SshError::NotConnected)?;
        let channel = session
            .shell_channels
            .get(channel_id)
            .ok_or_else(|| SshError::ChannelError("Channel not found".to_string()))?;
        let handle = channel.spawn_read_loop(on_data);
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(SshError::NotConnected)?;
        session
            .read_loop_handles
            .insert(channel_id.to_string(), handle);
        Ok(())
    }

    pub async fn shell_write(
        &mut self,
        session_id: &str,
        channel_id: &str,
        data: &[u8],
    ) -> Result<(), SshError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SshError::NotConnected)?;
        let channel = session
            .shell_channels
            .get(channel_id)
            .ok_or_else(|| SshError::ChannelError("Channel not found".to_string()))?;
        channel.write(data).await
    }

    pub async fn shell_resize(
        &self,
        session_id: &str,
        channel_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), SshError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SshError::NotConnected)?;
        let channel = session
            .shell_channels
            .get(channel_id)
            .ok_or_else(|| SshError::ChannelError("Channel not found".to_string()))?;
        channel.resize(cols, rows).await
    }

    pub async fn close_shell(
        &mut self,
        session_id: &str,
        channel_id: &str,
    ) -> Result<(), SshError> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if let Some(handle) = session.read_loop_handles.remove(channel_id) {
                handle.abort();
            }
            if let Some(channel) = session.shell_channels.remove(channel_id) {
                channel.close().await?;
                tracing::debug!(session_id, channel_id, "Shell channel closed");
            }
        }
        Ok(())
    }

    pub async fn open_sftp_channel(
        &self,
        session_id: &str,
    ) -> Result<russh::Channel<russh::client::Msg>, SshError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SshError::NotConnected)?;
        let handle = session.connection.handle().ok_or(SshError::NotConnected)?;
        handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(format!("Failed to open SFTP channel: {}", e)))
    }

    pub fn is_connected(&self, session_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .is_some_and(|s| s.connection.is_connected())
    }

    pub fn get_handle(
        &self,
        session_id: &str,
    ) -> Option<Arc<russh::client::Handle<ClientHandler>>> {
        self.sessions.get(session_id)?.connection.handle().cloned()
    }

    pub fn get_remote_forward_map(&self, session_id: &str) -> Option<RemoteForwardMap> {
        self.sessions
            .get(session_id)
            .map(|s| s.connection.remote_forwards.clone())
    }

    pub fn connected_session_ids(&self) -> Vec<String> {
        self.sessions
            .iter()
            .filter(|(_, s)| s.connection.is_connected())
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_new() {
        let mgr = SessionManager::new();
        assert!(!mgr.is_connected("nonexistent"));
    }

    #[test]
    fn test_session_manager_default() {
        let mgr = SessionManager::default();
        assert!(!mgr.is_connected("nonexistent"));
    }

    #[test]
    fn test_is_connected_empty() {
        let mgr = SessionManager::new();
        assert!(!mgr.is_connected("any-id"));
    }

    #[test]
    fn test_get_handle_nonexistent() {
        let mgr = SessionManager::new();
        assert!(mgr.get_handle("nonexistent").is_none());
    }

    #[test]
    fn test_get_remote_forward_map_nonexistent() {
        let mgr = SessionManager::new();
        assert!(mgr.get_remote_forward_map("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_disconnect_nonexistent() {
        let mut mgr = SessionManager::new();
        let result = mgr.disconnect("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_open_shell_nonexistent() {
        let mut mgr = SessionManager::new();
        let result = mgr.open_shell("nonexistent", "ch1", 80, 24).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::NotConnected));
    }

    #[tokio::test]
    async fn test_shell_write_nonexistent() {
        let mut mgr = SessionManager::new();
        let result = mgr.shell_write("nonexistent", "ch1", b"test").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::NotConnected));
    }

    #[tokio::test]
    async fn test_shell_resize_nonexistent() {
        let mgr = SessionManager::new();
        let result = mgr.shell_resize("nonexistent", "ch1", 120, 40).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::NotConnected));
    }

    #[tokio::test]
    async fn test_close_shell_nonexistent() {
        let mut mgr = SessionManager::new();
        let result = mgr.close_shell("nonexistent", "ch1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_open_sftp_channel_nonexistent() {
        let mgr = SessionManager::new();
        let result = mgr.open_sftp_channel("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::NotConnected));
    }

    #[tokio::test]
    async fn test_spawn_shell_read_loop_nonexistent() {
        let mut mgr = SessionManager::new();
        let result = mgr.spawn_shell_read_loop("nonexistent", "ch1", |_| {});
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::NotConnected));
    }

    #[tokio::test]
    async fn test_accept_host_key_nonexistent() {
        let mut mgr = SessionManager::new();
        let result = mgr.accept_host_key("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshError::NotConnected));
    }

    #[test]
    fn test_connect_result_serialization() {
        let result = ConnectResult {
            session_id: "test-123".to_string(),
            host_key_status: "Trusted".to_string(),
            host_key_fingerprint: Some("SHA256:abc123".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test-123"));
        assert!(json.contains("Trusted"));
        assert!(json.contains("SHA256:abc123"));
    }

    #[test]
    fn test_connect_result_debug() {
        let result = ConnectResult {
            session_id: "s1".to_string(),
            host_key_status: "Unknown".to_string(),
            host_key_fingerprint: None,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("s1"));
    }

    #[test]
    fn test_connect_result_clone() {
        let result = ConnectResult {
            session_id: "s1".to_string(),
            host_key_status: "Trusted".to_string(),
            host_key_fingerprint: Some("fp".to_string()),
        };
        let cloned = result.clone();
        assert_eq!(cloned.session_id, "s1");
        assert_eq!(cloned.host_key_status, "Trusted");
        assert_eq!(cloned.host_key_fingerprint, Some("fp".to_string()));
    }
}
