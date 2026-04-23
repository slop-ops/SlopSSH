use std::io::{BufRead, Write};
use std::path::PathBuf;

use base64::Engine;
use russh::keys::ssh_key;

#[derive(Debug, Clone, PartialEq)]
pub enum HostKeyStatus {
    Trusted,
    Unknown,
    Changed,
}

#[derive(Debug, Clone)]
struct KnownHostEntry {
    host_pattern: String,
    key_type: String,
    key_data: Vec<u8>,
}

pub struct HostKeyVerifier {
    entries: Vec<KnownHostEntry>,
    known_hosts_path: PathBuf,
}

impl HostKeyVerifier {
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::known_hosts_path()?;
        let entries = if path.exists() {
            Self::parse_file(&path)?
        } else {
            Vec::new()
        };
        Ok(Self {
            entries,
            known_hosts_path: path,
        })
    }

    pub fn verify(&self, host: &str, port: u16, public_key: &ssh_key::PublicKey) -> HostKeyStatus {
        let key_bytes = match public_key.to_bytes() {
            Ok(b) => b,
            Err(_) => return HostKeyStatus::Unknown,
        };

        for entry in &self.entries {
            if Self::host_matches(&entry.host_pattern, host, port) {
                if entry.key_data == key_bytes.as_ref() as &[u8] {
                    return HostKeyStatus::Trusted;
                } else {
                    return HostKeyStatus::Changed;
                }
            }
        }

        HostKeyStatus::Unknown
    }

    pub fn add_host_key(
        &mut self,
        host: &str,
        port: u16,
        public_key: &ssh_key::PublicKey,
    ) -> anyhow::Result<()> {
        let key_bytes = public_key
            .to_bytes()
            .map_err(|e| anyhow::anyhow!("Failed to encode public key: {}", e))?;

        let host_pattern = if port != 22 {
            format!("[{}]:{}", host, port)
        } else {
            host.to_string()
        };

        let key_type = match public_key.algorithm() {
            ssh_key::Algorithm::Rsa { .. } => "ssh-rsa",
            ssh_key::Algorithm::Ed25519 => "ssh-ed25519",
            ssh_key::Algorithm::Ecdsa { curve } => match curve {
                ssh_key::EcdsaCurve::NistP256 => "ecdsa-sha2-nistp256",
                ssh_key::EcdsaCurve::NistP384 => "ecdsa-sha2-nistp384",
                ssh_key::EcdsaCurve::NistP521 => "ecdsa-sha2-nistp521",
            },
            ssh_key::Algorithm::SkEd25519 => "sk-ssh-ed25519@openssh.com",
            _ => "ssh-unknown",
        };

        let entry = KnownHostEntry {
            host_pattern,
            key_type: key_type.to_string(),
            key_data: key_bytes.to_vec(),
        };

        let encoded = base64::engine::general_purpose::STANDARD.encode(&key_bytes);
        let line = format!("{} {} {}\n", entry.host_pattern, entry.key_type, encoded);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.known_hosts_path)?;
        file.write_all(line.as_bytes())?;

        self.entries.push(entry);
        Ok(())
    }

    fn known_hosts_path() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let ssh_dir = home.join(".ssh");
        if !ssh_dir.exists() {
            std::fs::create_dir_all(&ssh_dir)?;
        }
        Ok(ssh_dir.join("known_hosts"))
    }

    fn parse_file(path: &PathBuf) -> anyhow::Result<Vec<KnownHostEntry>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() < 3 {
                continue;
            }

            let host_pattern = parts[0].to_string();
            let key_type = parts[1].to_string();
            let key_data = match base64::engine::general_purpose::STANDARD.decode(parts[2]) {
                Ok(data) => data,
                Err(_) => continue,
            };

            entries.push(KnownHostEntry {
                host_pattern,
                key_type,
                key_data,
            });
        }

        Ok(entries)
    }

    fn host_matches(pattern: &str, host: &str, port: u16) -> bool {
        if pattern.starts_with('@') {
            return false;
        }

        for pattern in pattern.split(',') {
            let pattern = pattern.trim();

            if pattern == host {
                return true;
            }

            if let Some(bracketed) = pattern.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                let parts: Vec<&str> = bracketed.splitn(2, "]:").collect();
                if parts.len() == 2 {
                    let pattern_host = parts[0];
                    let pattern_port: u16 = parts[1].parse().unwrap_or(22);
                    if pattern_host == host && pattern_port == port {
                        return true;
                    }
                } else if bracketed == host && port == 22 {
                    return true;
                }
            }

            if (pattern.contains('?') || pattern.contains('*'))
                && Self::wildcard_match(pattern, host)
            {
                return true;
            }
        }

        false
    }

    fn wildcard_match(pattern: &str, text: &str) -> bool {
        let p: Vec<char> = pattern.chars().collect();
        let t: Vec<char> = text.chars().collect();
        let mut dp = vec![vec![false; t.len() + 1]; p.len() + 1];
        dp[0][0] = true;

        for i in 1..=p.len() {
            if p[i - 1] == '*' {
                dp[i][0] = dp[i - 1][0];
            }
        }

        for i in 1..=p.len() {
            for j in 1..=t.len() {
                if p[i - 1] == '*' {
                    dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
                } else if p[i - 1] == '?' || p[i - 1] == t[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                }
            }
        }

        dp[p.len()][t.len()]
    }
}

pub fn verify_host_key(host: &str, port: u16, public_key: &ssh_key::PublicKey) -> HostKeyStatus {
    match HostKeyVerifier::load() {
        Ok(verifier) => verifier.verify(host, port, public_key),
        Err(_) => HostKeyStatus::Unknown,
    }
}

pub fn add_host_key(host: &str, port: u16, public_key: &ssh_key::PublicKey) -> anyhow::Result<()> {
    let mut verifier = HostKeyVerifier::load()?;
    verifier.add_host_key(host, port, public_key)
}
