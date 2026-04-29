//! SSH config file (`~/.ssh/config`) parsing and session import.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::folder::SessionFolder;
use super::info::SessionInfo;
use crate::session::AuthType;

/// A single `Host` block parsed from an OpenSSH config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfigHost {
    /// The host alias or pattern from the `Host` directive.
    pub host_pattern: String,
    /// Resolved hostname or IP address (`HostName`).
    pub host_name: Option<String>,
    /// Port number.
    pub port: Option<u16>,
    /// Login user.
    pub user: Option<String>,
    /// Path to the identity (private key) file.
    pub identity_file: Option<String>,
    /// `ProxyCommand` value.
    pub proxy_command: Option<String>,
    /// `ProxyJump` value.
    pub proxy_jump: Option<String>,
    /// Whether agent forwarding is enabled.
    pub forward_agent: Option<bool>,
    /// Whether X11 forwarding is enabled.
    pub forward_x11: Option<bool>,
    /// Remote command to execute on connect.
    pub remote_command: Option<String>,
    /// Any unrecognized options captured as key/value pairs.
    pub extra_options: HashMap<String, String>,
}

/// Parser that converts OpenSSH config files into [`SessionInfo`] entries.
pub struct SshConfigImporter;

impl SshConfigImporter {
    /// Parses an SSH config file from the given path.
    pub fn parse_file(path: &Path) -> anyhow::Result<Vec<SshConfigHost>> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parses the default `~/.ssh/config` file if it exists.
    pub fn parse_default() -> anyhow::Result<Vec<SshConfigHost>> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let config_path = home.join(".ssh").join("config");
        if config_path.exists() {
            Self::parse_file(&config_path)
        } else {
            Ok(Vec::new())
        }
    }

    /// Parses an SSH config from a raw string.
    pub fn parse(content: &str) -> anyhow::Result<Vec<SshConfigHost>> {
        let mut hosts = Vec::new();
        let mut current: Option<SshConfigHost> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = if let Some(pos) = line.find(char::is_whitespace) {
                let (k, v) = line.split_at(pos);
                (k.trim(), v.trim())
            } else if let Some(pos) = line.find('=') {
                let (k, v) = line.split_at(pos);
                (k.trim(), v[1..].trim())
            } else {
                continue;
            };

            let key_lower = key.to_lowercase();

            match key_lower.as_str() {
                "host" => {
                    if let Some(host) = current.take() {
                        hosts.push(host);
                    }
                    current = Some(SshConfigHost {
                        host_pattern: value.to_string(),
                        host_name: None,
                        port: None,
                        user: None,
                        identity_file: None,
                        proxy_command: None,
                        proxy_jump: None,
                        forward_agent: None,
                        forward_x11: None,
                        remote_command: None,
                        extra_options: HashMap::new(),
                    });
                }
                "hostname" => {
                    if let Some(ref mut h) = current {
                        h.host_name = Some(value.to_string());
                    }
                }
                "port" => {
                    if let Some(ref mut h) = current {
                        h.port = value.parse().ok();
                    }
                }
                "user" => {
                    if let Some(ref mut h) = current {
                        h.user = Some(value.to_string());
                    }
                }
                "identityfile" => {
                    if let Some(ref mut h) = current {
                        let expanded = if let Some(rest) = value.strip_prefix("~/") {
                            if let Some(home) = dirs::home_dir() {
                                format!("{}/{}", home.display(), rest)
                            } else {
                                value.to_string()
                            }
                        } else {
                            value.to_string()
                        };
                        h.identity_file = Some(expanded);
                    }
                }
                "proxycommand" => {
                    if let Some(ref mut h) = current {
                        h.proxy_command = Some(value.to_string());
                    }
                }
                "proxyjump" => {
                    if let Some(ref mut h) = current {
                        h.proxy_jump = Some(value.to_string());
                    }
                }
                "forwardagent" => {
                    if let Some(ref mut h) = current {
                        h.forward_agent = Some(parse_bool(value));
                    }
                }
                "forwardx11" => {
                    if let Some(ref mut h) = current {
                        h.forward_x11 = Some(parse_bool(value));
                    }
                }
                "remotecommand" => {
                    if let Some(ref mut h) = current {
                        h.remote_command = Some(value.to_string());
                    }
                }
                _ => {
                    if let Some(ref mut h) = current {
                        h.extra_options.insert(key_lower, value.to_string());
                    }
                }
            }
        }

        if let Some(host) = current.take() {
            hosts.push(host);
        }

        Ok(hosts)
    }

    /// Converts a parsed host block into a [`SessionInfo`].
    ///
    /// Returns `None` for wildcard patterns or entries without a valid host.
    pub fn to_session_info(host: &SshConfigHost) -> Option<SessionInfo> {
        let host_pattern = &host.host_pattern;
        if host_pattern.contains('*') || host_pattern.contains('?') {
            return None;
        }

        let name = host.host_name.as_deref().unwrap_or(host_pattern);
        if name.is_empty() {
            return None;
        }

        let auth_type = if host.identity_file.is_some() {
            AuthType::PublicKey
        } else {
            AuthType::Password
        };

        Some(SessionInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: host_pattern.clone(),
            host: name.to_string(),
            port: host.port.unwrap_or(22),
            username: host.user.clone().unwrap_or_default(),
            auth_type,
            private_key_path: host.identity_file.clone().map(std::path::PathBuf::from),
            x11_forwarding: host.forward_x11.unwrap_or(false),
            remote_command: host.remote_command.clone(),
            ..Default::default()
        })
    }

    /// Converts a slice of parsed hosts into a [`SessionFolder`] ready for
    /// import into the session tree.
    pub fn import_to_folder(hosts: &[SshConfigHost]) -> SessionFolder {
        let mut folder = SessionFolder::new("SSH Config Import");
        for host in hosts {
            if let Some(info) = Self::to_session_info(host) {
                folder.items.push(info);
            }
        }
        folder
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "yes" | "true" | "1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let config = r#"
Host myserver
    HostName 192.168.1.100
    Port 2222
    User admin
    IdentityFile ~/.ssh/id_ed25519

Host github
    HostName github.com
    User git
    IdentityFile ~/.ssh/github_key
"#;
        let hosts = SshConfigImporter::parse(config).unwrap();
        assert_eq!(hosts.len(), 2);

        assert_eq!(hosts[0].host_pattern, "myserver");
        assert_eq!(hosts[0].host_name.as_deref(), Some("192.168.1.100"));
        assert_eq!(hosts[0].port, Some(2222));
        assert_eq!(hosts[0].user.as_deref(), Some("admin"));
        assert!(hosts[0].identity_file.is_some());

        assert_eq!(hosts[1].host_pattern, "github");
        assert_eq!(hosts[1].host_name.as_deref(), Some("github.com"));
        assert_eq!(hosts[1].port, None);
    }

    #[test]
    fn test_parse_wildcard_host_excluded() {
        let config = r#"
Host *
    ServerAliveInterval 60

Host real-server
    HostName 10.0.0.1
"#;
        let hosts = SshConfigImporter::parse(config).unwrap();
        let sessions: Vec<_> = hosts
            .iter()
            .filter_map(SshConfigImporter::to_session_info)
            .collect();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].host, "10.0.0.1");
    }

    #[test]
    fn test_import_to_folder() {
        let config = r#"
Host server1
    HostName 10.0.0.1
    User root

Host server2
    HostName 10.0.0.2
    Port 2222
    IdentityFile ~/.ssh/id_rsa
"#;
        let hosts = SshConfigImporter::parse(config).unwrap();
        let folder = SshConfigImporter::import_to_folder(&hosts);
        assert_eq!(folder.items.len(), 2);
        assert_eq!(folder.items[0].host, "10.0.0.1");
        assert_eq!(folder.items[1].port, 2222);
        assert_eq!(folder.items[1].auth_type, AuthType::PublicKey);
    }
}
