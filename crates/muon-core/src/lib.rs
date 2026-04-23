pub mod config;
pub mod credentials;
pub mod filesystem;
pub mod logging;
pub mod session;
pub mod snippets;
pub mod ssh;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
