use crate::error::{HttpProxyError, Socks4ProxyError, Socks5ProxyError};
use base64::Engine;
use futures_util::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io::{Read, Write};
use std::pin::pin;
use std::result::Result;

pub enum ProxyStream {
    TcpStream(std::net::TcpStream),
    TlsStream(native_tls::TlsStream<std::net::TcpStream>),
}

impl std::io::Read for ProxyStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::TcpStream(stream) => stream.read(buf),
            Self::TlsStream(stream) => stream.read(buf),
        }
    }
}

impl std::io::Write for ProxyStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::TcpStream(stream) => stream.write(buf),
            Self::TlsStream(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::TcpStream(stream) => stream.flush(),
            Self::TlsStream(stream) => stream.flush(),
        }
    }
}

pub enum ProxyAsyncStream {
    TcpStream(async_std::net::TcpStream),
    TlsStream(async_native_tls::TlsStream<async_std::net::TcpStream>),
}

impl AsyncRead for ProxyAsyncStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::TcpStream(stream) => pin!(stream).poll_read(cx, buf),
            Self::TlsStream(stream) => pin!(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ProxyAsyncStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::TcpStream(stream) => pin!(stream).poll_write(cx, buf),
            Self::TlsStream(stream) => pin!(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::TcpStream(stream) => pin!(stream).poll_flush(cx),
            Self::TlsStream(stream) => pin!(stream).poll_flush(cx),
        }
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::TcpStream(stream) => pin!(stream).poll_close(cx),
            Self::TlsStream(stream) => pin!(stream).poll_close(cx),
        }
    }
}

pub fn http_proxy(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> std::result::Result<ProxyStream, HttpProxyError> {
    if proxy.host().is_none() {
        return Err(HttpProxyError::NoProxyServerHostName(proxy));
    }
    let proxy_host = proxy.host().unwrap();
    if proxy_host.is_empty() {
        return Err(HttpProxyError::EmptyProxyServerHostName(proxy));
    }
    let proxy_port = proxy.port_u16().unwrap_or(match proxy.scheme_str() {
        None => 80,
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "https" => 443,
            "http" | _ => 80,
        },
    });

    let mut stream = match proxy.scheme_str() {
        None => {
            let stream = std::net::TcpStream::connect((proxy_host, proxy_port))?;
            ProxyStream::TcpStream(stream)
        }
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "http" => {
                let stream = std::net::TcpStream::connect((proxy_host, proxy_port))?;
                ProxyStream::TcpStream(stream)
            }
            "https" => {
                let connector = native_tls::TlsConnector::new()?;
                let stream = std::net::TcpStream::connect((proxy_host, proxy_port))?;
                let stream = connector.connect(proxy_host, stream).map_err(|e| match e {
                    native_tls::HandshakeError::Failure(f) => f,
                    native_tls::HandshakeError::WouldBlock(_) => {
                        panic!("Bug: TLS handshake not blocked")
                    }
                })?;
                ProxyStream::TlsStream(stream)
            }
            _ => return Err(HttpProxyError::NotSupportedScheme(proxy)),
        },
    };
    stream.write_all(build_http_proxy_request(target_host, username, password).as_bytes())?;
    stream.flush()?;

    let mut buf = [0u8; 1024];
    let mut n = 0;
    loop {
        n += stream.read(&mut buf[n..])?;
        if n >= 4 && &buf[n - 4..n] == b"\r\n\r\n" {
            break;
        }
    }

    let mut headers = [httparse::EMPTY_HEADER; 5];
    let mut response = httparse::Response::new(&mut headers);
    response.parse(&buf)?;

    match response.code {
        None => Err(HttpProxyError::NoStatusCode),
        Some(200) => Ok(stream),
        Some(code) => Err(HttpProxyError::BadResponse(
            code,
            response.reason.unwrap_or("").to_owned(),
        )),
    }
}

