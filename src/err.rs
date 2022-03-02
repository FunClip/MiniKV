use serde::{de, ser};
use serde_json::error;
use std::io;
use thiserror::Error;

/// Alias for a `Result` with the error type `kvs::KvsError`.
pub type Result<T> = std::result::Result<T, KvsError>;

/// This type represnts all possible errors that can occur in `kvs`.
#[derive(Debug, Error)]
pub enum KvsError {
    /// Io errors
    #[error("Io errors: {0:?}")]
    Io(#[from] io::Error),
    /// json ser/de errors
    #[error("JSON ser/de errors: {0:?}")]
    Json(#[from] error::Error),
    /// Command serialize error
    #[error("Command serialize error: {0}")]
    Serialize(String),
    /// Command deserialize error
    #[error("Command deserialize error: {0}")]
    Deserialize(String),
    /// Server error
    #[error("Commands execute on server failed: {0}")]
    Server(String),
    /// Logger initial errors
    #[error("Logger initial errors: {0:?}")]
    LoggerError(#[from] sloggers::Error),
    /// Key does not exist
    #[error("Key not found")]
    KeyNotFound,
    /// Invalid engine
    #[error("Invalid engine")]
    InValidEngine,
    /// Unexpected command
    #[error("Unexpected command")]
    UnexpectedCommand,
}

impl ser::Error for KvsError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Serialize(msg.to_string())
    }
}

impl de::Error for KvsError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Deserialize(msg.to_string())
    }
}
