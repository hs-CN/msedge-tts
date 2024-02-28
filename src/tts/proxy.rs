use crate::error::{HttpProxyError, Result};
use base64::Engine;
use std::io::{Read, Write};

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

pub fn build_http_proxy(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> std::result::Result<ProxyStream, HttpProxyError> {
    if proxy.host().is_none() {
        return Err(HttpProxyError::NoHostName(proxy));
    }
    let proxy_host = proxy.host().unwrap();
    if proxy_host.is_empty() {
        return Err(HttpProxyError::EmptyHostName(proxy));
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
        Some("http") | None | _ => {
            let stream = std::net::TcpStream::connect((proxy_host, proxy_port))?;
            ProxyStream::TcpStream(stream)
        }
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

pub async fn build_http_proxy_async(
    target_host: &str,
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<async_std::net::TcpStream> {
    todo!()
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
