//! Error Type

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "blocking")]
    #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
    #[error("ureq error: {0}")]
    UreqError(#[from] ureq::Error),

    #[cfg(any(feature = "smol-runtime", feature = "tokio-runtime"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "smol-runtime", feature = "tokio-runtime")))
    )]
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[cfg(feature = "blocking")]
    #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
    #[error("tls error: {0}")]
    TlsError(#[from] crate::tts::blocking::TlsError),

    #[cfg(feature = "proxy")]
    #[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
    #[error("proxy error: {0}")]
    ProxyError(#[from] crate::tts::proxy::ProxyError),

    #[error("tungstenite error: {0}")]
    TungsteniteError(#[from] tungstenite::Error),
    #[error("serde json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("unexpected message: {0}")]
    UnexpectedMessage(String),
}
