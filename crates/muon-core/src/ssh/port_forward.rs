//! SSH local and remote port forwarding.

use std::sync::Arc;

use russh::ChannelMsg;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::connection::{ClientHandler, RemoteForwardMap, SshError};

/// Direction of a port forwarding rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForwardDirection {
    /// Local port forward (`-L`): local listener → remote target.
    Local,
    /// Remote port forward (`-R`): remote listener → local target.
    Remote,
}

/// A single port forwarding rule with bind/target addresses and direction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardRule {
    /// Unique identifier for this rule.
    pub id: String,
    /// Address to bind the listener on.
    pub bind_host: String,
    /// Port to bind the listener on.
    pub bind_port: u16,
    /// Target host to forward traffic to.
    pub target_host: String,
    /// Target port to forward traffic to.
    pub target_port: u16,
    /// Whether this is a local or remote forward.
    pub direction: ForwardDirection,
}

impl PortForwardRule {
    /// Validates the rule fields, returning an error message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.bind_host.trim().is_empty() {
            return Err("Bind host cannot be empty".to_string());
        }
        if self.bind_port == 0 {
            return Err("Bind port cannot be 0".to_string());
        }
        if self.target_host.trim().is_empty() {
            return Err("Target host cannot be empty".to_string());
        }
        if self.target_port == 0 {
            return Err("Target port cannot be 0".to_string());
        }
        Ok(())
    }

    /// Creates a new local port forward rule with a generated ID.
    pub fn new_local(bind_host: &str, bind_port: u16, target_host: &str, target_port: u16) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            bind_host: bind_host.to_string(),
            bind_port,
            target_host: target_host.to_string(),
            target_port,
            direction: ForwardDirection::Local,
        }
    }

    /// Creates a new remote port forward rule with a generated ID.
    pub fn new_remote(
        bind_host: &str,
        bind_port: u16,
        target_host: &str,
        target_port: u16,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            bind_host: bind_host.to_string(),
            bind_port,
            target_host: target_host.to_string(),
            target_port,
            direction: ForwardDirection::Remote,
        }
    }
}

/// Internal bookkeeping for an active forward and its associated task.
struct ForwardEntry {
    task: tokio::task::JoinHandle<()>,
    direction: ForwardDirection,
    bind_host: String,
    bind_port: u32,
    forward_map: Option<RemoteForwardMap>,
    ssh_handle: Option<Arc<russh::client::Handle<ClientHandler>>>,
}

/// Manages active local and remote port forwarding tunnels.
pub struct PortForwardManager {
    entries: Vec<(String, ForwardEntry)>,
}

