//! Host key verification against `~/.ssh/known_hosts`.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use base64::Engine;
use russh::keys::ssh_key;
use sha2::{Digest, Sha256};

/// Trust status of a server host key after verification.
#[derive(Debug, Clone, PartialEq)]
pub enum HostKeyStatus {
    /// Key matches a known entry.
    Trusted,
    /// No entry found for this host.
    Unknown,
    /// Key differs from the previously stored entry.
    Changed,
}

/// A single parsed entry from a `known_hosts` file.
#[derive(Debug, Clone)]
struct KnownHostEntry {
    host_pattern: String,
    key_type: String,
    key_data: Vec<u8>,
}

/// Parses and verifies host keys against the user's `known_hosts` file.
pub struct HostKeyVerifier {
    entries: Vec<KnownHostEntry>,
    known_hosts_path: PathBuf,
}

impl HostKeyVerifier {
    /// Loads the default `~/.ssh/known_hosts` file.
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

    /// Checks whether the given public key is trusted, unknown, or changed for the host.
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

    /// Appends a new host key entry to the `known_hosts` file.
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

    /// Appends a host key entry using raw bytes and type string instead of a parsed key.
    pub fn add_host_key_raw(
        &mut self,
        host: &str,
        port: u16,
        key_bytes: &[u8],
        key_type: &str,
    ) -> anyhow::Result<()> {
        let host_pattern = if port != 22 {
            format!("[{}]:{}", host, port)
        } else {
            host.to_string()
        };

        let encoded = base64::engine::general_purpose::STANDARD.encode(key_bytes);
        let line = format!("{} {} {}\n", host_pattern, key_type, encoded);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.known_hosts_path)?;
        file.write_all(line.as_bytes())?;

        self.entries.push(KnownHostEntry {
            host_pattern,
            key_type: key_type.to_string(),
            key_data: key_bytes.to_vec(),
        });

