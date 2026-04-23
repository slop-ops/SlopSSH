pub mod config;
pub mod credentials;
pub mod session;
pub mod snippets;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
