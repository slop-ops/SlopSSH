pub mod folder;
pub mod import;
pub mod info;
pub mod pool;
pub mod store;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Password,
    PublicKey,
    KeyboardInteractive,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProxyType {
    None,
    Http,
    Socks5,
}
