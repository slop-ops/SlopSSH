//! SSH jump host (bastion) tunneling for multi-hop connections.

use std::collections::HashMap;
use std::sync::Arc;

use russh::keys::{PrivateKey, PrivateKeyWithHashAlg, load_secret_key, ssh_key};
use russh::*;

use super::auth::AuthMethod;
use super::connection::{ClientHandler, SshError};
use crate::session::AuthType;
use crate::session::info::SessionInfo;

/// Describes a single jump host in a connection chain.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JumpHost {
    /// Jump host hostname or IP.
    pub host: String,
    /// Jump host SSH port.
    pub port: u16,
    /// Username for the jump host.
    pub username: String,
    /// Authentication type to use for this jump host.
    pub auth_type: AuthType,
    /// Key into the credentials map for password-based auth.
    pub password_key: Option<String>,
    /// Path to the private key file for public-key auth.
    pub private_key_path: Option<std::path::PathBuf>,
}

/// russh client handler for intermediate jump host connections.
struct JumpClientHandler {
    host: String,
    port: u16,
}

impl client::Handler for JumpClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        let status = super::host_keys::verify_host_key(&self.host, self.port, server_public_key);
        match status {
            super::host_keys::HostKeyStatus::Trusted => Ok(true),
            super::host_keys::HostKeyStatus::Changed => {
                tracing::warn!(
                    host = %self.host,
                    port = self.port,
                    "Jump host key changed — possible MITM attack"
                );
                Ok(false)
            }
            super::host_keys::HostKeyStatus::Unknown => {
                tracing::info!(
                    host = %self.host,
                    port = self.port,
                    "Unknown jump host key — auto-accepting"
                );
                let _ = super::host_keys::add_host_key(&self.host, self.port, server_public_key);
                Ok(true)
            }
        }
    }
}

/// Establishes SSH connections through one or more jump hosts.
pub struct JumpHostTunnel;

impl JumpHostTunnel {
    /// Connects to the target host by tunnelling through the given jump hosts in order.
    pub async fn connect_via_jumps(
        target: &SessionInfo,
        target_auth: &AuthMethod,
        jump_hosts: &[JumpHost],
        jump_credentials: &HashMap<String, String>,
    ) -> Result<client::Handle<ClientHandler>, SshError> {
        if jump_hosts.is_empty() {
            return Err(SshError::Other("No jump hosts provided".to_string()));
        }

        let mut prev_handle: Option<client::Handle<JumpClientHandler>> = None;

        for (i, jump) in jump_hosts.iter().enumerate() {
            let (next_host, next_port) = if i + 1 < jump_hosts.len() {
                (jump_hosts[i + 1].host.clone(), jump_hosts[i + 1].port)
            } else {
                (target.host.clone(), target.port)
            };

            let handle = if let Some(ref ph) = prev_handle {
                Self::connect_through_jump(ph, jump, &next_host, next_port, jump_credentials)
                    .await?
            } else {
                Self::connect_direct(jump, jump_credentials).await?
            };

            prev_handle = Some(handle);
        }

        let final_handle = prev_handle.ok_or_else(|| {
            SshError::ConnectionFailed("Failed to establish jump chain".to_string())
        })?;

        let target_host = target.host.as_str();
        let target_port = target.port;

        let channel = final_handle
            .channel_open_direct_tcpip(target_host, target_port as u32, "127.0.0.1", 0)
            .await
            .map_err(|e| {
                SshError::ConnectionFailed(format!(
                    "Failed to open forwarding channel to target: {}",
                    e
                ))
            })?;

        let stream = channel.into_stream();

        let config = client::Config {
            ..Default::default()
        };
        let handler = ClientHandler::new(target.host.clone(), target.port);

        let mut session: client::Handle<ClientHandler> =
            client::connect_stream(Arc::new(config), stream, handler)
                .await
                .map_err(|e| {
                    SshError::ConnectionFailed(format!("Target connection failed: {}", e))
                })?;

        Self::authenticate_target(&mut session, target, target_auth).await?;

        Ok(session)
    }

    /// Connects directly to the first jump host in the chain.
    async fn connect_direct(
        jump: &JumpHost,
        jump_credentials: &HashMap<String, String>,
    ) -> Result<client::Handle<JumpClientHandler>, SshError> {
        let config = client::Config {
            ..Default::default()
        };
        let handler = JumpClientHandler {
            host: jump.host.clone(),
            port: jump.port,
        };

        let mut session = client::connect(Arc::new(config), (&*jump.host, jump.port), handler)
            .await
            .map_err(|e| SshError::ConnectionFailed(format!("Jump host connect failed: {}", e)))?;

        Self::authenticate_jump(&mut session, jump, jump_credentials).await?;

        Ok(session)
    }

