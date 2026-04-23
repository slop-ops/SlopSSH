use std::collections::HashMap;

use super::auth::AuthMethod;
use super::channel::ShellChannel;
use super::connection::{SshConnection, SshError};
use crate::session::info::SessionInfo;

struct ActiveSession {
    connection: SshConnection,
    shell_channels: HashMap<String, ShellChannel>,
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
    ) -> Result<String, SshError> {
        let id = session_info.id.clone();
        let connection = SshConnection::connect(session_info, auth_method).await?;
        self.sessions.insert(
            id.clone(),
            ActiveSession {
                connection,
                shell_channels: HashMap::new(),
            },
        );
        Ok(id)
    }

    pub async fn disconnect(&mut self, session_id: &str) -> Result<(), SshError> {
        if let Some(mut session) = self.sessions.remove(session_id) {
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
        let channel = ShellChannel::open(handle, cols, rows).await?;
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(SshError::NotConnected)?;
        session
            .shell_channels
            .insert(channel_id.to_string(), channel);
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
            .get_mut(session_id)
            .ok_or(SshError::NotConnected)?;
        let channel = session
            .shell_channels
            .get(channel_id)
            .ok_or_else(|| SshError::ChannelError("Channel not found".to_string()))?;
        channel.write(data).await
    }

    pub async fn shell_read(
        &mut self,
        session_id: &str,
        channel_id: &str,
    ) -> Result<Option<russh::ChannelMsg>, SshError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(SshError::NotConnected)?;
        let channel = session
            .shell_channels
            .get_mut(channel_id)
            .ok_or_else(|| SshError::ChannelError("Channel not found".to_string()))?;
        Ok(channel.read().await)
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

    pub fn is_connected(&self, session_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .is_some_and(|s| s.connection.is_connected())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
