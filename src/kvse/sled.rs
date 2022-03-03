use std::path::PathBuf;

use crate::{KvsEngine, KvsError, Result};

pub struct SledKvsEngine {
    db: sled::Db,
}

#[allow(dead_code)]
impl SledKvsEngine {
    pub fn open(path: impl Into<PathBuf>) -> Result<SledKvsEngine> {
        Ok(SledKvsEngine {
            db: sled::open(path.into())?,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self
            .db
            .get(&key)?
            .map(|val| String::from_utf8_lossy(val.as_ref()).to_string()))
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(&key, &*value)?;
        self.db.flush()?;
        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        self.db.remove(key)?.ok_or(KvsError::KeyNotFound)?;
        self.db.flush()?;
        Ok(())
    }
}