pub async fn http_proxy_async(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<ProxyAsyncStream, HttpProxyError> {
    if proxy.host().is_none() {
        return Err(HttpProxyError::NoProxyServerHostName(proxy));
    }
    let proxy_host = proxy.host().unwrap();
    if proxy_host.is_empty() {
        return Err(HttpProxyError::EmptyProxyServerHostName(proxy));
    }
    let proxy_port = proxy.port_u16().unwrap_or(match proxy.scheme_str() {
        None => 80,
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "https" => 443,
            "http" | _ => 80,
        },
    });

    let mut stream = match proxy.scheme_str() {
        None => {
            let stream = async_std::net::TcpStream::connect((proxy_host, proxy_port)).await?;
            ProxyAsyncStream::TcpStream(stream)
        }
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "http" => {
                let stream = async_std::net::TcpStream::connect((proxy_host, proxy_port)).await?;
                ProxyAsyncStream::TcpStream(stream)
            }
            "https" => {
                let stream = async_std::net::TcpStream::connect((proxy_host, proxy_port)).await?;
                let stream = async_native_tls::connect(proxy_host, stream).await?;
                ProxyAsyncStream::TlsStream(stream)
            }
            _ => return Err(HttpProxyError::NotSupportedScheme(proxy)),
        },
    };
    stream
        .write_all(build_http_proxy_request(target_host, username, password).as_bytes())
        .await?;
    stream.flush().await?;

    let mut buf = [0u8; 1024];
    let mut n = 0;
    loop {
        n += stream.read(&mut buf[n..]).await?;
        if n >= 4 && &buf[n - 4..n] == b"\r\n\r\n" {
            break;
        }
    }

    let mut headers = [httparse::EMPTY_HEADER; 5];
    let mut response = httparse::Response::new(&mut headers);
    response.parse(&buf)?;

    match response.code {
        None => Err(HttpProxyError::NoStatusCode),
        Some(200) => Ok(stream),
        Some(code) => Err(HttpProxyError::BadResponse(
            code,
            response.reason.unwrap_or("").to_owned(),
        )),
    }
}

