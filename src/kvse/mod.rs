use crate::Result;

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;

/// Trait for a key value storage engine
pub trait KvsEngine {
    /// Get the value of a given key
    /// Return `None` if the key does not exist
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// Set the value of a given key
    /// If the key exists, the value will be overwritten
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Remove the value of a given key
    /// Return `KvsError::KeyNotFound` if the key does not exist
    fn remove(&mut self, key: String) -> Result<()>;
}
