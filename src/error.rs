use thiserror::Error;

#[derive(Error, Debug)]
pub enum GunError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Storage error: {0}")]
    Storage(#[from] sled::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid soul: {0}")]
    InvalidSoul(String),

    #[error("Node not found")]
    NodeNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}

pub type GunResult<T> = Result<T, GunError>;
