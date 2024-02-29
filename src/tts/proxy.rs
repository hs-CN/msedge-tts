use crate::error::{HttpProxyError, Socks4ProxyError};
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
        Some("https") => 443,
        Some("http") | None | _ => 80,
    });

    let mut stream = match proxy.scheme_str() {
        Some("https") => {
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
        Some("http") | None => {
            let stream = std::net::TcpStream::connect((proxy_host, proxy_port))?;
            ProxyStream::TcpStream(stream)
        }
        Some(_) => Err(HttpProxyError::NotSupportedScheme(proxy))?,
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
        Some("https") => 443,
        Some("http") | None | _ => 80,
    });

    let mut stream = match proxy.scheme_str() {
        Some("https") => {
            let stream = async_std::net::TcpStream::connect((proxy_host, proxy_port)).await?;
            let stream = async_native_tls::connect(proxy_host, stream).await?;
            ProxyAsyncStream::TlsStream(stream)
        }
        Some("http") | None => {
            let stream = async_std::net::TcpStream::connect((proxy_host, proxy_port)).await?;
            ProxyAsyncStream::TcpStream(stream)
        }
        Some(_) => Err(HttpProxyError::NotSupportedScheme(proxy))?,
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
    let packets = match proxy.scheme_str() {
        Some("socks4") => {
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
                return Err(Socks4ProxyError::NoSocketAddrV4(format!(
                    "{}:443",
                    target_host
                )));
            }
            build_socks4_packets(target_host, ipv4, username)
        }
        Some("socks4a") => build_socks4_packets(target_host, None, username),
        Some(_) => Err(Socks4ProxyError::NotSupportedScheme(proxy))?,
        None => Err(Socks4ProxyError::NoScheme(proxy))?,
    };
    stream.write_all(&packets)?;
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
    let packets = match proxy.scheme_str() {
        Some("socks4") => {
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
                return Err(Socks4ProxyError::NoSocketAddrV4(format!(
                    "{}:443",
                    target_host
                )));
            }
            build_socks4_packets(target_host, ipv4, username)
        }
        Some("socks4a") => build_socks4_packets(target_host, None, username),
        Some(_) => Err(Socks4ProxyError::NotSupportedScheme(proxy))?,
        None => Err(Socks4ProxyError::NoScheme(proxy))?,
    };
    stream.write_all(&packets).await?;
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

fn build_socks4_packets(
    target_host: &str,
    dst_ip: Option<std::net::Ipv4Addr>,
    username: Option<&str>,
) -> Vec<u8> {
    // VER (1), CMD (1), DSTPORT (2)
    let mut bytes = vec![0x04, 0x01, 0x01, 0xbb];

    // DSTIP (4)
    if let Some(ip) = dst_ip {
        // socks4
        bytes.extend(ip.octets());
    } else {
        // socks4a
        bytes.extend([0x00, 0x00, 0x00, 0x01]);
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
