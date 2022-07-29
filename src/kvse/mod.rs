use std::path::PathBuf;

use crate::Result;

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;

/// Trait for a key value storage engine
pub trait KvsEngine: Clone + Send + 'static {
    /// Get the value of a given key
    /// Return `None` if the key does not exist
    fn get(&self, key: String) -> Result<Option<String>>;

    /// Set the value of a given key
    /// If the key exists, the value will be overwritten
    fn set(&self, key: String, value: String) -> Result<()>;

    /// Remove the value of a given key
    /// Return `KvsError::KeyNotFound` if the key does not exist
    fn remove(&self, key: String) -> Result<()>;

    /// Open or create a store engine from given path
    /// Return a `KvsEngine` with `Result` wrapper
    fn open(path: impl Into<PathBuf>) -> Result<Self>;
}
