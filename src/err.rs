use std::{io, fmt};
use serde::de::value;
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
    Parser(#[from] value::Error),
    /// Encode errors
    #[error("Encode errors")]
    Format(#[from] fmt::Error),
    /// Key does not exist
    #[error("Key not found")]
    KeyNotFound,
    /// Unexpected command
    #[error("Unexpected command")]
    UnexpectedCommand,
}