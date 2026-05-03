//! SlopSSH core library — SSH session management, file transfer, and
//! application configuration built on top of `russh`.

pub mod config;
pub mod credentials;
pub mod file_transfer;
pub mod filesystem;
pub mod local_terminal;
pub mod logging;
pub mod plugin;
pub mod scripts;
pub mod session;
pub mod snippets;
pub mod ssh;
pub mod tab_state;
pub mod tools;
pub mod updater;
pub mod utils;

/// Returns the crate version as determined at compile time by `Cargo.toml`.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
