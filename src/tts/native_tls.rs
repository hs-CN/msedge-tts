use crate::{
    error::{HttpProxyError, ProxyError, Result},
    tts::{
        build_websocket_request,
        proxy::{build_http_proxy, build_http_proxy_async, ProxyAsyncStream, ProxyStream},
    },
};

pub type WebSocketStream<T> = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<T>>;

pub fn websocket_connect() -> Result<WebSocketStream<std::net::TcpStream>> {
    let request = build_websocket_request()?;
    let (websocket, _) = tungstenite::connect(request)?;
    Ok(websocket)
}

pub fn websocket_connect_proxy(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<WebSocketStream<ProxyStream>> {
    use tungstenite::handshake::HandshakeError;

    let request = build_websocket_request()?;
    let stream: std::result::Result<ProxyStream, ProxyError> = match proxy.scheme_str() {
        Some(scheme) => match scheme {
            "socks4" | "socks4a" | "socks5" | "socks5h" => todo!(),
            "http" | "https" => {
                build_http_proxy(request.uri().host().unwrap(), proxy, username, password)
                    .map_err(|e| e.into())
            }
            _ => Err(HttpProxyError::NotSupportedScheme(proxy).into()),
        },
        None => build_http_proxy(request.uri().host().unwrap(), proxy, username, password)
            .map_err(|e| e.into()),
    };
    let (websocket, _) = tungstenite::client_tls(request, stream?).map_err(|e| match e {
        HandshakeError::Failure(e) => e,
        HandshakeError::Interrupted(_) => panic!("Bug: blocking handshake not blocked"),
    })?;
    Ok(websocket)
}

pub type WebSocketStreamAsync<T> =
    async_tungstenite::WebSocketStream<async_tungstenite::async_std::ClientStream<T>>;

pub async fn websocket_connect_async() -> Result<WebSocketStreamAsync<async_std::net::TcpStream>> {
    let request = build_websocket_request()?;
    let (websocket, _) = async_tungstenite::async_std::connect_async(request).await?;
    Ok(websocket)
}

pub async fn websocket_connect_proxy_async(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<WebSocketStreamAsync<ProxyAsyncStream>> {
    let request = build_websocket_request()?;
    let stream: std::result::Result<ProxyAsyncStream, ProxyError> = match proxy.scheme_str() {
        Some(scheme) => match scheme {
            "socks4" | "socks4a" | "socks5" | "socks5h" => todo!(),
            "http" | "https" => {
                build_http_proxy_async(request.uri().host().unwrap(), proxy, username, password)
                    .await
                    .map_err(|e| e.into())
            }
            _ => Err(HttpProxyError::NotSupportedScheme(proxy).into()),
        },
        None => build_http_proxy_async(request.uri().host().unwrap(), proxy, username, password)
            .await
            .map_err(|e| e.into()),
    };
    let (websocket, _) = async_tungstenite::async_std::client_async_tls(request, stream?).await?;
    Ok(websocket)
}
