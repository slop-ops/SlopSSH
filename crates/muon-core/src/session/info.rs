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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: AuthType,
    pub password_key: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub passphrase_key: Option<String>,
    pub proxy_type: ProxyType,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
    pub proxy_user: Option<String>,
    pub proxy_password_key: Option<String>,
    pub jump_hosts: Vec<String>,
    pub x11_forwarding: bool,
    pub remote_command: Option<String>,
    pub start_directory: Option<String>,
    pub encoding: String,
    pub folder_id: Option<String>,
    pub last_connected: Option<String>,
}

impl SessionInfo {
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