fn build_http_proxy_request(
    target_host: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> String {
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

pub fn socks4_proxy(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
) -> Result<ProxyStream, Socks4ProxyError> {
    use std::net::ToSocketAddrs;

    if proxy.scheme_str().is_none() {
        return Err(Socks4ProxyError::NoScheme(proxy));
    }
    if proxy.host().is_none() {
        return Err(Socks4ProxyError::NoProxyServerHostName(proxy));
    }
    let proxy_host = proxy.host().unwrap();
    if proxy_host.is_empty() {
        return Err(Socks4ProxyError::EmptyProxyServerHostName(proxy));
    }
    if proxy.port_u16().is_none() {
        return Err(Socks4ProxyError::NoProxyServerPort(proxy));
    }
    let proxy_port = proxy.port_u16().unwrap();

    let mut stream = std::net::TcpStream::connect((proxy_host, proxy_port))?;
    let request = match proxy.scheme_str().unwrap().to_lowercase().as_str() {
        "socks4" => {
            let mut socket_addrs = (target_host, 443).to_socket_addrs()?;
            let ipv4 = loop {
                match socket_addrs.next() {
                    Some(socket_addr) => match socket_addr.ip() {
                        std::net::IpAddr::V4(ip) => break Some(ip),
                        std::net::IpAddr::V6(_) => {}
                    },
                    None => break None,
                }
            };

            if ipv4.is_none() {
                return Err(Socks4ProxyError::NoIpV4Addr(format!("{}:443", target_host)));
            }
            build_socks4_connection_request(target_host, ipv4, username)
        }
        "socks4a" => build_socks4_connection_request(target_host, None, username),
        _ => return Err(Socks4ProxyError::NotSupportedScheme(proxy)),
    };
    stream.write_all(&request)?;
    stream.flush()?;

    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf)?;
    match buf[1] {
        0x5a => Ok(ProxyStream::TcpStream(stream)),
        0x5b => Err(Socks4ProxyError::RequestRejectedOrFailed(0x5b)),
        0x5c => Err(Socks4ProxyError::NoneAvailableIdentdService(0x5c)),
        0x5d => Err(Socks4ProxyError::IdentdCheckFailed(0x5d)),
        code => Err(Socks4ProxyError::UnknownReplyCode(code)),
    }
}

pub async fn socks4_proxy_async(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
) -> Result<ProxyAsyncStream, Socks4ProxyError> {
    use async_std::net::ToSocketAddrs;

    if proxy.scheme_str().is_none() {
        return Err(Socks4ProxyError::NoScheme(proxy));
    }
    if proxy.host().is_none() {
        return Err(Socks4ProxyError::NoProxyServerHostName(proxy));
    }
    let proxy_host = proxy.host().unwrap();
    if proxy_host.is_empty() {
        return Err(Socks4ProxyError::EmptyProxyServerHostName(proxy));
    }
    if proxy.port_u16().is_none() {
        return Err(Socks4ProxyError::NoProxyServerPort(proxy));
    }
    let proxy_port = proxy.port_u16().unwrap();

    let mut stream = async_std::net::TcpStream::connect((proxy_host, proxy_port)).await?;
    let request = match proxy.scheme_str().unwrap().to_lowercase().as_str() {
        "socks4" => {
            let mut socket_addrs = (target_host, 443).to_socket_addrs().await?;
            let ipv4 = loop {
                match socket_addrs.next() {
                    Some(socket_addr) => match socket_addr.ip() {
                        std::net::IpAddr::V4(ip) => break Some(ip),
                        std::net::IpAddr::V6(_) => {}
                    },
                    None => break None,
                }
            };

            if ipv4.is_none() {
                return Err(Socks4ProxyError::NoIpV4Addr(format!("{}:443", target_host)));
            }
            build_socks4_connection_request(target_host, ipv4, username)
        }
        "socks4a" => build_socks4_connection_request(target_host, None, username),
        _ => return Err(Socks4ProxyError::NotSupportedScheme(proxy)),
    };
    stream.write_all(&request).await?;
    stream.flush().await?;

    let mut buf = [0u8; 8];
    stream.read_exact(&mut buf).await?;
    match buf[1] {
        0x5a => Ok(ProxyAsyncStream::TcpStream(stream)),
        0x5b => Err(Socks4ProxyError::RequestRejectedOrFailed(0x5b)),
        0x5c => Err(Socks4ProxyError::NoneAvailableIdentdService(0x5c)),
        0x5d => Err(Socks4ProxyError::IdentdCheckFailed(0x5d)),
        code => Err(Socks4ProxyError::UnknownReplyCode(code)),
    }
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

pub fn socks5_proxy(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<ProxyStream, Socks5ProxyError> {
    use std::net::ToSocketAddrs;

    if proxy.scheme_str().is_none() {
        return Err(Socks5ProxyError::NoScheme(proxy));
    }
    if proxy.host().is_none() {
        return Err(Socks5ProxyError::NoProxyServerHostName(proxy));
    }
    let proxy_host = proxy.host().unwrap();
    if proxy_host.is_empty() {
        return Err(Socks5ProxyError::EmptyProxyServerHostName(proxy));
    }
    if proxy.port_u16().is_none() {
        return Err(Socks5ProxyError::NoProxyServerPort(proxy));
    }
    let proxy_port = proxy.port_u16().unwrap();

    let mut stream = std::net::TcpStream::connect((proxy_host, proxy_port))?;

    // Client greeting: VER (1), NAUTH (1), AUTH (NAUTH)
    let mut bytes = vec![0x05];
    if username.is_some() && password.is_some() {
        bytes.extend([0x02, 0x00, 0x02]); // NAUTH, No authentication (0x00), Username/Password (0x02)
    } else {
        bytes.extend([0x01, 0x00]); // NAUTH, No authentication (0x00)
    }
    stream.write_all(&bytes)?;
    stream.flush()?;

    // Server choice: VER (1), CAUTH (1)
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf)?;
    if buf[0] != 0x05 {
        return Err(Socks5ProxyError::BadResponseVersion(buf[0]));
    }
    if buf[1] != 0x00 && buf[1] != 0x02 {
        return Err(Socks5ProxyError::BadServerChoice(buf[1]));
    }

    // Client authentication
    if buf[1] == 0x02 {
        let request = build_socks5_authentication_request(username.unwrap(), password.unwrap());
        stream.write_all(&request)?;
        stream.flush()?;

        // Server response: VER (1), STATUS (1)
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf)?;
        if buf[0] != 0x05 {
            return Err(Socks5ProxyError::BadResponseVersion(buf[0]));
        }
        if buf[1] != 0x00 {
            return Err(Socks5ProxyError::ClientAuthenticationFailed(buf));
        }
    }

    // Client connection
    let request = match proxy.scheme_str().unwrap().to_lowercase().as_str() {
        "socks5" => {
            let socket_addr = (target_host, 443).to_socket_addrs()?.next();
            if let Some(ip_addr) = socket_addr {
                build_socks5_connection_request(target_host, Some(ip_addr.ip()))
            } else {
                return Err(Socks5ProxyError::NoIpAddr(format!("{}:443", target_host)));
            }
        }
        "socks5h" => build_socks5_connection_request(target_host, None),
        _ => return Err(Socks5ProxyError::NotSupportedScheme(proxy)),
    };
    stream.write_all(&request)?;
    stream.flush()?;

    // Server response: VER (1), STATUS (1), RSV (1), BNDADDR [TYPE (1), ADDR (4/16)], BNDPORT (2)
    let mut buf = [0u8; 4]; // VER (1), STATUS (1), RSV (1), BNDADDR TYPE (1)
    stream.read_exact(&mut buf)?;
    match buf[1] {
        0x00 => match buf[3] {
            0x01 | 0x04 => {
                let (ip, port) = {
                    if buf[3] == 0x01 {
                        // BNDADDR ADDR (4), BNDPORT (2)
                        let mut buf = [0u8; 6];
                        stream.read_exact(&mut buf)?;
                        let ip = std::net::IpAddr::from([buf[0], buf[1], buf[2], buf[3]]);
                        let port = u16::from_be_bytes([buf[4], buf[5]]);
                        (ip, port)
                    } else {
                        // BNDADDR ADDR (16), BNDPORT (2)
                        let mut buf = [0u8; 18];
                        stream.read_exact(&mut buf)?;
                        let ip = std::net::IpAddr::from([
                            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8],
                            buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
                        ]);
                        let port = u16::from_be_bytes([buf[16], buf[17]]);
                        (ip, port)
                    }
                };

                let socket_addr = stream.peer_addr().unwrap();
                if socket_addr.ip() == ip && socket_addr.port() == port {
                    Ok(ProxyStream::TcpStream(stream))
                } else {
                    let stream = std::net::TcpStream::connect((ip, port))?;
                    Ok(ProxyStream::TcpStream(stream))
                }
            }
            addr_t => Err(Socks5ProxyError::NotSupportedServerBindAddressType(addr_t)),
        },
        0x01 => Err(Socks5ProxyError::GeneralFailure(0x01)),
        0x02 => Err(Socks5ProxyError::ConnectionNotAllowedByRules(0x02)),
        0x03 => Err(Socks5ProxyError::NetworkUnreachable(0x03)),
        0x04 => Err(Socks5ProxyError::HostUnreachable(0x04)),
        0x05 => Err(Socks5ProxyError::ConnectionRefused(0x05)),
        0x06 => Err(Socks5ProxyError::TtlExpired(0x06)),
        0x07 => Err(Socks5ProxyError::CommandNotSupported(0x07)),
        0x08 => Err(Socks5ProxyError::AddressTypeNotSupported(0x08)),
        code => Err(Socks5ProxyError::UnknownReplyCode(code)),
    }
}

