use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyInfo {
    pub path: String,
    pub name: String,
    pub key_type: String,
    pub fingerprint: Option<String>,
    pub has_public_key: bool,
}

pub struct KeyManager;

impl KeyManager {
    pub fn list_local_keys() -> anyhow::Result<Vec<SshKeyInfo>> {
        let ssh_dir = Self::ssh_dir()?;
        let mut keys = Vec::new();

        let entries = std::fs::read_dir(&ssh_dir)?;
        for entry in entries {
            let entry = entry?;
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
        Ok(keys)
    }

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

    pub fn generate_key_pair(
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

        let output = cmd.output()?;
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

    pub fn read_public_key(path: &str) -> anyhow::Result<String> {
        let pub_path = PathBuf::from(path).with_extension("pub");
        Ok(std::fs::read_to_string(&pub_path)?.trim().to_string())
    }

    fn ssh_dir() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let ssh_dir = home.join(".ssh");
        if !ssh_dir.exists() {
            std::fs::create_dir_all(&ssh_dir)?;
        }
        Ok(ssh_dir)
    }

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

    fn truncate_fingerprint(hash: &str) -> String {
        if hash.len() > 20 {
            format!("{}...", &hash[..20])
        } else {
            hash.to_string()
        }
    }
}
