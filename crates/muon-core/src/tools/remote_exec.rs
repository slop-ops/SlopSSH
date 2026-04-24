use russh::ChannelMsg;
use tokio::time::{Duration, timeout};

use crate::ssh::connection::{ClientHandler, SshError};

pub struct RemoteExecutor;

impl RemoteExecutor {
    pub async fn execute(
        handle: &russh::client::Handle<ClientHandler>,
        command: &str,
        timeout_secs: u64,
    ) -> Result<CommandResult, SshError> {
        let mut channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        channel
            .request_shell(true)
            .await
            .map_err(|e| SshError::ChannelError(format!("Shell request failed: {}", e)))?;

        channel
            .data(format!("{}\nexit $?\n", command).as_bytes())
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        let mut stdout = Vec::new();
        let mut exit_code: i32 = -1;

        let deadline = Duration::from_secs(timeout_secs);

        loop {
            let result = timeout(deadline, channel.wait()).await;
            match result {
                Ok(Some(ChannelMsg::Data { data })) => {
                    stdout.extend_from_slice(&data);
                }
                Ok(Some(ChannelMsg::ExitStatus { exit_status })) => {
                    exit_code = exit_status as i32;
                }
                Ok(Some(ChannelMsg::Eof)) | Ok(None) => {
                    break;
                }
                Ok(Some(_)) => {}
                Err(_) => {
                    return Err(SshError::Timeout);
                }
            }
        }

        Ok(CommandResult { stdout, exit_code })
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub stdout: Vec<u8>,
    pub exit_code: i32,
}

impl CommandResult {
    pub fn stdout_string(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }
}
