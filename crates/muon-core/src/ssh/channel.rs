use std::sync::Arc;
use std::time::Duration;

use russh::ChannelMsg;
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::connection::{ClientHandler, SshError};

pub struct ShellChannel {
    channel: Arc<Mutex<russh::Channel<russh::client::Msg>>>,
}

impl ShellChannel {
    pub async fn open(
        handle: &russh::client::Handle<ClientHandler>,
        cols: u16,
        rows: u16,
    ) -> Result<Self, SshError> {
        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        channel
            .request_pty(false, "xterm-256color", cols as u32, rows as u32, 0, 0, &[])
            .await
            .map_err(|e| SshError::ChannelError(format!("PTY request failed: {}", e)))?;

        channel
            .request_shell(true)
            .await
            .map_err(|e| SshError::ChannelError(format!("Shell request failed: {}", e)))?;

        Ok(Self {
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    pub async fn write(&self, data: &[u8]) -> Result<(), SshError> {
        let ch = self.channel.lock().await;
        ch.data(data)
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))
    }

    pub async fn resize(&self, cols: u16, rows: u16) -> Result<(), SshError> {
        let ch = self.channel.lock().await;
        ch.window_change(cols as u32, rows as u32, 0, 0)
            .await
            .map_err(|e| SshError::ChannelError(format!("Resize failed: {}", e)))
    }

    pub async fn close(&self) -> Result<(), SshError> {
        let ch = self.channel.lock().await;
        ch.eof()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))
    }

    pub fn spawn_read_loop<F>(&self, on_data: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let channel = self.channel.clone();
        tokio::spawn(async move {
            loop {
                let result = {
                    let mut ch = channel.lock().await;
                    timeout(Duration::from_millis(100), ch.wait()).await
                };
                match result {
                    Ok(Some(ChannelMsg::Data { data })) => {
                        on_data(data.to_vec());
                    }
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(_) => {}
                }
            }
        })
    }
}
