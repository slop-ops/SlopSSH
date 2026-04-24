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
    KeyboardInteractive {
        responses: Vec<String>,
    },
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_auth() {
        let auth = AuthMethod::Password {
            password: "secret".to_string(),
        };
        match auth {
            AuthMethod::Password { password } => assert_eq!(password, "secret"),
            _ => panic!("Expected Password variant"),
        }
    }

    #[test]
    fn test_publickey_auth() {
        let auth = AuthMethod::PublicKey {
            key_path: PathBuf::from("/home/user/.ssh/id_rsa"),
            passphrase: Some("mypass".to_string()),
        };
        match auth {
            AuthMethod::PublicKey {
                key_path,
                passphrase,
            } => {
                assert_eq!(key_path, PathBuf::from("/home/user/.ssh/id_rsa"));
                assert_eq!(passphrase, Some("mypass".to_string()));
            }
            _ => panic!("Expected PublicKey variant"),
        }
    }

    #[test]
    fn test_none_auth() {
        let auth = AuthMethod::None;
        assert!(matches!(auth, AuthMethod::None));
    }

    #[test]
    fn test_keyboard_interactive_auth() {
        let auth = AuthMethod::KeyboardInteractive {
            responses: vec!["password123".to_string()],
        };
        match auth {
            AuthMethod::KeyboardInteractive { responses } => {
                assert_eq!(responses.len(), 1);
                assert_eq!(responses[0], "password123");
            }
            _ => panic!("Expected KeyboardInteractive variant"),
        }
    }
}
