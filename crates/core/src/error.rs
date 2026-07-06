use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("camera error: {0}")]
    Camera(String),

    #[error("detection error: {0}")]
    Detection(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("modem error: {0}")]
    Modem(String),

    #[error("privileged helper error: {0}")]
    Privileged(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("authentication required")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
