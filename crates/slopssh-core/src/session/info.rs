//! SSH session connection details and validation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{AuthType, ProxyType};

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_session() -> SessionInfo {
        SessionInfo {
            host: "example.com".to_string(),
            port: 22,
            username: "user".to_string(),
            auth_type: AuthType::Password,
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_valid_session() {
        assert!(valid_session().validate().is_ok());
    }

    #[test]
    fn test_validate_empty_host() {
        let mut s = valid_session();
        s.host = String::new();
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_whitespace_host() {
        let mut s = valid_session();
        s.host = "  ".to_string();
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_host_with_spaces() {
        let mut s = valid_session();
        s.host = "bad host".to_string();
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_port_zero() {
        let mut s = valid_session();
        s.port = 0;
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_empty_username() {
        let mut s = valid_session();
        s.username = String::new();
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_pubkey_no_path() {
        let mut s = valid_session();
        s.auth_type = AuthType::PublicKey;
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_pubkey_with_path() {
        let mut s = valid_session();
        s.auth_type = AuthType::PublicKey;
        s.private_key_path = Some(PathBuf::from("/home/user/.ssh/id_rsa"));
        assert!(s.validate().is_ok());
    }

    #[test]
    fn test_validate_proxy_no_host() {
        let mut s = valid_session();
        s.proxy_type = ProxyType::Http;
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_proxy_port_zero() {
        let mut s = valid_session();
        s.proxy_type = ProxyType::Socks5;
        s.proxy_host = Some("proxy.example.com".to_string());
        s.proxy_port = Some(0);
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_proxy_valid() {
        let mut s = valid_session();
        s.proxy_type = ProxyType::Http;
        s.proxy_host = Some("proxy.example.com".to_string());
        s.proxy_port = Some(8080);
        assert!(s.validate().is_ok());
    }

    #[test]
    fn test_validate_ip_address_host() {
        let mut s = valid_session();
        s.host = "192.168.1.1".to_string();
        assert!(s.validate().is_ok());
    }

    #[test]
    fn test_validate_ipv6_host() {
        let mut s = valid_session();
        s.host = "::1".to_string();
        assert!(s.validate().is_ok());
    }

    #[test]
    fn test_validate_host_too_long() {
        let mut s = valid_session();
        s.host = "a".repeat(254);
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_validate_keyboard_interactive() {
        let mut s = valid_session();
        s.auth_type = AuthType::KeyboardInteractive;
        assert!(s.validate().is_ok());
    }

    #[test]
    fn test_validate_none_auth() {
        let mut s = valid_session();
        s.auth_type = AuthType::None;
        assert!(s.validate().is_ok());
    }
}

/// Complete connection parameters for a single SSH session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionInfo {
    /// Unique session identifier (UUID).
    pub id: String,
    /// User-visible display name.
    pub name: String,
    /// Remote hostname or IP address.
    pub host: String,
    /// Remote TCP port.
    pub port: u16,
    /// SSH login username.
    pub username: String,
    /// Authentication method to use.
    pub auth_type: AuthType,
    /// Backend key for the stored password credential.
    pub password_key: Option<String>,
    /// Path to the SSH private key file (required when `auth_type` is `PublicKey`).
    pub private_key_path: Option<PathBuf>,
    /// Backend key for the stored private-key passphrase credential.
    pub passphrase_key: Option<String>,
    /// Proxy protocol to tunnel through.
    pub proxy_type: ProxyType,
    /// Proxy hostname.
    pub proxy_host: Option<String>,
    /// Proxy port.
    pub proxy_port: Option<u16>,
    /// Proxy authentication username.
    pub proxy_user: Option<String>,
    /// Backend key for the stored proxy password credential.
    pub proxy_password_key: Option<String>,
    /// Ordered list of jump-host session IDs for cascaded connections.
    pub jump_hosts: Vec<String>,
    /// Whether X11 forwarding is requested.
    pub x11_forwarding: bool,
    /// Optional command to execute instead of an interactive shell.
    pub remote_command: Option<String>,
    /// Working directory to open after connecting.
    pub start_directory: Option<String>,
    /// Character encoding for the session (e.g. `"utf-8"`).
    pub encoding: String,
    /// ID of the parent folder in the session tree.
    pub folder_id: Option<String>,
    /// ISO-8601 timestamp of the last successful connection.
    pub last_connected: Option<String>,
}

impl SessionInfo {
    /// Validates all fields and returns an error message describing the first
    /// problem found.
    pub fn validate(&self) -> Result<(), String> {
        if self.host.trim().is_empty() {
            return Err("Host cannot be empty".to_string());
        }
        if self.host.contains(' ') || self.host.contains('\0') || self.host.len() > 253 {
            return Err(format!("Invalid host format: '{}'", self.host));
        }
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.auth_type == AuthType::PublicKey && self.private_key_path.is_none() {
            return Err("Private key path required for public key authentication".to_string());
        }
        if self.proxy_type != ProxyType::None {
            match &self.proxy_host {
                Some(h) if !h.trim().is_empty() => {}
                _ => return Err("Proxy host required when proxy is enabled".to_string()),
            }
            if self.proxy_port.unwrap_or(0) == 0 {
                return Err("Proxy port must be > 0 when proxy is enabled".to_string());
            }
        }
        Ok(())
    }
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            host: String::new(),
            port: 22,
            username: String::new(),
            auth_type: AuthType::Password,
            password_key: None,
            private_key_path: None,
            passphrase_key: None,
            proxy_type: ProxyType::None,
            proxy_host: None,
            proxy_port: None,
            proxy_user: None,
            proxy_password_key: None,
            jump_hosts: Vec::new(),
            x11_forwarding: false,
            remote_command: None,
            start_directory: None,
            encoding: "utf-8".to_string(),
            folder_id: None,
            last_connected: None,
        }
    }
}