impl PortForwardManager {
    /// Creates an empty manager with no active forwards.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Starts a local (`-L` style) port forward. Returns the forward ID.
    pub fn start_local(
        &mut self,
        handle: Arc<russh::client::Handle<ClientHandler>>,
        rule: PortForwardRule,
    ) -> Result<String, SshError> {
        let forward_id = rule.id.clone();
        let target_host = rule.target_host.clone();
        let target_port = rule.target_port;
        let bind_addr = format!("{}:{}", rule.bind_host, rule.bind_port);

        let task = tokio::spawn(async move {
            let listener = match TcpListener::bind(&bind_addr).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::warn!(bind_addr, error = %e, "Failed to bind local port forward");
                    return;
                }
            };

            tracing::info!(bind_addr, "Local port forward listening");

            while let Ok((tcp_stream, _addr)) = listener.accept().await {
                let h = handle.clone();
                let th = target_host.clone();
                tokio::spawn(async move {
                    if let Ok(mut channel) = h
                        .channel_open_direct_tcpip(&th, target_port as u32, "127.0.0.1", 0)
                        .await
                    {
                        let _ = forward_local_connection(&mut channel, tcp_stream).await;
                    }
                });
            }
        });

        self.entries.push((
            forward_id.clone(),
            ForwardEntry {
                task,
                direction: ForwardDirection::Local,
                bind_host: rule.bind_host.clone(),
                bind_port: rule.bind_port as u32,
                forward_map: None,
                ssh_handle: None,
            },
        ));
        Ok(forward_id)
    }

    /// Starts a remote (`-R` style) port forward. Returns the forward ID.
    pub async fn start_remote(
        &mut self,
        handle: Arc<russh::client::Handle<ClientHandler>>,
        rule: PortForwardRule,
        forward_map: RemoteForwardMap,
    ) -> Result<String, SshError> {
        let forward_id = rule.id.clone();
        let bind_host = rule.bind_host.clone();
        let bind_port = rule.bind_port as u32;
        let target_host = rule.target_host.clone();
        let target_port = rule.target_port;

        {
            let mut map = forward_map.lock().await;
            map.insert((bind_host.clone(), bind_port), (target_host, target_port));
        }

        let actual_port = handle
            .tcpip_forward(&bind_host, bind_port)
            .await
            .map_err(|e| SshError::ChannelError(format!("tcpip_forward failed: {}", e)))?;

        let idle_task = tokio::spawn(async {
            let _ = tokio::signal::ctrl_c().await;
        });

        let handle_clone = handle.clone();
        self.entries.push((
            forward_id.clone(),
            ForwardEntry {
                task: idle_task,
                direction: ForwardDirection::Remote,
                bind_host: bind_host.clone(),
                bind_port: actual_port,
                forward_map: Some(forward_map),
                ssh_handle: Some(handle_clone),
            },
        ));
        Ok(forward_id)
    }

    /// Stops a running forward by ID, cleaning up listeners and remote registrations.
    pub async fn stop(&mut self, forward_id: &str) -> Result<(), SshError> {
        if let Some(pos) = self.entries.iter().position(|(id, _)| id == forward_id) {
            let (_, entry) = self.entries.remove(pos);
            entry.task.abort();

            if entry.direction == ForwardDirection::Remote {
                if let Some(map) = &entry.forward_map {
                    let mut map = map.lock().await;
                    map.remove(&(entry.bind_host.clone(), entry.bind_port));
                }
                if let Some(handle) = &entry.ssh_handle {
                    let _ = handle
                        .cancel_tcpip_forward(&entry.bind_host, entry.bind_port)
                        .await;
                }
            }

            Ok(())
        } else {
            Err(SshError::ChannelError(format!(
                "Forward {} not found",
                forward_id
            )))
        }
    }

    /// Stops all active forwards.
    pub async fn stop_all(&mut self) {
        for (_, entry) in self.entries.drain(..) {
            entry.task.abort();
            if entry.direction == ForwardDirection::Remote {
                if let Some(map) = &entry.forward_map {
                    let mut map = map.lock().await;
                    map.remove(&(entry.bind_host.clone(), entry.bind_port));
                }
                if let Some(handle) = &entry.ssh_handle {
                    let _ = handle
                        .cancel_tcpip_forward(&entry.bind_host, entry.bind_port)
                        .await;
                }
            }
        }
    }

    /// Returns the IDs of all currently active forwards.
    pub fn list_active(&self) -> Vec<&str> {
        self.entries.iter().map(|(id, _)| id.as_str()).collect()
    }
}

