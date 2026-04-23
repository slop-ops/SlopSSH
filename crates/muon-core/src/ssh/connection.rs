use std::sync::Arc;

use russh::keys::{PrivateKey, PrivateKeyWithHashAlg, load_secret_key, ssh_key};
use russh::*;

use super::auth::AuthMethod;
use super::host_keys::HostKeyStatus;
use crate::session::info::SessionInfo;

#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Host key verification failed: {0}")]
    HostKeyError(String),
    #[error("Proxy error: {0}")]
    ProxyError(String),
    #[error("Timeout")]
    Timeout,
    #[error("Not connected")]
    NotConnected,
    #[error("{0}")]
    Other(String),
}

pub struct ClientHandler {
    pub host: String,
    pub port: u16,
    pub host_key_status: HostKeyStatus,
}

impl ClientHandler {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            host_key_status: HostKeyStatus::Unknown,
        }
    }
}

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        let status = super::host_keys::verify_host_key(&self.host, self.port, server_public_key);
        match status {
            HostKeyStatus::Trusted => {
                self.host_key_status = HostKeyStatus::Trusted;
                Ok(true)
            }
            HostKeyStatus::Changed => {
                self.host_key_status = HostKeyStatus::Changed;
                Ok(false)
            }
            HostKeyStatus::Unknown => {
                self.host_key_status = HostKeyStatus::Unknown;
                let _ = super::host_keys::add_host_key(&self.host, self.port, server_public_key);
                Ok(true)
            }
        }
    }
}

pub struct SshConnection {
    handle: Option<client::Handle<ClientHandler>>,
    session_info: SessionInfo,
    connected: bool,
}

impl SshConnection {
    pub async fn connect(
        session_info: SessionInfo,
        auth_method: AuthMethod,
    ) -> Result<Self, SshError> {
        let config = client::Config {
            ..Default::default()
        };

        let handler = ClientHandler::new(session_info.host.clone(), session_info.port);

        let mut session = client::connect(
            Arc::new(config),
            (&*session_info.host, session_info.port),
            handler,
        )
        .await
        .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;

        let auth_result = match auth_method {
            AuthMethod::Password { password } => session
                .authenticate_password(&session_info.username, &password)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?,
            AuthMethod::PublicKey { key_path, .. } => {
                let key_pair = load_key_pair(&key_path)?;
                let hash_alg = session
                    .best_supported_rsa_hash()
                    .await
                    .map_err(|e| SshError::AuthFailed(e.to_string()))?
                    .flatten();
                session
                    .authenticate_publickey(
                        &session_info.username,
                        PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
                    )
                    .await
                    .map_err(|e| SshError::AuthFailed(e.to_string()))?
            }
            AuthMethod::None => session
                .authenticate_none(&session_info.username)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?,
        };

        if !auth_result.success() {
            return Err(SshError::AuthFailed("Authentication rejected".to_string()));
        }

        Ok(Self {
            handle: Some(session),
            session_info,
            connected: true,
        })
    }

    pub fn is_connected(&self) -> bool {
        self.connected && self.handle.is_some()
    }

    pub fn handle(&self) -> Option<&client::Handle<ClientHandler>> {
        self.handle.as_ref()
    }

    pub async fn disconnect(&mut self) -> Result<(), SshError> {
        if let Some(handle) = self.handle.take() {
            handle
                .disconnect(Disconnect::ByApplication, "", "")
                .await
                .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;
        }
        self.connected = false;
        Ok(())
    }

    pub fn session_info(&self) -> &SessionInfo {
        &self.session_info
    }
}

fn load_key_pair(path: &std::path::Path) -> Result<PrivateKey, SshError> {
    let path_str = path.to_string_lossy().to_string();
    load_secret_key(&path_str, None)
        .map_err(|e| SshError::AuthFailed(format!("Failed to load key: {}", e)))
}
