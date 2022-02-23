use serde_json::error;
use std::{fmt, io};
use thiserror::Error;

/// Alias for a `Result` with the error type `kvs::KvsError`.
pub type Result<T> = std::result::Result<T, KvsError>;

/// This type represnts all possible errors that can occur in `kvs`.
#[derive(Debug, Error)]
pub enum KvsError {
    /// Io errors
    #[error("Io errors")]
    Io(#[from] io::Error),
    /// Decode errors
    #[error("Decode errors")]
    Parser(#[from] error::Error),
    /// Encode errors
    #[error("Encode errors")]
    Format(#[from] fmt::Error),
    /// Logger initial errors
    #[error("Logger initial errors")]
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
