//! SSH session definitions: authentication and proxy types.

pub mod folder;
pub mod import;
pub mod info;
pub mod pool;
pub mod store;

use serde::{Deserialize, Serialize};

/// Supported SSH authentication methods.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// Password-based authentication.
    Password,
    /// Public-key authentication.
    PublicKey,
    /// Keyboard-interactive authentication.
    KeyboardInteractive,
    /// No authentication.
    None,
}

/// Supported proxy types for tunnelled connections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyType {
    /// No proxy.
    None,
    /// HTTP CONNECT proxy.
    Http,
    /// SOCKS5 proxy.
    Socks5,
}
