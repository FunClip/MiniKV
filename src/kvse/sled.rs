use std::path::PathBuf;

use crate::{KvsEngine, KvsError, Result};

/// SledKvsEngine by `sled::Db`
#[derive(Clone)]
pub struct SledKvsEngine {
    db: sled::Db,
}

#[allow(dead_code)]
impl SledKvsEngine {
    /// Create a SledKvsEngine by `sled::open`
    pub fn open(path: impl Into<PathBuf>) -> Result<SledKvsEngine> {
        Ok(SledKvsEngine {
            db: sled::open(path.into())?,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self
            .db
            .get(&key)?
            .map(|val| String::from_utf8_lossy(val.as_ref()).to_string()))
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        self.db.insert(&key, &*value)?;
        self.db.flush()?;
        Ok(())
    }

    fn remove(&self, key: String) -> Result<()> {
        self.db.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        self.db.flush()?;
        Ok(())
    }

    fn open(path: impl Into<PathBuf>) -> Result<Self> {
        SledKvsEngine::open(path)
    }
}
