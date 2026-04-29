//! SSH key discovery, generation, and deployment.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Metadata about an SSH key pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyInfo {
    /// Filesystem path to the private key.
    pub path: String,
    /// Filename of the key.
    pub name: String,
    /// Key format (e.g. "OpenSSH", "RSA (PEM)").
    pub key_type: String,
    /// SHA-256 fingerprint of the public key, if available.
    pub fingerprint: Option<String>,
    /// Whether a `.pub` companion file exists.
    pub has_public_key: bool,
}

/// Utility for listing, generating, and deploying SSH keys.
pub struct KeyManager;

impl KeyManager {
    /// Lists all SSH private keys found in `~/.ssh/`.
    pub fn list_local_keys() -> anyhow::Result<Vec<SshKeyInfo>> {
        let ssh_dir = Self::ssh_dir()?;
        Ok(Self::list_local_keys_sync(&ssh_dir))
    }

    /// Lists public keys from the remote host's `authorized_keys`.
    pub async fn list_remote_keys(
        handle: &russh::client::Handle<super::connection::ClientHandler>,
    ) -> anyhow::Result<Vec<SshKeyInfo>> {
        let result = crate::tools::remote_exec::RemoteExecutor::execute(
            handle,
            "ls -1 ~/.ssh/*.pub 2>/dev/null; echo '---'; cat ~/.ssh/authorized_keys 2>/dev/null",
            10,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list remote keys: {}", e))?;

        let output = result.stdout_string();
        let mut keys = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line == "---" || line == "ls:" {
                continue;
            }

            if line.contains("ssh-") || line.contains("ecdsa-") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let key_type = parts.first().unwrap_or(&"").to_string();
                let fingerprint = if parts.len() >= 2 {
                    Some(Self::truncate_fingerprint(parts[1]))
                } else {
                    None
                };
                let comment = parts.get(2).map(|s| s.to_string());

                keys.push(SshKeyInfo {
                    path: "~/.ssh/authorized_keys".to_string(),
                    name: comment.unwrap_or_else(|| "key".to_string()),
                    key_type,
                    fingerprint,
                    has_public_key: true,
                });
            }
        }