        Ok(())
    }

    /// Returns the path to `~/.ssh/known_hosts`, creating the `.ssh` directory if needed.
    fn known_hosts_path() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let ssh_dir = home.join(".ssh");
        if !ssh_dir.exists() {
            std::fs::create_dir_all(&ssh_dir)?;
        }
        Ok(ssh_dir.join("known_hosts"))
    }

    /// Parses a `known_hosts` file into structured entries.
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

    /// Checks whether a `known_hosts` pattern matches the given host and port.
    fn host_matches(pattern: &str, host: &str, port: u16) -> bool {
        if pattern.starts_with('@') {
            return false;
        }

        for pattern in pattern.split(',') {
            let pattern = pattern.trim();

            if pattern == host {
                return true;
            }

            if pattern.starts_with('[') {
                if let Some(bracket_end) = pattern.find("]:") {
                    let pattern_host = &pattern[1..bracket_end];
                    let port_str = &pattern[bracket_end + 2..];
                    let pattern_port: u16 = port_str.parse().unwrap_or(22);
                    if pattern_host == host && pattern_port == port {
                        return true;
                    }
                } else if let Some(bracketed) =
                    pattern.strip_prefix('[').and_then(|s| s.strip_suffix(']'))
                    && bracketed == host
                    && port == 22
                {
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

    /// Matches a glob-style pattern (with `*` and `?`) against text using dynamic programming.
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

/// Convenience function: verifies a host key against `known_hosts`.
pub fn verify_host_key(host: &str, port: u16, public_key: &ssh_key::PublicKey) -> HostKeyStatus {
    match HostKeyVerifier::load() {
        Ok(verifier) => verifier.verify(host, port, public_key),
        Err(_) => HostKeyStatus::Unknown,
    }
}

/// Convenience function: adds a host key to `known_hosts`.
pub fn add_host_key(host: &str, port: u16, public_key: &ssh_key::PublicKey) -> anyhow::Result<()> {
    let mut verifier = HostKeyVerifier::load()?;
    verifier.add_host_key(host, port, public_key)
}

/// Convenience function: adds a raw host key entry to `known_hosts`.
pub fn add_host_key_raw(
    host: &str,
    port: u16,
    key_bytes: &[u8],
    key_type: &str,
) -> anyhow::Result<()> {
    let mut verifier = HostKeyVerifier::load()?;
    verifier.add_host_key_raw(host, port, key_bytes, key_type)
}

/// Computes a SHA-256 fingerprint string for the given public key.
pub fn compute_fingerprint(public_key: &ssh_key::PublicKey) -> Option<String> {
    let key_bytes = public_key.to_bytes().ok()?;
    let hash = Sha256::digest(key_bytes.as_ref() as &[u8]);
    Some(format!(
        "SHA256:{}",
        base64::engine::general_purpose::STANDARD.encode(hash)
    ))
}

/// Returns the SSH key type name (e.g. `"ssh-ed25519"`) for the given public key.
pub fn key_type_name(public_key: &ssh_key::PublicKey) -> &'static str {
    match public_key.algorithm() {
        ssh_key::Algorithm::Rsa { .. } => "ssh-rsa",
        ssh_key::Algorithm::Ed25519 => "ssh-ed25519",
        ssh_key::Algorithm::Ecdsa { curve } => match curve {
            ssh_key::EcdsaCurve::NistP256 => "ecdsa-sha2-nistp256",
            ssh_key::EcdsaCurve::NistP384 => "ecdsa-sha2-nistp384",
            ssh_key::EcdsaCurve::NistP521 => "ecdsa-sha2-nistp521",
        },
        ssh_key::Algorithm::SkEd25519 => "sk-ssh-ed25519@openssh.com",
        _ => "ssh-unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_matches_simple() {
        assert!(HostKeyVerifier::host_matches(
            "example.com",
            "example.com",
            22
        ));
        assert!(!HostKeyVerifier::host_matches(
            "example.com",
            "other.com",
            22
        ));
    }

    #[test]
    fn test_host_matches_with_port() {
        assert!(
            HostKeyVerifier::host_matches("[example.com]:2222", "example.com", 2222),
            "exact bracketed pattern should match"
        );
        assert!(
            !HostKeyVerifier::host_matches("[example.com]:2222", "example.com", 22),
            "wrong port should not match"
        );
        assert!(
            !HostKeyVerifier::host_matches("[example.com]:2222", "other.com", 2222),
            "wrong host should not match"
        );
    }

    #[test]
    fn test_host_matches_multiple_patterns() {
        assert!(HostKeyVerifier::host_matches(
            "server1,server2",
            "server1",
            22
        ));
        assert!(HostKeyVerifier::host_matches(
            "server1,server2",
            "server2",
            22
        ));
        assert!(!HostKeyVerifier::host_matches(
            "server1,server2",
            "server3",
            22
        ));
    }

    #[test]
    fn test_host_matches_wildcard() {
        assert!(HostKeyVerifier::host_matches(
            "*.example.com",
            "www.example.com",
            22
        ));
        assert!(HostKeyVerifier::host_matches(
            "*.example.com",
            "api.example.com",
            22
        ));
        assert!(!HostKeyVerifier::host_matches(
            "*.example.com",
            "example.com",
            22
        ));
        assert!(HostKeyVerifier::host_matches("server?", "server1", 22));
        assert!(HostKeyVerifier::host_matches("server?", "server2", 22));
        assert!(!HostKeyVerifier::host_matches("server?", "server12", 22));
    }

    #[test]
    fn test_host_matches_revoked_marker() {
        assert!(!HostKeyVerifier::host_matches(
            "@revoked example.com",
            "example.com",
            22
        ));
    }

    #[test]
    fn test_host_matches_bracketed_no_port() {
        assert!(
            HostKeyVerifier::host_matches("[example.com]", "example.com", 22),
            "bracketed host without port should match on port 22"
        );
    }

    #[test]
    fn test_host_matches_bracketed_no_port_wrong_port() {
        assert!(
            !HostKeyVerifier::host_matches("[example.com]", "example.com", 2222),
            "bracketed host without port should not match non-22 port"
        );
    }

    #[test]
    fn test_host_key_status_equality() {
        assert_eq!(HostKeyStatus::Trusted, HostKeyStatus::Trusted);
        assert_eq!(HostKeyStatus::Unknown, HostKeyStatus::Unknown);
        assert_eq!(HostKeyStatus::Changed, HostKeyStatus::Changed);
        assert_ne!(HostKeyStatus::Trusted, HostKeyStatus::Unknown);
        assert_ne!(HostKeyStatus::Changed, HostKeyStatus::Trusted);
    }

    #[test]
    fn test_host_key_status_debug() {
        assert!(format!("{:?}", HostKeyStatus::Trusted).contains("Trusted"));
        assert!(format!("{:?}", HostKeyStatus::Unknown).contains("Unknown"));
        assert!(format!("{:?}", HostKeyStatus::Changed).contains("Changed"));
    }

    #[test]
    fn test_host_matches_empty_pattern() {
        assert!(!HostKeyVerifier::host_matches("", "example.com", 22));
    }

    #[test]
    fn test_host_matches_localhost() {
        assert!(HostKeyVerifier::host_matches("localhost", "localhost", 22));
        assert!(!HostKeyVerifier::host_matches("localhost", "127.0.0.1", 22));
    }

    #[test]
    fn test_host_matches_ipv4() {
        assert!(HostKeyVerifier::host_matches(
            "192.168.1.1",
            "192.168.1.1",
            22
        ));
        assert!(!HostKeyVerifier::host_matches(
            "192.168.1.1",
            "192.168.1.2",
            22
        ));
    }

    #[test]
    fn test_host_matches_wildcard_subdomain() {
        assert!(HostKeyVerifier::host_matches(
            "*.example.com",
            "sub.example.com",
            22
        ));
        assert!(HostKeyVerifier::host_matches(
            "*.example.com",
            "a.b.example.com",
            22
        ));
    }

    #[test]
    fn test_host_matches_question_mark() {
        assert!(HostKeyVerifier::host_matches("host?", "host1", 22));
        assert!(HostKeyVerifier::host_matches("host?", "hostA", 22));
        assert!(!HostKeyVerifier::host_matches("host?", "host12", 22));
        assert!(!HostKeyVerifier::host_matches("host?", "host", 22));
    }

    #[test]
    fn test_host_matches_multiple_with_port() {
        assert!(HostKeyVerifier::host_matches(
            "server1,server2",
            "server1",
            22
        ));
        assert!(HostKeyVerifier::host_matches(
            "[server1]:2222,[server2]:2222",
            "server1",
            2222
        ));
    }

    #[test]
    fn test_key_type_name_matches() {
        let pairs = vec![
            ("*.example.com", "www.example.com"),
            ("?", "x"),
            ("*.*", "a.b"),
        ];
        for (pattern, host) in pairs {
            assert!(HostKeyVerifier::host_matches(pattern, host, 22));
        }
    }

    #[test]
    fn test_parse_line_valid() {
        let line = "example.com ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl";
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "example.com");
        assert_eq!(parts[1], "ssh-ed25519");
    }

    #[test]
    fn test_parse_line_too_short() {
        let line = "example.com ssh-ed25519";
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_host_matches_comma_patterns_all_must_fail() {
        assert!(!HostKeyVerifier::host_matches(
            "server1,server2,server3",
            "server4",
            22
        ));
    }
}