impl Default for PortForwardManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Forwards data between a local TCP stream and an SSH channel.
async fn forward_local_connection(
    channel: &mut russh::Channel<russh::client::Msg>,
    mut tcp_stream: TcpStream,
) -> Result<(), SshError> {
    let (mut tcp_read, mut tcp_write) = tcp_stream.split();
    let mut tcp_buf = vec![0u8; 8192];

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
            tcp_result = tcp_read.read(&mut tcp_buf) => {
                match tcp_result {
                    Ok(0) => {
                        let _ = channel.eof().await;
                        break;
                    }
                    Ok(n) => {
                        if channel.data(&tcp_buf[..n]).await.is_err() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_direction_equality() {
        assert_eq!(ForwardDirection::Local, ForwardDirection::Local);
        assert_eq!(ForwardDirection::Remote, ForwardDirection::Remote);
        assert_ne!(ForwardDirection::Local, ForwardDirection::Remote);
    }

    #[test]
    fn test_forward_direction_serialize() {
        let json = serde_json::to_string(&ForwardDirection::Local).unwrap();
        assert!(json.contains("local"));
        let json = serde_json::to_string(&ForwardDirection::Remote).unwrap();
        assert!(json.contains("remote"));
    }

    #[test]
    fn test_port_forward_rule_new_local() {
        let rule = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        assert!(!rule.id.is_empty());
        assert_eq!(rule.bind_host, "127.0.0.1");
        assert_eq!(rule.bind_port, 8080);
        assert_eq!(rule.target_host, "10.0.0.1");
        assert_eq!(rule.target_port, 80);
        assert_eq!(rule.direction, ForwardDirection::Local);
    }

    #[test]
    fn test_port_forward_rule_new_remote() {
        let rule = PortForwardRule::new_remote("0.0.0.0", 9090, "localhost", 3000);
        assert!(!rule.id.is_empty());
        assert_eq!(rule.bind_host, "0.0.0.0");
        assert_eq!(rule.bind_port, 9090);
        assert_eq!(rule.target_host, "localhost");
        assert_eq!(rule.target_port, 3000);
        assert_eq!(rule.direction, ForwardDirection::Remote);
    }

    #[test]
    fn test_port_forward_rule_unique_ids() {
        let r1 = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        let r2 = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        assert_ne!(r1.id, r2.id);
    }

    #[test]
    fn test_port_forward_rule_serialize_deserialize() {
        let rule = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        let json = serde_json::to_string(&rule).unwrap();
        let parsed: PortForwardRule = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, rule.id);
        assert_eq!(parsed.bind_host, "127.0.0.1");
        assert_eq!(parsed.bind_port, 8080);
        assert_eq!(parsed.target_host, "10.0.0.1");
        assert_eq!(parsed.target_port, 80);
        assert_eq!(parsed.direction, ForwardDirection::Local);
    }

    #[test]
    fn test_port_forward_manager_new() {
        let mgr = PortForwardManager::new();
        assert!(mgr.list_active().is_empty());
    }

    #[test]
    fn test_port_forward_manager_default() {
        let mgr = PortForwardManager::default();
        assert!(mgr.list_active().is_empty());
    }

    #[test]
    fn test_port_forward_rule_validate_valid() {
        let rule = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        assert!(rule.validate().is_ok());
    }

    #[test]
    fn test_port_forward_rule_validate_empty_bind_host() {
        let mut rule = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        rule.bind_host = String::new();
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_port_forward_rule_validate_zero_bind_port() {
        let mut rule = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        rule.bind_port = 0;
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_port_forward_rule_validate_empty_target_host() {
        let mut rule = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        rule.target_host = String::new();
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_port_forward_rule_validate_zero_target_port() {
        let mut rule = PortForwardRule::new_local("127.0.0.1", 8080, "10.0.0.1", 80);
        rule.target_port = 0;
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_port_forward_rule_validate_remote_valid() {
        let rule = PortForwardRule::new_remote("0.0.0.0", 9090, "localhost", 3000);
        assert!(rule.validate().is_ok());
    }

    #[tokio::test]
    async fn test_port_forward_manager_stop_nonexistent() {
        let mut mgr = PortForwardManager::new();
        let result = mgr.stop("nonexistent").await;
        assert!(result.is_err());
        if let Err(SshError::ChannelError(msg)) = result {
            assert!(msg.contains("nonexistent"));
        } else {
            panic!("Expected ChannelError");
        }
    }

    #[tokio::test]
    async fn test_port_forward_manager_stop_all_empty() {
        let mut mgr = PortForwardManager::new();
        mgr.stop_all().await;
        assert!(mgr.list_active().is_empty());
    }
}
