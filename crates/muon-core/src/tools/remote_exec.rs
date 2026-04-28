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
            .exec(true, command)
            .await
            .map_err(|e| SshError::ChannelError(format!("Exec failed: {}", e)))?;

        let mut stdout = Vec::new();
        let mut exit_code: i32 = -1;

        let deadline = Duration::from_secs(timeout_secs);

        loop {
            let result = timeout(deadline, channel.wait()).await;
            match result {
                Ok(Some(ChannelMsg::Data { data })) => {
                    stdout.extend_from_slice(&data);
                }
                Ok(Some(ChannelMsg::ExtendedData { data, .. })) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_stdout_string_ascii() {
        let result = CommandResult {
            stdout: b"hello world".to_vec(),
            exit_code: 0,
        };
        assert_eq!(result.stdout_string(), "hello world");
    }

    #[test]
    fn test_command_result_stdout_string_empty() {
        let result = CommandResult {
            stdout: Vec::new(),
            exit_code: 0,
        };
        assert_eq!(result.stdout_string(), "");
    }

    #[test]
    fn test_command_result_stdout_string_utf8() {
        let result = CommandResult {
            stdout: "héllo wörld".as_bytes().to_vec(),
            exit_code: 0,
        };
        assert_eq!(result.stdout_string(), "héllo wörld");
    }

    #[test]
    fn test_command_result_stdout_string_invalid_utf8() {
        let result = CommandResult {
            stdout: vec![0xff, 0xfe, 0x00],
            exit_code: 1,
        };
        let s = result.stdout_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_command_result_exit_code_zero() {
        let result = CommandResult {
            stdout: Vec::new(),
            exit_code: 0,
        };
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_command_result_exit_code_nonzero() {
        let result = CommandResult {
            stdout: b"error".to_vec(),
            exit_code: 127,
        };
        assert_eq!(result.exit_code, 127);
    }

    #[test]
    fn test_command_result_debug_format() {
        let result = CommandResult {
            stdout: b"test".to_vec(),
            exit_code: 0,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("stdout"));
        assert!(debug.contains("exit_code"));
    }

    #[test]
    fn test_command_result_clone() {
        let result = CommandResult {
            stdout: b"data".to_vec(),
            exit_code: 42,
        };
        let cloned = result.clone();
        assert_eq!(cloned.stdout, result.stdout);
        assert_eq!(cloned.exit_code, result.exit_code);
    }

    #[test]
    fn test_command_result_multiline_stdout() {
        let result = CommandResult {
            stdout: b"line1\nline2\nline3".to_vec(),
            exit_code: 0,
        };
        assert_eq!(result.stdout_string(), "line1\nline2\nline3");
    }

    #[test]
    fn test_command_result_large_stdout() {
        let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let result = CommandResult {
            stdout: data.clone(),
            exit_code: 0,
        };
        assert_eq!(result.stdout.len(), 10000);
    }
}
