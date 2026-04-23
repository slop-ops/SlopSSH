use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{AuthType, ProxyType};

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
