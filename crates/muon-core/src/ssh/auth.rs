use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Password {
        password: String,
    },
    PublicKey {
        key_path: PathBuf,
        passphrase: Option<String>,
    },
    None,
}
