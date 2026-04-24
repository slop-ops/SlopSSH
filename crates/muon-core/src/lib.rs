pub mod config;
pub mod credentials;
pub mod file_transfer;
pub mod filesystem;
pub mod local_terminal;
pub mod logging;
pub mod scripts;
pub mod session;
pub mod snippets;
pub mod ssh;
pub mod tools;
pub mod updater;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
