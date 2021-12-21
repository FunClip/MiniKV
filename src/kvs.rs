use std::collections::HashMap;

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
/// assert_eq!(val, Some(String::from("key")));
/// ```
pub struct KvStore {
    map: HashMap<String, String>
}

impl KvStore {
    /// Create a `KvStore`
    pub fn new() -> KvStore{
        KvStore {
            map: HashMap::new()
        }
    }

    /// Set the value of a string key to a string.
    /// 
    /// If the key exists, the value will be overwritten.
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// Get the string value of a given string key
    /// 
    /// Return `None` if the key doesn't exist.
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}