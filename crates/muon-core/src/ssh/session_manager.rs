use std::collections::HashMap;
use std::sync::Arc;

use super::auth::AuthMethod;
use super::channel::ShellChannel;
use super::connection::{
    ClientHandler, ConnectionOptions, RemoteForwardMap, SshConnection, SshError,
};
use crate::session::info::SessionInfo;

struct ActiveSession {
    connection: SshConnection,
    shell_channels: HashMap<String, ShellChannel>,
    read_loop_handles: HashMap<String, tokio::task::JoinHandle<()>>,
    x11_display: Option<Arc<super::x11::X11Display>>,
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
    ) -> Result<String, SshError> {
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
    ) -> Result<String, SshError> {
        let id = session_info.id.clone();
        let remote_forwards: RemoteForwardMap = Arc::new(tokio::sync::Mutex::new(HashMap::new()));

        let x11_display = if session_info.x11_forwarding {
            super::x11::X11Display::from_env().map(Arc::new)
        } else {
            None
        };

        let connection = if !session_info.jump_hosts.is_empty() {
            let jump_hosts = session_info
                .jump_hosts
                .iter()
                .filter_map(|jh| serde_json::from_str(jh).ok())
                .collect::<Vec<super::jump_host::JumpHost>>();

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

                let mut conn = SshConnection {
                    handle: None,
                    session_info: crate::session::info::SessionInfo::default(),
                    connected: false,
                    remote_forwards: remote_forwards.clone(),
                    x11_display: x11_display.clone(),
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

        self.sessions.insert(
            id.clone(),
            ActiveSession {
                connection,
                shell_channels: HashMap::new(),
                read_loop_handles: HashMap::new(),
                x11_display,
            },
        );
        Ok(id)
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
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