    /// Connects to a jump host by opening a forwarding channel through the previous hop.
    async fn connect_through_jump(
        prev_handle: &client::Handle<JumpClientHandler>,
        jump: &JumpHost,
        next_host: &str,
        next_port: u16,
        jump_credentials: &HashMap<String, String>,
    ) -> Result<client::Handle<JumpClientHandler>, SshError> {
        let channel = prev_handle
            .channel_open_direct_tcpip(next_host, next_port as u32, "127.0.0.1", 0)
            .await
            .map_err(|e| {
                SshError::ConnectionFailed(format!(
                    "Forwarding channel to {}:{} failed: {}",
                    next_host, next_port, e
                ))
            })?;

        let stream = channel.into_stream();

        let config = client::Config {
            ..Default::default()
        };
        let handler = JumpClientHandler {
            host: jump.host.clone(),
            port: jump.port,
        };

        let mut session: client::Handle<JumpClientHandler> =
            client::connect_stream(Arc::new(config), stream, handler)
                .await
                .map_err(|e| {
                    SshError::ConnectionFailed(format!(
                        "Connect to {}:{} through jump failed: {}",
                        next_host, next_port, e
                    ))
                })?;

        Self::authenticate_jump(&mut session, jump, jump_credentials).await?;

        Ok(session)
    }

    /// Looks up a jump host password from the credentials map.
    fn resolve_password(
        jump: &JumpHost,
        jump_credentials: &HashMap<String, String>,
    ) -> Option<String> {
        jump.password_key
            .as_ref()
            .and_then(|pk| jump_credentials.get(pk).cloned())
    }

    /// Authenticates to an intermediate jump host.
    async fn authenticate_jump(
        session: &mut client::Handle<JumpClientHandler>,
        jump: &JumpHost,
        jump_credentials: &HashMap<String, String>,
    ) -> Result<(), SshError> {
        let auth_result = match jump.auth_type {
            AuthType::Password => {
                let password = Self::resolve_password(jump, jump_credentials).ok_or_else(|| {
                    SshError::AuthFailed(format!(
                        "No password configured for jump host '{}'",
                        jump.host
                    ))
                })?;
                session
                    .authenticate_password(&jump.username, &password)
                    .await
                    .map_err(|e| SshError::AuthFailed(e.to_string()))?
            }
            AuthType::PublicKey => {
                if let Some(ref key_path) = jump.private_key_path {
                    let key_pair = load_jump_key(key_path)?;
                    let hash_alg = session
                        .best_supported_rsa_hash()
                        .await
                        .map_err(|e| SshError::AuthFailed(e.to_string()))?
                        .flatten();
                    session
                        .authenticate_publickey(
                            &jump.username,
                            PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
                        )
                        .await
                        .map_err(|e| SshError::AuthFailed(e.to_string()))?
                } else {
                    return Err(SshError::AuthFailed(
                        "No private key path for jump host".to_string(),
                    ));
                }
            }
            _ => session
                .authenticate_none(&jump.username)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?,
        };

        if !auth_result.success() {
            return Err(SshError::AuthFailed(format!(
                "Jump host {} auth rejected",
                jump.host
            )));
        }

        Ok(())
    }

    /// Authenticates to the final target host through the jump chain.
    async fn authenticate_target(
        session: &mut client::Handle<ClientHandler>,
        target: &SessionInfo,
        auth: &AuthMethod,
    ) -> Result<(), SshError> {
        let auth_result = match auth {
            AuthMethod::Password { password } => session
                .authenticate_password(&target.username, password)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?,
            AuthMethod::PublicKey { key_path, .. } => {
                let key_pair = load_jump_key(key_path)?;
                let hash_alg = session
                    .best_supported_rsa_hash()
                    .await
                    .map_err(|e| SshError::AuthFailed(e.to_string()))?
                    .flatten();
                session
                    .authenticate_publickey(
                        &target.username,
                        PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
                    )
                    .await
                    .map_err(|e| SshError::AuthFailed(e.to_string()))?
            }
            AuthMethod::KeyboardInteractive { .. } => {
                return Err(SshError::AuthFailed(
                    "Keyboard-interactive auth not supported via jump hosts".to_string(),
                ));
            }
            AuthMethod::None => session
                .authenticate_none(&target.username)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?,
        };

        if !auth_result.success() {
            return Err(SshError::AuthFailed(
                "Target host authentication rejected".to_string(),
            ));
        }

        Ok(())
    }
}

