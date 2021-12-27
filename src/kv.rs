use std::fs::{File, self};
use std::io::{BufWriter, Write, Seek, Read, BufReader, SeekFrom};
use std::{collections::HashMap, path::PathBuf};
use serde::{Serialize, Deserialize};

use crate::Result;
use crate::err::KvsError;

const COMPACTION_THRESHOLD: u64 = 1024;

/// The `KvStore` stores key-value pairs
///
/// Key-value pairs are stored in memory by `HashMap` and not persisted in disk.
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set(String::from("key"), String::from("value"));
/// let val = store.get(String::from("key"));
/// assert_eq!(val, Some(String::from("value")));
/// ```
pub struct KvStore {
    writer: BufWriter<File>,
    readers: Vec<BufReader<File>>,
    current: usize,
    map: HashMap<String, String>,
    uncompacted: u64,
}

impl KvStore {

    /// Set the value of a string key to a string.
    ///
    /// If the key exists, the value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.map.insert(key, value);
        Ok(())
    }

    /// Get the string value of a given string key
    ///
    /// Return `None` if the key doesn't exist.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.map.get(&key) {
            Some(value) => Ok(Some(value.clone())),
            None => Err(KvsError::KeyNotFound),
        }
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.map.remove(&key) {
            Some(_) => Ok(()),
            None => Err(KvsError::KeyNotFound),
        }
    }

    /// Open a `KvStore` with the given path.
    /// 
    /// If the given path doesn't exist, it will create one.
    /// 
    /// # Errors
    /// 
    /// 
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        fs::create_dir_all(path.into())?;

        todo!()
    }
}


/// Command log object for store
#[derive(Debug, Serialize, Deserialize)]
enum Command {
    /// Set the value of a string key to a string
    Set {
        /// A string key
        key: String,
        /// A string value of the key
        value: String,
    },
    /// Remove a given key
    Rm {
        /// A string key
        key: String,
    },
}
