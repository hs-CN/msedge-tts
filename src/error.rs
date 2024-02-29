use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected message: {0}")]
    UnexpectedMessage(String),
    #[error("isahc error: {0}")]
    IsahcError(#[from] isahc::Error),
    #[error("tungstenite error: {0}")]
    TungsteniteError(#[from] tungstenite::Error),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("proxy error: {0}")]
    ProxyError(#[from] ProxyError),
}

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("http proxy error: {0}")]
    HttpProxyError(#[from] HttpProxyError),
    #[error("socks4 proxy error: {0}")]
    Socks4ProxyError(#[from] Socks4ProxyError),
}

#[derive(Error, Debug)]
pub enum HttpProxyError {
    #[error("no proxy server host name: {0}")]
    NoProxyServerHostName(http::Uri),
    #[error("proxy host name is empty: {0}")]
    EmptyProxyServerHostName(http::Uri),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("native tls error: {0}")]
    NativeTlsError(#[from] native_tls::Error),
    #[error("invalid response: {0}")]
    InvalidResponse(#[from] httparse::Error),
    #[error("bad response: {0} {1}")]
    BadResponse(u16, String),
    #[error("no status code")]
    NoStatusCode,
    #[error("not supported scheme: {0}")]
    NotSupportedScheme(http::Uri),
}

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
    #[error("lookup socket addr v4 failed: {0}")]
    NoSocketAddrV4(String),
    #[error("request rejected or failed")]
    RequestRejectedOrFailed(u8),
    #[error("no available identd service")]
    NoneAvailableIdentdService(u8),
    #[error("identd check failed: {0}")]
    IdentdCheckFailed(u8),
    #[error("unknown reply code: {0}")]
    UnknownReplyCode(u8),
}