/// Loads an SSH private key for jump host authentication.
fn load_jump_key(path: &std::path::Path) -> Result<PrivateKey, SshError> {
    let path_str = path.to_string_lossy().to_string();
    load_secret_key(&path_str, None)
        .map_err(|e| SshError::AuthFailed(format!("Failed to load key: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::AuthType;

    #[test]
    fn test_jump_host_serialization() {
        let jh = JumpHost {
            host: "jump.example.com".to_string(),
            port: 22,
            username: "admin".to_string(),
            auth_type: AuthType::Password,
            password_key: Some("jump-password".to_string()),
            private_key_path: None,
        };
        let json = serde_json::to_string(&jh).unwrap();
        assert!(json.contains("jump.example.com"));
        assert!(json.contains("admin"));
        let parsed: JumpHost = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.host, "jump.example.com");
        assert_eq!(parsed.port, 22);
        assert_eq!(parsed.username, "admin");
    }

    #[test]
    fn test_jump_host_with_public_key() {
        let jh = JumpHost {
            host: "bastion.io".to_string(),
            port: 2222,
            username: "deploy".to_string(),
            auth_type: AuthType::PublicKey,
            password_key: None,
            private_key_path: Some(std::path::PathBuf::from("/home/user/.ssh/id_rsa")),
        };
        let json = serde_json::to_string(&jh).unwrap();
        let parsed: JumpHost = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.auth_type, AuthType::PublicKey);
        assert_eq!(
            parsed.private_key_path,
            Some(std::path::PathBuf::from("/home/user/.ssh/id_rsa"))
        );
    }

    #[test]
    fn test_jump_host_clone() {
        let jh = JumpHost {
            host: "test.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::Password,
            password_key: Some("key".to_string()),
            private_key_path: None,
        };
        let cloned = jh.clone();
        assert_eq!(cloned.host, jh.host);
        assert_eq!(cloned.port, jh.port);
        assert_eq!(cloned.username, jh.username);
    }

    #[test]
    fn test_jump_host_debug() {
        let jh = JumpHost {
            host: "debug.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::None,
            password_key: None,
            private_key_path: None,
        };
        let debug = format!("{:?}", jh);
        assert!(debug.contains("debug.com"));
    }

    #[test]
    fn test_resolve_password_with_key() {
        let jh = JumpHost {
            host: "jump.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::Password,
            password_key: Some("cred-key-1".to_string()),
            private_key_path: None,
        };
        let mut creds = HashMap::new();
        creds.insert("cred-key-1".to_string(), "secret123".to_string());
        creds.insert("other-key".to_string(), "other".to_string());

        let result = JumpHostTunnel::resolve_password(&jh, &creds);
        assert_eq!(result, Some("secret123".to_string()));
    }

    #[test]
    fn test_resolve_password_no_key() {
        let jh = JumpHost {
            host: "jump.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::Password,
            password_key: None,
            private_key_path: None,
        };
        let creds = HashMap::new();
        let result = JumpHostTunnel::resolve_password(&jh, &creds);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_password_missing_credential() {
        let jh = JumpHost {
            host: "jump.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::Password,
            password_key: Some("nonexistent-key".to_string()),
            private_key_path: None,
        };
        let creds = HashMap::new();
        let result = JumpHostTunnel::resolve_password(&jh, &creds);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_password_empty_credentials() {
        let jh = JumpHost {
            host: "jump.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::Password,
            password_key: Some("key".to_string()),
            private_key_path: None,
        };
        let creds = HashMap::new();
        assert_eq!(JumpHostTunnel::resolve_password(&jh, &creds), None);
    }

    #[test]
    fn test_jump_host_auth_type_none() {
        let jh = JumpHost {
            host: "jump.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::None,
            password_key: None,
            private_key_path: None,
        };
        assert_eq!(jh.auth_type, AuthType::None);
        assert!(jh.password_key.is_none());
        assert!(jh.private_key_path.is_none());
    }

    #[test]
    fn test_jump_host_non_standard_port() {
        let jh = JumpHost {
            host: "jump.com".to_string(),
            port: 2222,
            username: "user".to_string(),
            auth_type: AuthType::Password,
            password_key: Some("pk".to_string()),
            private_key_path: None,
        };
        assert_eq!(jh.port, 2222);
        let json = serde_json::to_string(&jh).unwrap();
        assert!(json.contains("2222"));
    }
}
