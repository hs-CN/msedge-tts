use crate::{error::Result, tts::build_websocket_request};

pub async fn websocket_connect_async() -> Result<
    async_tungstenite::WebSocketStream<async_tungstenite::smol::ClientStream<smol::net::TcpStream>>,
> {
    let request = build_websocket_request()?;
    let (websocket, _) = async_tungstenite::smol::connect_async(request).await?;
    Ok(websocket)
}

#[cfg(feature = "proxy")]
use crate::tts::proxy::{
    ProxyError,
    smol_runtime::{ProxyAsyncStream, http_proxy_async, socks4_proxy_async, socks5_proxy_asnyc},
};

#[cfg(feature = "proxy")]
pub async fn websocket_connect_proxy_async(
    proxy: http::Uri,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<
    async_tungstenite::WebSocketStream<async_tungstenite::smol::ClientStream<ProxyAsyncStream>>,
> {
    let request = build_websocket_request()?;
    let stream: std::result::Result<ProxyAsyncStream, ProxyError> = match proxy.scheme_str() {
        Some(scheme) => match scheme.to_lowercase().as_str() {
            "socks4" | "socks4a" => {
                socks4_proxy_async(request.uri().host().unwrap(), proxy, username)
                    .await
                    .map_err(|e| e.into())
            }
            "socks5" | "socks5h" => {
                socks5_proxy_asnyc(request.uri().host().unwrap(), proxy, username, password)
                    .await
                    .map_err(|e| e.into())
            }
            "http" | "https" => {
                http_proxy_async(request.uri().host().unwrap(), proxy, username, password)
                    .await
                    .map_err(|e| e.into())
            }
            _ => Err(ProxyError::NotSupportedScheme(proxy)),
        },
        None => http_proxy_async(request.uri().host().unwrap(), proxy, username, password)
            .await
            .map_err(|e| e.into()),
    };
    let (websocket, _) = async_tungstenite::smol::client_async_tls(request, stream?).await?;
    Ok(websocket)
}
