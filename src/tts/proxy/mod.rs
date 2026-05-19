use thiserror::Error;

/// Proxy Error
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("not supported scheme: {0}")]
    NotSupportedScheme(http::Uri),
    #[error("http proxy error: {0}")]
    HttpProxyError(#[from] HttpProxyError),
    #[error("socks4 proxy error: {0}")]
    Socks4ProxyError(#[from] Socks4ProxyError),
    #[error("socks5 proxy error: {0}")]
    Socks5ProxyError(#[from] Socks5ProxyError),
}

/// Http Proxy Error
#[derive(Error, Debug)]
pub enum HttpProxyError {
    #[error("no proxy server host name: {0}")]
    NoProxyServerHostName(http::Uri),
    #[error("proxy host name is empty: {0}")]
    EmptyProxyServerHostName(http::Uri),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "blocking")]
    #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
    #[error("tls error: {0}")]
    TlsError(#[from] crate::tts::blocking::TlsError),

    #[error("invalid response: {0}")]
    InvalidResponse(#[from] httparse::Error),
    #[error("bad response: {0} {1}")]
    BadResponse(u16, String),
    #[error("no status code")]
    NoStatusCode,
    #[error("not supported scheme: {0}")]
    NotSupportedScheme(http::Uri),
}

/// Socks4 Proxy Error
#[derive(Error, Debug)]
pub enum Socks4ProxyError {
    #[error("no proxy server host name: {0}")]
    NoProxyServerHostName(http::Uri),
    #[error("no proxy server port: {0}")]
    NoProxyServerPort(http::Uri),
    #[error("proxy host name is empty: {0}")]
    EmptyProxyServerHostName(http::Uri),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("empty scheme: {0}")]
    NoScheme(http::Uri),
    #[error("not supported scheme: {0}")]
    NotSupportedScheme(http::Uri),
    #[error("lookup ip v4 addrs failed: {0}")]
    NoIpV4Addr(String),
    #[error("request rejected or failed")]
    RequestRejectedOrFailed(u8),
    #[error("no available identd service")]
    NoneAvailableIdentdService(u8),
    #[error("identd check failed: {0}")]
    IdentdCheckFailed(u8),
    #[error("unknown reply code: {0}")]
    UnknownReplyCode(u8),
}

/// Socks5 Proxy Error
#[derive(Error, Debug)]
pub enum Socks5ProxyError {
    #[error("no proxy server host name: {0}")]
    NoProxyServerHostName(http::Uri),
    #[error("no proxy server port: {0}")]
    NoProxyServerPort(http::Uri),
    #[error("proxy host name is empty: {0}")]
    EmptyProxyServerHostName(http::Uri),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("empty scheme: {0}")]
    NoScheme(http::Uri),
    #[error("not supported scheme: {0}")]
    NotSupportedScheme(http::Uri),
    #[error("bad response version: {0}")]
    BadResponseVersion(u8),
    #[error("bad server choice: {0:?}")]
    BadServerChoice(u8),
    #[error("client authentication failed: {0:?}")]
    ClientAuthenticationFailed([u8; 2]),
    #[error("lookup ip addrs failed: {0}")]
    NoIpAddr(String),
    #[error("not supported server bind address type: {0}")]
    NotSupportedServerBindAddressType(u8),
    #[error("general failure: {0}")]
    GeneralFailure(u8),
    #[error("connection not allowed by rules: {0}")]
    ConnectionNotAllowedByRules(u8),
    #[error("network unreachable: {0}")]
    NetworkUnreachable(u8),
    #[error("host unreachable: {0}")]
    HostUnreachable(u8),
    #[error("connection refused: {0}")]
    ConnectionRefused(u8),
    #[error("ttl expired: {0}")]
    TtlExpired(u8),
    #[error("command not supported: {0}")]
    CommandNotSupported(u8),
    #[error("address type not supported: {0}")]
    AddressTypeNotSupported(u8),
    #[error("unknown reply code: {0}")]
    UnknownReplyCode(u8),
}

/// VER (1), CMD (1), DSTPORT (2), DSTIP (4), ID (? + 1), socks4a? DOMAIN (? + 1)
fn build_socks4_connection_request(
    target_host: &str,
    dst_ip: Option<std::net::Ipv4Addr>,
    username: Option<&str>,
) -> Vec<u8> {
    // VER, CMD, DSTPORT
    let mut bytes = vec![0x04, 0x01, 0x01, 0xbb];

    // DSTIP (4)
    if let Some(ip) = dst_ip {
        bytes.extend(ip.octets()); // socks4
    } else {
        bytes.extend([0x00, 0x00, 0x00, 0x01]); // socks4a
    }

    // ID (? + 1)
    if let Some(username) = username {
        bytes.extend(username.as_bytes());
    }
    bytes.push(0x00);

    // socks4a DOMAIN (? + 1)
    if dst_ip.is_none() {
        bytes.extend(target_host.as_bytes());
        bytes.push(0x00);
    }

    bytes
}

/// VER (1), IDLEN (1), ID (IDLEN), PWLEN (1), PW (PWLEN)
fn build_socks5_authentication_request(username: &str, password: &str) -> Vec<u8> {
    let mut bytes = vec![0x01]; // VER
    if username.len() == 0 {
        bytes.extend([0x01, 0x00]);
    } else {
        bytes.push(username.len() as u8); // IDLEN
        bytes.extend(username.as_bytes()); // ID
    }
    if password.len() == 0 {
        bytes.extend([0x01, 0x00]);
    } else {
        bytes.push(password.len() as u8); // PWLEN
        bytes.extend(password.as_bytes()); // PW
    }
    bytes
}

/// VER (1), CMD (1), RSV (1), DSTADDR [TYPE (1), ADDR (?)], DSTPORT (2)
fn build_socks5_connection_request(target_host: &str, dst_ip: Option<std::net::IpAddr>) -> Vec<u8> {
    // VER, CMD, RSV
    let mut bytes = vec![0x05, 0x01, 0x00];

    // DSTADDR
    if let Some(ip) = dst_ip {
        match ip {
            std::net::IpAddr::V4(ip) => {
                bytes.push(0x01); // TYPE
                bytes.extend(ip.octets()); // ADDR
            }
            std::net::IpAddr::V6(ip) => {
                bytes.push(0x04); // TYPE
                bytes.extend(ip.octets()); // ADDR
            }
        }
    } else {
        bytes.push(0x03); // TYPE
        bytes.push(target_host.len() as u8); // ADDRLEN
        bytes.extend(target_host.as_bytes()); // ADDR
    }

    // DSTPORT
    bytes.extend([0x01, 0xbb]);

    bytes
}

fn build_http_proxy_request(
    target_host: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> String {
    use base64::*;

    if username.is_some() && password.is_some() {
        let credential = base64::prelude::BASE64_STANDARD.encode(format!(
            "{}:{}",
            username.unwrap(),
            password.unwrap()
        ));
        format!(
            "CONNECT {}:443 HTTP/1.1\r\nHost: {}:443\r\nProxy-Authorization: Basic {}\r\nProxy-Connection: Keep-Alive\r\n\r\n",
            target_host, target_host, credential
        )
    } else {
        format!(
            "CONNECT {}:443 HTTP/1.1\r\nHost: {}:443\r\nProxy-Connection: Keep-Alive\r\n\r\n",
            target_host, target_host
        )
    }
}

#[cfg(feature = "blocking")]
pub(crate) mod blocking;
