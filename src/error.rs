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
}

#[derive(Error, Debug)]
pub enum HttpProxyError {
    #[error("proxy url no host name: {0}")]
    NoHostName(http::Uri),
    #[error("proxy url empty host name: {0}")]
    EmptyHostName(http::Uri),
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
