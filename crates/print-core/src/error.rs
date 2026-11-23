use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrintError {
    #[error("Empty print payload")]
    EmptyPayload,

    #[error("Invalid base64 payload: {0}")]
    InvalidBase64(String),

    #[error("Invalid gzip payload: {0}")]
    InvalidGzip(String),

    #[error("Invalid deeplink URL: {0}")]
    InvalidDeeplink(String),

    #[error("Disallowed callback URL: {0}")]
    DisallowedCallback(String),

    #[error("Invalid printer target: {0}")]
    InvalidTarget(String),

    #[error("Transport I/O error: {0}")]
    Transport(String),
}

pub type Result<T> = std::result::Result<T, PrintError>;
