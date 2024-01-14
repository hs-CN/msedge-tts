use super::build_websocket_request;

pub type WebSocketStream =
    tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

pub fn websocket_connect() -> anyhow::Result<WebSocketStream> {
    let request = build_websocket_request()?;
    let (websocket, _) = tungstenite::connect(request)?;
    Ok(websocket)
}

pub type WebSocketStreamAsync =
    async_tungstenite::WebSocketStream<async_tungstenite::async_std::ConnectStream>;

pub async fn websocket_connect_asnyc() -> anyhow::Result<WebSocketStreamAsync> {
    let request = build_websocket_request()?;
    let (websocket, _) = async_tungstenite::async_std::connect_async(request).await?;
    Ok(websocket)
}
