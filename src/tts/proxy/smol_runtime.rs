use std::pin::pin;

use futures_rustls::client::TlsStream;
use rustls_platform_verifier::ConfigVerifierExt;
use smol::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpStream, resolve},
};

use crate::tts::{
    TlsError,
    proxy::{
        HttpProxyError, Socks4ProxyError, Socks5ProxyError, build_http_proxy_request,
        build_socks4_connection_request, build_socks5_authentication_request,
        build_socks5_connection_request,
    },
};

pub enum ProxyAsyncStream {
    TcpStream(TcpStream),
    TlsStream(TlsStream<TcpStream>),
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

pub async fn socks4_proxy_async(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
) -> Result<ProxyAsyncStream, Socks4ProxyError> {
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

    let mut stream = TcpStream::connect((proxy_host, proxy_port)).await?;
    let request = match proxy.scheme_str().unwrap().to_lowercase().as_str() {
        "socks4" => {
            let socket_addrs = resolve((proxy_host, proxy_port)).await?;
            let ipv4 = socket_addrs.into_iter().find_map(|addr| match addr {
                std::net::SocketAddr::V4(addr) => Some(addr.ip().to_owned()),
                std::net::SocketAddr::V6(_) => None,
            });

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

pub async fn socks5_proxy_asnyc(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<ProxyAsyncStream, Socks5ProxyError> {
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

    let mut stream = TcpStream::connect((proxy_host, proxy_port)).await?;

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
        if buf[1] != 0x00 {
            return Err(Socks5ProxyError::ClientAuthenticationFailed(buf));
        }
    }

    // Client connection
    let request = match proxy.scheme_str().unwrap().to_lowercase().as_str() {
        "socks5" => {
            let socket_addrs = smol::net::resolve((proxy_host, proxy_port)).await?;
            if let Some(ip_addr) = socket_addrs.first() {
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
                    let stream = TcpStream::connect((ip, port)).await?;
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

pub async fn http_proxy_async(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<ProxyAsyncStream, HttpProxyError> {
    if let Some(proxy_host) = proxy.host() {
        if proxy_host.is_empty() {
            return Err(HttpProxyError::EmptyProxyServerHostName(proxy));
        }
    } else {
        return Err(HttpProxyError::NoProxyServerHostName(proxy));
    };
    let proxy_host = proxy.host().unwrap().to_owned();

    let proxy_port = proxy.port_u16().unwrap_or(match proxy.scheme_str() {
        None => 80,
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "https" => 443,
            "http" | _ => 80,
        },
    });

    let mut stream = match proxy.scheme_str() {
        None => {
            let stream = TcpStream::connect((proxy_host, proxy_port)).await?;
            ProxyAsyncStream::TcpStream(stream)
        }
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "http" => {
                let stream = TcpStream::connect((proxy_host, proxy_port)).await?;
                ProxyAsyncStream::TcpStream(stream)
            }
            "https" => {
                let stream = TcpStream::connect((proxy_host.as_str(), proxy_port)).await?;
                let config = futures_rustls::rustls::ClientConfig::with_platform_verifier()
                    .map_err(|e| TlsError::TlsError(e))?;
                let name = rustls::pki_types::ServerName::try_from(proxy_host)
                    .map_err(|e| TlsError::InvalidDnsName(e))?;
                let connector = futures_rustls::TlsConnector::from(std::sync::Arc::new(config));

                let stream = connector.connect(name, stream).await?;
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
