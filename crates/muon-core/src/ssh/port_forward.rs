use std::sync::Arc;

use russh::ChannelMsg;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::connection::{ClientHandler, SshError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForwardDirection {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardRule {
    pub id: String,
    pub bind_host: String,
    pub bind_port: u16,
    pub target_host: String,
    pub target_port: u16,
    pub direction: ForwardDirection,
}

impl PortForwardRule {
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

pub struct PortForwardManager {
    active: Vec<(String, tokio::task::JoinHandle<()>)>,
}

impl PortForwardManager {
    pub fn new() -> Self {
        Self { active: Vec::new() }
    }

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
                Err(_) => return,
            };

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

        self.active.push((forward_id.clone(), task));
        Ok(forward_id)
    }

    pub fn start_remote(
        &mut self,
        _handle: Arc<russh::client::Handle<ClientHandler>>,
        rule: PortForwardRule,
    ) -> Result<String, SshError> {
        let forward_id = rule.id.clone();

        let task = tokio::spawn(async {
            let _ = tokio::signal::ctrl_c().await;
        });

        self.active.push((forward_id.clone(), task));
        Ok(forward_id)
    }

    pub async fn stop(&mut self, forward_id: &str) -> Result<(), SshError> {
        if let Some(pos) = self.active.iter().position(|(id, _)| id == forward_id) {
            let (_, handle) = self.active.remove(pos);
            handle.abort();
            Ok(())
        } else {
            Err(SshError::ChannelError(format!(
                "Forward {} not found",
                forward_id
            )))
        }
    }

    pub async fn stop_all(&mut self) {
        for (_, handle) in self.active.drain(..) {
            handle.abort();
        }
    }

    pub fn list_active(&self) -> Vec<&str> {
        self.active.iter().map(|(id, _)| id.as_str()).collect()
    }
}

impl Default for PortForwardManager {
    fn default() -> Self {
        Self::new()
    }
}

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
