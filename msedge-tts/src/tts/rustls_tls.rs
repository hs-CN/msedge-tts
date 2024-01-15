use super::build_websocket_request;
use crate::error::Result;

pub type WebSocketStream =
    tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

pub fn websocket_connect() -> Result<WebSocketStream> {
    use tungstenite::handshake::HandshakeError;
    use tungstenite::Error;

    let request = build_websocket_request()?;
    let stream = try_connect(&request)?;
    let connector = build_rustls_connector()?;
    let (websocket, _) =
        tungstenite::client_tls_with_config(request, stream, None, Some(connector)).map_err(
            |e| match e {
                HandshakeError::Failure(e) => e,
                e => Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )),
            },
        )?;
    Ok(websocket)
}

fn try_connect(request: &tungstenite::handshake::client::Request) -> Result<std::net::TcpStream> {
    use tungstenite::error::UrlError;
    use tungstenite::Error;

    let uri = request.uri();
    let host = uri.host().ok_or(Error::Url(UrlError::NoHostName))?;
    let stream =
        std::net::TcpStream::connect((host, 443)).map_err(|err| tungstenite::Error::from(err))?;
    stream
        .set_nodelay(true)
        .map_err(|err| tungstenite::Error::from(err))?;
    Ok(stream)
}

fn build_rustls_connector() -> Result<tungstenite::Connector> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_parsable_certificates(
        rustls_native_certs::load_native_certs().map_err(|err| tungstenite::Error::from(err))?,
    );
    let mut client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = std::sync::Arc::new(rustls::KeyLogFile::new());
    Ok(tungstenite::Connector::Rustls(std::sync::Arc::new(
        client_config,
    )))
}

pub type WebSocketStreamAsync = async_tungstenite::WebSocketStream<
    async_tungstenite::async_tls::ClientStream<async_net::TcpStream>,
>;

pub async fn websocket_connect_asnyc() -> Result<WebSocketStreamAsync> {
    use async_tungstenite::async_tls::client_async_tls_with_connector;

    let request = build_websocket_request()?;
    let stream = try_connect_async(&request).await?;
    let connector = build_async_tls_connector()?;
    let (websocket, _) = client_async_tls_with_connector(request, stream, Some(connector)).await?;
    Ok(websocket)
}

async fn try_connect_async(
    request: &tungstenite::handshake::client::Request,
) -> Result<async_net::TcpStream> {
    use tungstenite::error::UrlError;
    use tungstenite::Error;

    let uri = request.uri();
    let host = uri.host().ok_or(Error::Url(UrlError::NoHostName))?;
    let stream = async_net::TcpStream::connect((host, 443))
        .await
        .map_err(|err| tungstenite::Error::from(err))?;
    Ok(stream)
}

fn build_async_tls_connector() -> Result<async_tls::TlsConnector> {
    let certs: Vec<_> = rustls_native_certs::load_native_certs()
        .map_err(|err| tungstenite::Error::from(err))?
        .into_iter()
        .map(|x| x.to_vec())
        .collect();
    let mut root_store = old_rustls::RootCertStore::empty();
    root_store.add_parsable_certificates(&certs);
    let mut client_config = old_rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = std::sync::Arc::new(old_rustls::KeyLogFile::new());
    Ok(async_tls::TlsConnector::from(client_config))
}
