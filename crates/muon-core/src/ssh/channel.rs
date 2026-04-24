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

    pub async fn open_with_x11(
        handle: &russh::client::Handle<ClientHandler>,
        cols: u16,
        rows: u16,
        x11_display: &super::x11::X11Display,
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
            .request_x11(
                false,
                true,
                x11_display.auth_protocol(),
                String::new(),
                x11_display.screen_number,
            )
            .await
            .map_err(|e| SshError::ChannelError(format!("X11 request failed: {}", e)))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_error_variants() {
        let err = SshError::ChannelError("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = SshError::ConnectionFailed("conn failed".to_string());
        assert!(err.to_string().contains("conn failed"));

        let err = SshError::AuthFailed("auth failed".to_string());
        assert!(err.to_string().contains("auth failed"));

        let err = SshError::HostKeyError("bad key".to_string());
        assert!(err.to_string().contains("bad key"));

        let err = SshError::ProxyError("proxy fail".to_string());
        assert!(err.to_string().contains("proxy fail"));

        let err = SshError::Timeout;
        assert!(err.to_string().contains("Timeout"));

        let err = SshError::NotConnected;
        assert!(err.to_string().contains("Not connected"));

        let err = SshError::Other("misc".to_string());
        assert!(err.to_string().contains("misc"));
    }

    #[test]
    fn test_ssh_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SshError>();
    }

    #[test]
    fn test_ssh_error_debug_format() {
        let err = SshError::ChannelError("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ChannelError"));
    }
}
