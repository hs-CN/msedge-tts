use thiserror::Error;

use crate::{error::Result, tts::build_websocket_request};

/// Tls Error
#[derive(Error, Debug)]
pub enum TlsError {
    #[error("invalid DNS name: {0}")]
    InvalidDnsName(#[from] rustls::pki_types::InvalidDnsNameError),
    #[error("rustls error: {0}")]
    TlsError(#[from] rustls::Error),
}

pub fn rustls_stream<T: std::io::Read + std::io::Write>(
    stream: T,
    host: String,
) -> std::result::Result<RustlsStream<T>, TlsError> {
    use rustls::pki_types::ServerName;
    use rustls::{ClientConfig, ClientConnection, StreamOwned};
    use rustls_platform_verifier::ConfigVerifierExt;
    use std::sync::Arc;

    let config = ClientConfig::with_platform_verifier()?;
    let name = ServerName::try_from(host)?;
    let client = ClientConnection::new(Arc::new(config), name)?;
    Ok(StreamOwned::new(client, stream))
}

pub type RustlsStream<T> = rustls::StreamOwned<rustls::ClientConnection, T>;
pub type WebSocket<T> = tungstenite::WebSocket<RustlsStream<T>>;

pub fn websocket_connect() -> Result<WebSocket<std::net::TcpStream>> {
    use tungstenite::{ClientHandshake, Error, HandshakeError, error::*};

    let request = build_websocket_request()?;
    let host = request
        .uri()
        .host()
        .ok_or(Error::Url(UrlError::NoHostName))?
        .to_owned();

    let stream = std::net::TcpStream::connect((host.as_str(), 443)).map_err(|e| Error::Io(e))?;
    stream.set_nodelay(true).map_err(|e| Error::Io(e))?;

    let stream = rustls_stream(stream, host)?;
    let (websocket, _) = ClientHandshake::start(stream, request, None)?
        .handshake()
        .map_err(|e| match e {
            HandshakeError::Failure(e) => e,
            HandshakeError::Interrupted(_) => {
                panic!("Bug: blocking handshake not blocked")
            }
        })?;
    Ok(websocket)
}

#[cfg(feature = "proxy")]
use crate::tts::proxy::{
    ProxyError,
    blocking::{ProxyStream, http_proxy, socks4_proxy, socks5_proxy},
};

#[cfg(feature = "proxy")]
pub fn websocket_connect_proxy(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<WebSocket<ProxyStream>> {
    use tungstenite::ClientHandshake;
    use tungstenite::error::*;
    use tungstenite::handshake::HandshakeError;

    let request = build_websocket_request()?;
    let target_host = request
        .uri()
        .host()
        .ok_or(Error::Url(UrlError::NoHostName))?
        .to_owned();
    let stream: std::result::Result<ProxyStream, ProxyError> = match proxy.scheme_str() {
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "socks4" | "socks4a" => {
                socks4_proxy(target_host.as_str(), proxy, username).map_err(|e| e.into())
            }
            "socks5" | "socks5h" => {
                socks5_proxy(target_host.as_str(), proxy, username, password).map_err(|e| e.into())
            }
            "http" | "https" => {
                http_proxy(target_host.as_str(), proxy, username, password).map_err(|e| e.into())
            }
            _ => Err(ProxyError::NotSupportedScheme(proxy)),
        },
        None => http_proxy(target_host.as_str(), proxy, username, password).map_err(|e| e.into()),
    };
    let stream = rustls_stream(stream?, target_host)?;
    let (websocket, _) = ClientHandshake::start(stream, request, None)?
        .handshake()
        .map_err(|e| match e {
            HandshakeError::Failure(e) => e,
            HandshakeError::Interrupted(_) => {
                panic!("Bug: blocking handshake not blocked")
            }
        })?;
    Ok(websocket)
}
