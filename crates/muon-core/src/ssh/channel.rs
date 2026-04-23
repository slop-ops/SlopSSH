use russh::ChannelMsg;

use super::connection::{ClientHandler, SshError};

pub struct ShellChannel {
    channel: russh::Channel<russh::client::Msg>,
    cols: u16,
    rows: u16,
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
            channel,
            cols,
            rows,
        })
    }

    pub async fn write(&self, data: &[u8]) -> Result<(), SshError> {
        self.channel
            .data(data)
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))
    }

    pub async fn read(&mut self) -> Option<ChannelMsg> {
        self.channel.wait().await
    }

    pub async fn resize(&self, cols: u16, rows: u16) -> Result<(), SshError> {
        self.channel
            .window_change(cols as u32, rows as u32, 0, 0)
            .await
            .map_err(|e| SshError::ChannelError(format!("Resize failed: {}", e)))?;
        Ok(())
    }

    pub async fn close(&self) -> Result<(), SshError> {
        self.channel
            .eof()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }
}
