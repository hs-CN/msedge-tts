use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unexpected message: {0}")]
    UnexpectedMessage(String),
    #[error("isahc error: {0}")]
    IsahcError(isahc::Error),
    #[error("tungstenite error: {0}")]
    TungsteniteError(tungstenite::Error),
    #[error("serde json error: {0}")]
    SerdeJsonError(serde_json::Error),
}

impl From<isahc::Error> for Error {
    fn from(error: isahc::Error) -> Self {
        Self::IsahcError(error)
    }
}

impl From<tungstenite::Error> for Error {
    fn from(error: tungstenite::Error) -> Self {
        Self::TungsteniteError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeJsonError(error)
    }
}
