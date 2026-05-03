//! X11 forwarding over SSH.

use std::path::PathBuf;
use std::sync::Arc;

use russh::ChannelMsg;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use super::connection::SshError;

/// Parsed X11 display information from the `$DISPLAY` environment variable.
#[derive(Debug, Clone)]
pub struct X11Display {
    /// X11 display number (e.g. `0` for `:0`).
    pub display_number: u32,
    /// X11 screen number (e.g. `0` for `:0.0`).
    pub screen_number: u32,
    /// Path to the X11 Unix domain socket.
    pub socket_path: PathBuf,
}

impl X11Display {
    /// Parses the `$DISPLAY` environment variable.
    pub fn from_env() -> Option<Self> {
        let display = std::env::var("DISPLAY").ok()?;
        Self::parse(&display)
    }

    /// Parses a `DISPLAY` string (e.g. `:0`, `:1.0`) into display info.
    pub fn parse(display: &str) -> Option<Self> {
        let display = display.strip_prefix(':').unwrap_or(display);

        let (host_screen, screen) = if let Some(pos) = display.rfind('.') {
            (&display[..pos], display[pos + 1..].parse().unwrap_or(0))
        } else {
            (display, 0)
        };

        let display_number: u32 = host_screen.parse().ok()?;

        let socket_path = PathBuf::from(format!("/tmp/.X11-unix/X{}", display_number));

        Some(Self {
            display_number,
            screen_number: screen,
            socket_path,
        })
    }

    /// Returns the X11 authentication protocol name.
    pub fn auth_protocol(&self) -> &str {
        "MIT-MAGIC-COOKIE-1"
    }

    /// Connects to the local X11 Unix domain socket.
    pub async fn connect(&self) -> Result<UnixStream, SshError> {
        UnixStream::connect(&self.socket_path).await.map_err(|e| {
            SshError::ChannelError(format!(
                "Failed to connect to X11 display at {:?}: {}",
                self.socket_path, e
            ))
        })
    }
}

/// Forwards data between an SSH X11 channel and the local X11 display.
pub struct X11Forwarder;

impl X11Forwarder {
    /// Spawns a bidirectional forwarding task between the SSH channel and the X display.
    pub fn spawn_forward(
        channel: russh::Channel<russh::client::Msg>,
        display: Arc<X11Display>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut unix_stream = match display.connect().await {
                Ok(s) => s,
                Err(_) => return,
            };

            let mut channel = channel;
            let mut buf = vec![0u8; 8192];

            loop {
                tokio::select! {
                    channel_data = channel.wait() => {
                        match channel_data {
                            Some(ChannelMsg::Data { data })
                                if unix_stream.write_all(&data).await.is_err() =>
                            {
                                break;
                            }
                            Some(ChannelMsg::Eof) | None => break,
                            _ => {}
                        }
                    }
                    unix_result = unix_stream.read(&mut buf) => {
                        match unix_result {
                            Ok(0) => {
                                let _ = channel.eof().await;
                                break;
                            }
                            Ok(n) => {
                                if channel.data(&buf[..n]).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
            }

            let _ = channel.eof().await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x11_display_parse_local() {
        let display = X11Display::parse(":0").unwrap();
        assert_eq!(display.display_number, 0);
        assert_eq!(display.screen_number, 0);
        assert_eq!(display.socket_path, PathBuf::from("/tmp/.X11-unix/X0"));
    }

    #[test]
    fn test_x11_display_parse_with_screen() {
        let display = X11Display::parse(":1.0").unwrap();
        assert_eq!(display.display_number, 1);
        assert_eq!(display.screen_number, 0);
    }

    #[test]
    fn test_x11_display_parse_high_number() {
        let display = X11Display::parse(":10.2").unwrap();
        assert_eq!(display.display_number, 10);
        assert_eq!(display.screen_number, 2);
        assert_eq!(display.socket_path, PathBuf::from("/tmp/.X11-unix/X10"));
    }

    #[test]
    fn test_x11_display_parse_invalid() {
        assert!(X11Display::parse("invalid").is_none());
        assert!(X11Display::parse("").is_none());
        assert!(X11Display::parse(":abc").is_none());
    }

    #[test]
    fn test_x11_display_auth_protocol() {
        let display = X11Display::parse(":0").unwrap();
        assert_eq!(display.auth_protocol(), "MIT-MAGIC-COOKIE-1");
    }

    #[test]
    fn test_x11_display_no_env() {
        unsafe { std::env::remove_var("DISPLAY") };
        assert!(X11Display::from_env().is_none());
    }

    #[test]
    fn test_x11_display_from_env_set() {
        unsafe { std::env::set_var("DISPLAY", ":99.0") };
        let display = X11Display::from_env();
        unsafe { std::env::remove_var("DISPLAY") };
        let display = display.unwrap();
        assert_eq!(display.display_number, 99);
        assert_eq!(display.screen_number, 0);
    }
}
