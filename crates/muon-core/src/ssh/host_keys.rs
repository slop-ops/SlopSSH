use russh::keys::ssh_key;

pub enum HostKeyStatus {
    Trusted,
    Unknown,
    Changed,
}

pub fn verify_host_key(_host: &str, _port: u16, _public_key: &ssh_key::PublicKey) -> HostKeyStatus {
    HostKeyStatus::Unknown
}

pub fn add_host_key(
    _host: &str,
    _port: u16,
    _public_key: &ssh_key::PublicKey,
) -> anyhow::Result<()> {
    Ok(())
}
