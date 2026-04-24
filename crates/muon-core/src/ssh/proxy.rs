use base64::Engine;
use tokio::net::TcpStream;

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyType {
    None,
    Http,
    Socks5,
}

pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub async fn connect_via_proxy(
    target_host: &str,
    target_port: u16,
    proxy: &ProxyConfig,
) -> Result<TcpStream, std::io::Error> {
    let addr = format!("{}:{}", proxy.host, proxy.port);
    let mut stream = TcpStream::connect(&addr).await?;

    match proxy.proxy_type {
        ProxyType::None => unreachable!(),
        ProxyType::Http => {
            connect_http_proxy(&mut stream, target_host, target_port, proxy).await?;
            Ok(stream)
        }
        ProxyType::Socks5 => {
            connect_socks5_proxy(&mut stream, target_host, target_port, proxy).await?;
            Ok(stream)
        }
    }
}

async fn connect_http_proxy(
    stream: &mut TcpStream,
    target_host: &str,
    target_port: u16,
    proxy: &ProxyConfig,
) -> Result<(), std::io::Error> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n",
        target_host, target_port, target_host, target_port
    );

    if let (Some(user), Some(pass)) = (&proxy.username, &proxy.password) {
        use std::fmt::Write;
        let credentials =
            base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", user, pass));
        write!(request, "Proxy-Authorization: Basic {}\r\n", credentials).unwrap();
    }

    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).await?;

    let mut response = vec![0u8; 4096];
    let n = stream.read(&mut response).await?;
    let response_str = String::from_utf8_lossy(&response[..n]);

    if response_str.contains("200") {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Proxy connection failed: {}", response_str),
        ))
    }
}

async fn connect_socks5_proxy(
    stream: &mut TcpStream,
    target_host: &str,
    target_port: u16,
    proxy: &ProxyConfig,
) -> Result<(), std::io::Error> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let auth_method = if proxy.username.is_some() {
        0x02u8
    } else {
        0x00u8
    };

    stream.write_all(&[0x05, 0x01, auth_method]).await?;

    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;

    if buf[0] != 0x05 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Not a SOCKS5 proxy",
        ));
    }

    if buf[1] == 0x02
        && let (Some(user), Some(pass)) = (&proxy.username, &proxy.password)
    {
        let user_bytes = user.as_bytes();
        let pass_bytes = pass.as_bytes();
        let mut auth_req = vec![0x01, user_bytes.len() as u8];
        auth_req.extend_from_slice(user_bytes);
        auth_req.push(pass_bytes.len() as u8);
        auth_req.extend_from_slice(pass_bytes);
        stream.write_all(&auth_req).await?;

        let mut auth_resp = [0u8; 2];
        stream.read_exact(&mut auth_resp).await?;
        if auth_resp[1] != 0x00 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "SOCKS5 authentication failed",
            ));
        }
    }

    let host_bytes = target_host.as_bytes();
    let mut connect_req = vec![0x05, 0x01, 0x00, 0x03, host_bytes.len() as u8];
    connect_req.extend_from_slice(host_bytes);
    connect_req.extend_from_slice(&target_port.to_be_bytes());
    stream.write_all(&connect_req).await?;

    let mut resp = [0u8; 10];
    stream.read_exact(&mut resp).await?;

    if resp[1] != 0x00 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("SOCKS5 connect failed: status {}", resp[1]),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_type_equality() {
        assert_eq!(ProxyType::None, ProxyType::None);
        assert_eq!(ProxyType::Http, ProxyType::Http);
        assert_eq!(ProxyType::Socks5, ProxyType::Socks5);
        assert_ne!(ProxyType::None, ProxyType::Http);
        assert_ne!(ProxyType::Http, ProxyType::Socks5);
    }

    #[test]
    fn test_proxy_type_debug() {
        assert_eq!(format!("{:?}", ProxyType::None), "None");
        assert_eq!(format!("{:?}", ProxyType::Http), "Http");
        assert_eq!(format!("{:?}", ProxyType::Socks5), "Socks5");
    }

    #[test]
    fn test_proxy_config_fields() {
        let config = ProxyConfig {
            proxy_type: ProxyType::Http,
            host: "proxy.example.com".to_string(),
            port: 3128,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };
        assert_eq!(config.proxy_type, ProxyType::Http);
        assert_eq!(config.host, "proxy.example.com");
        assert_eq!(config.port, 3128);
        assert_eq!(config.username.as_deref(), Some("user"));
        assert_eq!(config.password.as_deref(), Some("pass"));
    }

    #[test]
    fn test_proxy_config_no_auth() {
        let config = ProxyConfig {
            proxy_type: ProxyType::Socks5,
            host: "socks.example.com".to_string(),
            port: 1080,
            username: None,
            password: None,
        };
        assert!(config.username.is_none());
        assert!(config.password.is_none());
    }

    #[test]
    fn test_proxy_config_fields_match() {
        let config = ProxyConfig {
            proxy_type: ProxyType::Http,
            host: "proxy.example.com".to_string(),
            port: 3128,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };
        let config2 = ProxyConfig {
            proxy_type: ProxyType::Http,
            host: "proxy.example.com".to_string(),
            port: 3128,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };
        assert_eq!(config.host, config2.host);
        assert_eq!(config.port, config2.port);
    }
}