pub async fn socks5_proxy_asnyc(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<ProxyAsyncStream, Socks5ProxyError> {
    use async_std::net::ToSocketAddrs;

    if proxy.scheme_str().is_none() {
        return Err(Socks5ProxyError::NoScheme(proxy));
    }
    if proxy.host().is_none() {
        return Err(Socks5ProxyError::NoProxyServerHostName(proxy));
    }
    let proxy_host = proxy.host().unwrap();
    if proxy_host.is_empty() {
        return Err(Socks5ProxyError::EmptyProxyServerHostName(proxy));
    }
    if proxy.port_u16().is_none() {
        return Err(Socks5ProxyError::NoProxyServerPort(proxy));
    }
    let proxy_port = proxy.port_u16().unwrap();

    let mut stream = async_std::net::TcpStream::connect((proxy_host, proxy_port)).await?;

    // Client greeting: VER (1), NAUTH (1), AUTH (NAUTH)
    let mut bytes = vec![0x05];
    if username.is_some() && password.is_some() {
        bytes.extend([0x02, 0x00, 0x02]); // NAUTH, No authentication (0x00), Username/Password (0x02)
    } else {
        bytes.extend([0x01, 0x00]); // NAUTH, No authentication (0x00)
    }
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    // Server choice: VER (1), CAUTH (1)
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf).await?;
    if buf[0] != 0x05 {
        return Err(Socks5ProxyError::BadResponseVersion(buf[0]));
    }
    if buf[1] != 0x00 && buf[1] != 0x02 {
        return Err(Socks5ProxyError::BadServerChoice(buf[1]));
    }

    // Client authentication
    if buf[1] == 0x02 {
        let request = build_socks5_authentication_request(username.unwrap(), password.unwrap());
        stream.write_all(&request).await?;
        stream.flush().await?;

        // Server response: VER (1), STATUS (1)
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf).await?;
        if buf[0] != 0x05 {
            return Err(Socks5ProxyError::BadResponseVersion(buf[0]));
        }
        if buf[1] != 0x00 {
            return Err(Socks5ProxyError::ClientAuthenticationFailed(buf));
        }
    }

    // Client connection
    let request = match proxy.scheme_str().unwrap().to_lowercase().as_str() {
        "socks5" => {
            let socket_addr = (target_host, 443).to_socket_addrs().await?.next();
            if let Some(ip_addr) = socket_addr {
                build_socks5_connection_request(target_host, Some(ip_addr.ip()))
            } else {
                return Err(Socks5ProxyError::NoIpAddr(format!("{}:443", target_host)));
            }
        }
        "socks5h" => build_socks5_connection_request(target_host, None),
        _ => return Err(Socks5ProxyError::NotSupportedScheme(proxy)),
    };
    stream.write_all(&request).await?;
    stream.flush().await?;

    // Server response: VER (1), STATUS (1), RSV (1), BNDADDR [TYPE (1), ADDR (4/16)], BNDPORT (2)
    let mut buf = [0u8; 4]; // VER (1), STATUS (1), RSV (1), BNDADDR TYPE (1)
    stream.read_exact(&mut buf).await?;
    match buf[1] {
        0x00 => match buf[3] {
            0x01 | 0x04 => {
                let (ip, port) = {
                    if buf[3] == 0x01 {
                        // BNDADDR ADDR (4), BNDPORT (2)
                        let mut buf = [0u8; 6];
                        stream.read_exact(&mut buf).await?;
                        let ip = std::net::IpAddr::from([buf[0], buf[1], buf[2], buf[3]]);
                        let port = u16::from_be_bytes([buf[4], buf[5]]);
                        (ip, port)
                    } else {
                        // BNDADDR ADDR (16), BNDPORT (2)
                        let mut buf = [0u8; 18];
                        stream.read_exact(&mut buf).await?;
                        let ip = std::net::IpAddr::from([
                            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8],
                            buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
                        ]);
                        let port = u16::from_be_bytes([buf[16], buf[17]]);
                        (ip, port)
                    }
                };

                let socket_addr = stream.peer_addr().unwrap();
                if socket_addr.ip() == ip && socket_addr.port() == port {
                    Ok(ProxyAsyncStream::TcpStream(stream))
                } else {
                    let stream = async_std::net::TcpStream::connect((ip, port)).await?;
                    Ok(ProxyAsyncStream::TcpStream(stream))
                }
            }
            addr_t => Err(Socks5ProxyError::NotSupportedServerBindAddressType(addr_t)),
        },
        0x01 => Err(Socks5ProxyError::GeneralFailure(0x01)),
        0x02 => Err(Socks5ProxyError::ConnectionNotAllowedByRules(0x02)),
        0x03 => Err(Socks5ProxyError::NetworkUnreachable(0x03)),
        0x04 => Err(Socks5ProxyError::HostUnreachable(0x04)),
        0x05 => Err(Socks5ProxyError::ConnectionRefused(0x05)),
        0x06 => Err(Socks5ProxyError::TtlExpired(0x06)),
        0x07 => Err(Socks5ProxyError::CommandNotSupported(0x07)),
        0x08 => Err(Socks5ProxyError::AddressTypeNotSupported(0x08)),
        code => Err(Socks5ProxyError::UnknownReplyCode(code)),
    }
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