        Ok(keys)
    }

    /// Generates a new SSH key pair using `ssh-keygen`.
    pub async fn generate_key_pair(
        algorithm: &str,
        path: &str,
        passphrase: Option<&str>,
    ) -> anyhow::Result<SshKeyInfo> {
        let key_path = PathBuf::from(path);
        let pub_path = key_path.with_extension("pub");

        let alg = match algorithm {
            "ed25519" => "ed25519",
            "ecdsa" => "ecdsa",
            "rsa" => "rsa",
            _ => "ed25519",
        };

        let bits = if alg == "rsa" {
            " -b 4096"
        } else if alg == "ecdsa" {
            " -b 521"
        } else {
            ""
        };

        let mut cmd = std::process::Command::new("ssh-keygen");
        cmd.arg("-t")
            .arg(alg)
            .arg("-f")
            .arg(&key_path)
            .arg("-N")
            .arg(passphrase.unwrap_or(""))
            .arg("-C")
            .arg("muon-ssh-generated");

        if !bits.is_empty() {
            cmd.arg("-b");
            let b = bits.trim_start_matches(" -b ");
            cmd.arg(b);
        }

        let output = tokio::task::spawn_blocking(move || cmd.output())
            .await
            .map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?
            .map_err(|e| anyhow::anyhow!("Key generation failed: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("ssh-keygen failed: {}", stderr));
        }

        let fingerprint = Self::read_public_key_fingerprint(&pub_path);

        Ok(SshKeyInfo {
            path: key_path.to_string_lossy().to_string(),
            name: key_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("key")
                .to_string(),
            key_type: algorithm.to_string(),
            fingerprint,
            has_public_key: true,
        })
    }

    /// Deploys a public key string to the remote host's `authorized_keys`.
    pub async fn deploy_public_key(
        handle: &russh::client::Handle<super::connection::ClientHandler>,
        public_key: &str,
    ) -> anyhow::Result<()> {
        let escaped = public_key.replace('\'', "'\\''");
        let command = format!(
            "mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '{}' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && sort -u ~/.ssh/authorized_keys -o ~/.ssh/authorized_keys",
            escaped
        );

        let result = crate::tools::remote_exec::RemoteExecutor::execute(handle, &command, 15)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to deploy key: {}", e))?;

        if result.exit_code != 0 {
            return Err(anyhow::anyhow!(
                "Deploy failed with exit code {}",
                result.exit_code
            ));
        }

        Ok(())
    }

    /// Reads the `.pub` file corresponding to the given private key path.
    pub fn read_public_key(path: &str) -> anyhow::Result<String> {
        let pub_path = PathBuf::from(path).with_extension("pub");
        Ok(std::fs::read_to_string(&pub_path)?.trim().to_string())
    }

    /// Returns the path to `~/.ssh/`, creating it if needed.
    fn ssh_dir() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let ssh_dir = home.join(".ssh");
        if !ssh_dir.exists() {
            std::fs::create_dir_all(&ssh_dir)?;
        }
        Ok(ssh_dir)
    }

    /// Reads the fingerprint of a public key file using `ssh-keygen -lf`.
    fn read_public_key_fingerprint(pub_path: &PathBuf) -> Option<String> {
        if !pub_path.exists() {
            return None;
        }
        let output = std::process::Command::new("ssh-keygen")
            .arg("-lf")
            .arg(pub_path)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let line = String::from_utf8_lossy(&output.stdout);
        Some(line.trim().to_string())
    }

    /// Truncates a fingerprint string to 20 characters with an ellipsis.
    fn truncate_fingerprint(hash: &str) -> String {
        if hash.len() > 20 {
            format!("{}...", &hash[..20])
        } else {
            hash.to_string()
        }
    }

    /// Synchronous implementation that scans a directory for SSH private keys.
    pub fn list_local_keys_sync(ssh_dir: &std::path::Path) -> Vec<SshKeyInfo> {
        let mut keys = Vec::new();
        let entries = match std::fs::read_dir(ssh_dir) {
            Ok(e) => e,
            Err(_) => return keys,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if name.ends_with(".pub")
                || name.starts_with('.')
                || name == "known_hosts"
                || name == "authorized_keys"
                || name == "config"
                || name == "sshd_config"
            {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if !content.contains("PRIVATE KEY") && !content.contains("SSH PRIVATE KEY") {
                continue;
            }

            let key_type = if content.contains("BEGIN OPENSSH PRIVATE KEY") {
                "OpenSSH".to_string()
            } else if content.contains("BEGIN RSA PRIVATE KEY") {
                "RSA (PEM)".to_string()
            } else if content.contains("BEGIN EC PRIVATE KEY") {
                "EC (PEM)".to_string()
            } else if content.contains("BEGIN PRIVATE KEY") {
                "PKCS8".to_string()
            } else {
                "Unknown".to_string()
            };

            let pub_path = path.with_extension("pub");
            let has_public_key = pub_path.exists();
            let fingerprint = Self::read_public_key_fingerprint(&pub_path);

            keys.push(SshKeyInfo {
                path: path.to_string_lossy().to_string(),
                name,
                key_type,
                fingerprint,
                has_public_key,
            });
        }

        keys.sort_by(|a, b| a.name.cmp(&b.name));
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_key_info_serialization() {
        let info = SshKeyInfo {
            path: "/home/user/.ssh/id_rsa".to_string(),
            name: "id_rsa".to_string(),
            key_type: "RSA (PEM)".to_string(),
            fingerprint: Some("SHA256:abc123".to_string()),
            has_public_key: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("id_rsa"));
        let parsed: SshKeyInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.path, info.path);
        assert_eq!(parsed.name, "id_rsa");
        assert_eq!(parsed.key_type, "RSA (PEM)");
    }

    #[test]
    fn test_ssh_key_info_clone() {
        let info = SshKeyInfo {
            path: "/home/user/.ssh/id_ed25519".to_string(),
            name: "id_ed25519".to_string(),
            key_type: "OpenSSH".to_string(),
            fingerprint: None,
            has_public_key: false,
        };
        let cloned = info.clone();
        assert_eq!(cloned.path, info.path);
        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.fingerprint, None);
    }

    #[test]
    fn test_ssh_key_info_debug() {
        let info = SshKeyInfo {
            path: "/test".to_string(),
            name: "test_key".to_string(),
            key_type: "Ed25519".to_string(),
            fingerprint: Some("fp".to_string()),
            has_public_key: true,
        };
        let debug = format!("{:?}", info);
        assert!(debug.contains("test_key"));
    }

    #[test]
    fn test_truncate_fingerprint_short() {
        let result = KeyManager::truncate_fingerprint("short");
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_fingerprint_exactly_20() {
        let hash: String = "a".repeat(20);
        let result = KeyManager::truncate_fingerprint(&hash);
        assert_eq!(result, hash);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn test_truncate_fingerprint_long() {
        let hash: String = "a".repeat(50);
        let result = KeyManager::truncate_fingerprint(&hash);
        assert_eq!(result.len(), 23);
        assert!(result.ends_with("..."));
        assert_eq!(&result[..20], &hash[..20]);
    }

    #[test]
    fn test_truncate_fingerprint_empty() {
        let result = KeyManager::truncate_fingerprint("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_read_public_key_nonexistent() {
        let result = KeyManager::read_public_key("/nonexistent/path/id_rsa");
        assert!(result.is_err());
    }

    #[test]
    fn test_ssh_key_info_no_fingerprint() {
        let info = SshKeyInfo {
            path: "/test/key".to_string(),
            name: "test".to_string(),
            key_type: "Unknown".to_string(),
            fingerprint: None,
            has_public_key: false,
        };
        assert!(info.fingerprint.is_none());
        assert!(!info.has_public_key);
    }

    #[test]
    fn test_ssh_key_info_key_types() {
        let types = vec!["OpenSSH", "RSA (PEM)", "EC (PEM)", "PKCS8", "Unknown"];
        for kt in types {
            let info = SshKeyInfo {
                path: "/test".to_string(),
                name: "key".to_string(),
                key_type: kt.to_string(),
                fingerprint: None,
                has_public_key: false,
            };
            assert_eq!(info.key_type, kt);
        }
    }

    #[test]
    fn test_truncate_fingerprint_realistic() {
        let fp = "SHA256:uNiVztksCsDhcc0u9e8BgrJXVGDewarHSa0ZnNd7N4k";
        let result = KeyManager::truncate_fingerprint(fp);
        assert!(result.ends_with("..."));
        assert_eq!(&result[..20], &fp[..20]);
    }

    #[test]
    fn test_list_local_keys_no_ssh_dir() {
        if std::env::var("HOME").is_ok() {
            let _ = KeyManager::list_local_keys();
        }
    }
}
