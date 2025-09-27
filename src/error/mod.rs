use thiserror::Error;

#[derive(Debug, Error)]
pub enum NagadError {
    #[error("configuration should be a map")]
    InvalidConfiguration,
    #[error("params should be a map")]
    InvalidParams,
    #[error("missing parameter: {0}")]
    MissingParam(&'static str),
    #[error("invalid public key: {0}")]
    PublicKey(String),
    #[error("invalid private key: {0}")]
    PrivateKey(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("json error: {0}")]
    Json(String),
    #[error("url parse error: {0}")]
    UrlParse(String),
    #[error("verification error: {0}")]
    Verification(String),
    #[error("chrono error: {0}")]
    Chrono(String),
}

pub type Result<T> = std::result::Result<T, NagadError>;
