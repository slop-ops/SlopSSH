use std::collections::HashMap;
use std::sync::Arc;

use russh::keys::{PrivateKey, PrivateKeyWithHashAlg, load_secret_key, ssh_key};
use russh::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::auth::AuthMethod;
use super::host_keys::HostKeyStatus;
use crate::session::info::SessionInfo;

pub type RemoteForwardMap = Arc<tokio::sync::Mutex<HashMap<(String, u32), (String, u16)>>>;

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

#[derive(Debug, Clone)]
pub struct ConnectionOptions {
    pub keep_alive_interval_secs: Option<u64>,
    pub keep_alive_max_count: u32,
    pub enable_compression: bool,
    pub connection_timeout_secs: u64,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            keep_alive_interval_secs: Some(60),
            keep_alive_max_count: 3,
            enable_compression: false,
            connection_timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientHandler {
    pub host: String,
    pub port: u16,
    pub host_key_status: HostKeyStatus,
    pub remote_forwards: RemoteForwardMap,
}

impl ClientHandler {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            host_key_status: HostKeyStatus::Unknown,
            remote_forwards: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn with_remote_forwards(
        host: String,
        port: u16,
        remote_forwards: RemoteForwardMap,
    ) -> Self {
        Self {
            host,
            port,
            host_key_status: HostKeyStatus::Unknown,
            remote_forwards,
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

    async fn server_channel_open_forwarded_tcpip(
        &mut self,
        channel: Channel<client::Msg>,
        connected_address: &str,
        connected_port: u32,
        _originator_address: &str,
        _originator_port: u32,
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        let key = (connected_address.to_string(), connected_port);
        let target = {
            let map = self.remote_forwards.lock().await;
            map.get(&key).cloned()
        };

        if let Some((target_host, target_port)) = target {
            tokio::spawn(async move {
                if let Ok(mut tcp_stream) =
                    TcpStream::connect((target_host.as_str(), target_port)).await
                {
                    let _ = forward_channel_tcp(channel, &mut tcp_stream).await;
                }
            });
        }

        Ok(())
    }
}

async fn forward_channel_tcp(
    mut channel: Channel<client::Msg>,
    tcp_stream: &mut TcpStream,
) -> Result<(), SshError> {
    let (mut tcp_read, mut tcp_write) = tcp_stream.split();
    let mut buf = vec![0u8; 8192];

    loop {
        tokio::select! {
            channel_data = channel.wait() => {
                match channel_data {
                    Some(ChannelMsg::Data { data }) => {
                        if tcp_write.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                    Some(ChannelMsg::Eof) | None => break,
                    _ => {}
                }
            }
            tcp_result = tcp_read.read(&mut buf) => {
                match tcp_result {
                    Ok(0) => {
                        let _ = channel.eof().await;
                        break;
                    }
                    Ok(n) => {
                        if channel.data(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }

    Ok(())
}

pub struct SshConnection {
    pub(crate) handle: Option<Arc<client::Handle<ClientHandler>>>,
    pub(crate) session_info: SessionInfo,
    pub(crate) connected: bool,
    pub(crate) remote_forwards: RemoteForwardMap,
}

impl SshConnection {
    pub async fn connect(
        session_info: SessionInfo,
        auth_method: AuthMethod,
    ) -> Result<Self, SshError> {
        Self::connect_with_options(
            session_info,
            auth_method,
            ConnectionOptions::default(),
            Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        )
        .await
    }

    pub async fn connect_with_options(
        session_info: SessionInfo,
        auth_method: AuthMethod,
        options: ConnectionOptions,
        remote_forwards: RemoteForwardMap,
    ) -> Result<Self, SshError> {
        let preferred = if options.enable_compression {
            russh::Preferred {
                compression: std::borrow::Cow::Owned(vec![
                    russh::compression::ZLIB,
                    russh::compression::ZLIB_LEGACY,
                    russh::compression::NONE,
                ]),
                ..Default::default()
            }
        } else {
            russh::Preferred::default()
        };

        let config = client::Config {
            keepalive_interval: options
                .keep_alive_interval_secs
                .map(std::time::Duration::from_secs),
            keepalive_max: options.keep_alive_max_count as usize,
            preferred,
            ..Default::default()
        };

        let handler = ClientHandler::with_remote_forwards(
            session_info.host.clone(),
            session_info.port,
            remote_forwards.clone(),
        );

        let mut session = client::connect(
            Arc::new(config),
            (&*session_info.host, session_info.port),
            handler,
        )
        .await
        .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;

        let auth_ok = match &auth_method {
            AuthMethod::Password { password } => session
                .authenticate_password(&session_info.username, password)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?
                .success(),
            AuthMethod::PublicKey {
                key_path,
                passphrase,
            } => {
                let key_pair = if let Some(pp) = passphrase {
                    load_key_pair_with_passphrase(key_path, Some(pp.as_str()))?
                } else {
                    load_key_pair(key_path)?
                };
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
                    .success()
            }
            AuthMethod::KeyboardInteractive { responses } => {
                let result = session
                    .authenticate_keyboard_interactive_start(&session_info.username, None::<String>)
                    .await
                    .map_err(|e| SshError::AuthFailed(e.to_string()))?;

                match result {
                    client::KeyboardInteractiveAuthResponse::Success => true,
                    client::KeyboardInteractiveAuthResponse::Failure { .. } => false,
                    client::KeyboardInteractiveAuthResponse::InfoRequest { prompts, .. } => {
                        let answers: Vec<String> = prompts
                            .iter()
                            .enumerate()
                            .map(|(i, _)| responses.get(i).cloned().unwrap_or_default())
                            .collect();
                        let result = session
                            .authenticate_keyboard_interactive_respond(answers)
                            .await
                            .map_err(|e| SshError::AuthFailed(e.to_string()))?;
                        matches!(result, client::KeyboardInteractiveAuthResponse::Success)
                    }
                }
            }
            AuthMethod::None => session
                .authenticate_none(&session_info.username)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?
                .success(),
        };

        if !auth_ok {
            return Err(SshError::AuthFailed("Authentication rejected".to_string()));
        }

        Ok(Self {
            handle: Some(Arc::new(session)),
            session_info,
            connected: true,
            remote_forwards,
        })
    }

    pub async fn connect_via_proxy(
        session_info: SessionInfo,
        auth_method: AuthMethod,
        proxy: super::proxy::ProxyConfig,
        options: ConnectionOptions,
        remote_forwards: RemoteForwardMap,
    ) -> Result<Self, SshError> {
        let preferred = if options.enable_compression {
            russh::Preferred {
                compression: std::borrow::Cow::Owned(vec![
                    russh::compression::ZLIB,
                    russh::compression::ZLIB_LEGACY,
                    russh::compression::NONE,
                ]),
                ..Default::default()
            }
        } else {
            russh::Preferred::default()
        };

        let config = client::Config {
            keepalive_interval: options
                .keep_alive_interval_secs
                .map(std::time::Duration::from_secs),
            keepalive_max: options.keep_alive_max_count as usize,
            preferred,
            ..Default::default()
        };

        let handler = ClientHandler::with_remote_forwards(
            session_info.host.clone(),
            session_info.port,
            remote_forwards.clone(),
        );

        let tcp_stream =
            super::proxy::connect_via_proxy(&session_info.host, session_info.port, &proxy)
                .await
                .map_err(|e| SshError::ProxyError(e.to_string()))?;

        let mut session = client::connect_stream(Arc::new(config), tcp_stream, handler)
            .await
            .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;

        let auth_ok = match &auth_method {
            AuthMethod::Password { password } => session
                .authenticate_password(&session_info.username, password)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?
                .success(),
            AuthMethod::PublicKey {
                key_path,
                passphrase,
            } => {
                let key_pair = if let Some(pp) = passphrase {
                    load_key_pair_with_passphrase(key_path, Some(pp.as_str()))?
                } else {
                    load_key_pair(key_path)?
                };
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
                    .success()
            }
            AuthMethod::KeyboardInteractive { responses } => {
                let result = session
                    .authenticate_keyboard_interactive_start(&session_info.username, None::<String>)
                    .await
                    .map_err(|e| SshError::AuthFailed(e.to_string()))?;

                match result {
                    client::KeyboardInteractiveAuthResponse::Success => true,
                    client::KeyboardInteractiveAuthResponse::Failure { .. } => false,
                    client::KeyboardInteractiveAuthResponse::InfoRequest { prompts, .. } => {
                        let answers: Vec<String> = prompts
                            .iter()
                            .enumerate()
                            .map(|(i, _)| responses.get(i).cloned().unwrap_or_default())
                            .collect();
                        let result = session
                            .authenticate_keyboard_interactive_respond(answers)
                            .await
                            .map_err(|e| SshError::AuthFailed(e.to_string()))?;
                        matches!(result, client::KeyboardInteractiveAuthResponse::Success)
                    }
                }
            }
            AuthMethod::None => session
                .authenticate_none(&session_info.username)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?
                .success(),
        };

        if !auth_ok {
            return Err(SshError::AuthFailed("Authentication rejected".to_string()));
        }

        Ok(Self {
            handle: Some(Arc::new(session)),
            session_info,
            connected: true,
            remote_forwards,
        })
    }

    pub fn is_connected(&self) -> bool {
        self.connected && self.handle.is_some()
    }

    pub fn handle(&self) -> Option<&Arc<client::Handle<ClientHandler>>> {
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
        .map_err(|e| SshError::AuthFailed(format!("Failed to load key '{}': {}", path_str, e)))
}

fn load_key_pair_with_passphrase(
    path: &std::path::Path,
    passphrase: Option<&str>,
) -> Result<PrivateKey, SshError> {
    let path_str = path.to_string_lossy().to_string();
    load_secret_key(&path_str, passphrase)
        .map_err(|e| SshError::AuthFailed(format!("Failed to load key '{}': {}", path_str, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_options_default() {
        let opts = ConnectionOptions::default();
        assert_eq!(opts.keep_alive_interval_secs, Some(60));
        assert_eq!(opts.keep_alive_max_count, 3);
        assert!(!opts.enable_compression);
        assert_eq!(opts.connection_timeout_secs, 30);
    }

    #[test]
    fn test_connection_options_clone() {
        let opts = ConnectionOptions {
            keep_alive_interval_secs: Some(120),
            keep_alive_max_count: 5,
            enable_compression: true,
            connection_timeout_secs: 60,
        };
        let cloned = opts.clone();
        assert_eq!(cloned.keep_alive_interval_secs, Some(120));
        assert!(cloned.enable_compression);
    }

    #[test]
    fn test_ssh_error_display() {
        let err = SshError::ConnectionFailed("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = SshError::AuthFailed("bad password".to_string());
        assert!(err.to_string().contains("bad password"));

        let err = SshError::NotConnected;
        assert!(err.to_string().contains("Not connected"));
    }

    #[test]
    fn test_client_handler_new() {
        let handler = ClientHandler::new("example.com".to_string(), 22);
        assert_eq!(handler.host, "example.com");
        assert_eq!(handler.port, 22);
        assert_eq!(handler.host_key_status, HostKeyStatus::Unknown);
    }

    #[test]
    fn test_client_handler_with_remote_forwards() {
        let map: RemoteForwardMap = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        let handler = ClientHandler::with_remote_forwards("example.com".to_string(), 22, map);
        assert_eq!(handler.host, "example.com");
        assert_eq!(handler.port, 22);
    }
}
