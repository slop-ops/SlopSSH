//! Utility functions for path validation, shell escaping, and AES-256-GCM
//! encryption of credential values.

pub fn validate_sftp_path(path: &str) -> Result<String, String> {
    if path.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    if path.contains('\0') {
        return Err("Path contains null bytes".to_string());
    }
    let normalized = if path.starts_with('/') || path.starts_with('~') {
        normalize_path(path)
    } else {
        normalize_path(&format!("./{}", path))
    };
    Ok(normalized)
}

/// Normalises a POSIX-style path by resolving `.` and `..` segments.
fn normalize_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let mut result: Vec<&str> = Vec::new();
    for part in parts {
        match part {
            "" | "." => {}
            ".." => {
                result.pop();
            }
            _ => result.push(part),
        }
    }
    if path.starts_with('/') {
        format!("/{}", result.join("/"))
    } else if result.is_empty() {
        ".".to_string()
    } else {
        result.join("/")
    }
}

/// Escapes a string for safe interpolation into a POSIX shell command.
///
/// Wraps the value in single quotes when it contains any shell-active
/// characters.
pub fn shell_escape(s: &str) -> String {
    if s.contains(' ')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('$')
        || s.contains('`')
        || s.contains('\\')
        || s.contains('(')
        || s.contains(')')
        || s.contains('&')
        || s.contains('|')
        || s.contains(';')
        || s.contains('<')
        || s.contains('>')
        || s.contains('*')
        || s.contains('?')
        || s.contains('~')
    {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.to_string()
    }
}

/// Encrypts a plaintext string using AES-256-GCM with a machine-derived key.
///
/// Returns a base64-encoded string containing the nonce prepended to the
/// ciphertext.
pub fn encrypt_value(plaintext: &str) -> anyhow::Result<String> {
    use aes_gcm::Aes256Gcm;
    use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
    use base64::Engine;

    let key_bytes = derive_machine_key();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Cipher init failed: {}", e))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

/// Decrypts a base64-encoded AES-256-GCM ciphertext produced by
/// [`encrypt_value`].
pub fn decrypt_value(encoded: &str) -> anyhow::Result<String> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};
    use base64::Engine;

    let key_bytes = derive_machine_key();
    let combined = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))?;

    if combined.len() < 12 {
        return Err(anyhow::anyhow!("Ciphertext too short"));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Cipher init failed: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("UTF-8 decode failed: {}", e))
}

/// Derives a 32-byte AES key from the config directory path and current user
/// name using SHA-256.
fn derive_machine_key() -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let config_path = crate::config::paths::config_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default();
    let seed = format!("muon-ssh-credential-key:{}:{}", config_path, user);

    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_simple() {
        assert_eq!(shell_escape("hello"), "hello");
    }

    #[test]
    fn test_shell_escape_spaces() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
    }

    #[test]
    fn test_shell_escape_single_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_shell_escape_dollar() {
        assert_eq!(shell_escape("$var"), "'$var'");
    }

    #[test]
    fn test_shell_escape_backtick() {
        assert_eq!(shell_escape("`cmd`"), "'`cmd`'");
    }

    #[test]
    fn test_shell_escape_semicolon() {
        assert_eq!(shell_escape("a;b"), "'a;b'");
    }

    #[test]
    fn test_shell_escape_pipe() {
        assert_eq!(shell_escape("a|b"), "'a|b'");
    }

    #[test]
    fn test_shell_escape_ampersand() {
        assert_eq!(shell_escape("a&b"), "'a&b'");
    }

    #[test]
    fn test_shell_escape_parentheses() {
        assert_eq!(shell_escape("(cmd)"), "'(cmd)'");
    }

    #[test]
    fn test_shell_escape_redirect() {
        assert_eq!(shell_escape(">file"), "'>file'");
    }

    #[test]
    fn test_shell_escape_wildcard() {
        assert_eq!(shell_escape("*.txt"), "'*.txt'");
    }

    #[test]
    fn test_shell_escape_tilde() {
        assert_eq!(shell_escape("~/path"), "'~/path'");
    }

    #[test]
    fn test_shell_escape_backslash() {
        assert_eq!(shell_escape("a\\b"), "'a\\b'");
    }

    #[test]
    fn test_shell_escape_empty() {
        assert_eq!(shell_escape(""), "");
    }

    #[test]
    fn test_shell_escape_path_with_spaces() {
        assert_eq!(
            shell_escape("/path/to/my file.txt"),
            "'/path/to/my file.txt'"
        );
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = "my-secret-password";
        let encrypted = encrypt_value(plaintext).unwrap();
        let decrypted = decrypt_value(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        let plaintext = "same-value";
        let enc1 = encrypt_value(plaintext).unwrap();
        let enc2 = encrypt_value(plaintext).unwrap();
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        assert!(decrypt_value("not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        use base64::Engine;
        let short = base64::engine::general_purpose::STANDARD.encode(b"short");
        assert!(decrypt_value(&short).is_err());
    }

    #[test]
    fn test_encrypt_decrypt_empty_string() {
        let encrypted = encrypt_value("").unwrap();
        let decrypted = decrypt_value(&encrypted).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_encrypt_decrypt_unicode() {
        let plaintext = "パスワード🔐";
        let encrypted = encrypt_value(plaintext).unwrap();
        let decrypted = decrypt_value(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_derive_machine_key_deterministic() {
        let key1 = derive_machine_key();
        let key2 = derive_machine_key();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_machine_key_is_32_bytes() {
        let key = derive_machine_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_validate_sftp_path_absolute() {
        assert_eq!(
            validate_sftp_path("/home/user/file.txt").unwrap(),
            "/home/user/file.txt"
        );
    }

    #[test]
    fn test_validate_sftp_path_root() {
        assert_eq!(validate_sftp_path("/").unwrap(), "/");
    }

    #[test]
    fn test_validate_sftp_path_empty() {
        assert!(validate_sftp_path("").is_err());
    }

    #[test]
    fn test_validate_sftp_path_whitespace() {
        assert!(validate_sftp_path("   ").is_err());
    }

    #[test]
    fn test_validate_sftp_path_null_bytes() {
        assert!(validate_sftp_path("/foo\0bar").is_err());
    }

    #[test]
    fn test_validate_sftp_path_traversal() {
        let result = validate_sftp_path("/home/user/../../etc/passwd").unwrap();
        assert_eq!(result, "/etc/passwd");
    }

    #[test]
    fn test_validate_sftp_path_double_dots() {
        let result = validate_sftp_path("/a/b/../c").unwrap();
        assert_eq!(result, "/a/c");
    }

    #[test]
    fn test_validate_sftp_path_dots() {
        let result = validate_sftp_path("/a/./b/./c").unwrap();
        assert_eq!(result, "/a/b/c");
    }

    #[test]
    fn test_validate_sftp_path_tilde() {
        let result = validate_sftp_path("~/documents").unwrap();
        assert_eq!(result, "~/documents");
    }

    #[test]
    fn test_normalize_path_trailing_parent() {
        let result = validate_sftp_path("/home/user/..").unwrap();
        assert_eq!(result, "/home");
    }

    #[test]
    fn test_normalize_path_complex() {
        let result = validate_sftp_path("/a/b/../../c").unwrap();
        assert_eq!(result, "/c");
    }
}
