use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Seek, BufRead, SeekFrom, Write};
use std::path::Path;
use std::path::PathBuf;

use crate::err::KvsError;
use crate::Result;

const COMPACTION_THRESHOLD: u64 = 1024;
const BLOCK_THRESHOLD: u64 = 1024;

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
    current: u64,
    index: BTreeMap<String, Postion>,
    uncompacted: u64,
}

impl KvStore {
    /// Set the value of a string key to a string.
    ///
    /// If the key exists, the value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let mut cmd = serde_json::to_string(&Command::Set{key: key.clone(), value})?;
        cmd += "\n";
        let position = self.writer.stream_position()?;
        let size = self.writer.write(cmd.as_bytes())?;
        let file = self.current - 1;
        self.writer.flush()?;

        if let Some(pos) = self.index.insert(key, Postion{
            file,
            position,
            size: size as u64
        }) {
            self.uncompacted += pos.size;
        }

        Ok(())
    }

    /// Get the string value of a given string key
    ///
    /// Return `None` if the key doesn't exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.index.get(&key) {
            Some(pos) => {
                let mut buf = String::new();
                let reader = self.readers.get_mut(pos.file as usize).unwrap();
                reader.seek(SeekFrom::Start(pos.position))?;
                reader.read_line(&mut buf)?;
                match serde_json::from_str(&buf).unwrap() {
                    Command::Set{key: _, value} => Ok(Some(value)),
                    Command::Rm{key: _} => unreachable!()
                }
            },
            None => Ok(None),
        }
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if let None = self.index.get(&key) {
            return Err(KvsError::KeyNotFound)
        }
        
        let mut cmd = serde_json::to_string(&Command::Rm{key: key.clone()})?;
        cmd += "\n";
        self.writer.stream_position()?;
        let size = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;

        if let Some(pos) = self.index.remove(&key) {
            self.uncompacted += pos.size + size as u64;
        }

        Ok(())
    }

    /// Open a `KvStore` with the given path.
    ///
    /// If the given path doesn't exist, it will create one.
    ///
    /// # Errors
    ///
    ///
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut readers: Vec<BufReader<File>> = Vec::new();
        let mut current = 0u64;
        let mut uncompacted = 0u64;
        let mut index: BTreeMap<String, Postion> = BTreeMap::new();

        let path = path.into();
        fs::create_dir_all(&path)?;
        let files = get_log_files(&path)?;

        // create BufReader from files
        for file in &files {
            current += 1;
            readers.push(BufReader::new(
                File::open(file)?
            ))
        }

        // build index from readers
        uncompacted += load_index_from_readers(&mut readers, &mut index)?;

        // create BufWriter
        let mut writer = if current == 0 || readers[(current - 1) as usize].stream_position()? > BLOCK_THRESHOLD {
            current += 1;
            let f_w = File::create(&path.join(format!("{}.log", current)))?;
            let f_r = File::open(&path.join(format!("{}.log", current)))?;
            readers.push(BufReader::new(f_r));
            BufWriter::new(f_w)
        }
        else {
            BufWriter::new(
                File::options().write(true).open(&files[(current - 1)as usize])?
            )
        };

        writer.seek(SeekFrom::End(0))?;

        Ok(KvStore{
            writer,
            readers,
            current,
            index,
            uncompacted,
        })
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

/// Log entry's postion in files
struct Postion {
    file: u64,
    position: u64,
    size: u64,
}

fn get_log_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut log_files = fs::read_dir(path)?
        .filter(|p| p.is_ok())
        .map(|res| res.unwrap().path())
        .filter(|p| p.is_file() && p.extension() == Some("log".as_ref()) && p.file_stem().is_some())
        .collect::<Vec<PathBuf>>();
    log_files.sort();
    Ok(log_files)
}

fn load_index_from_readers(readers: &mut Vec<BufReader<File>>, index: &mut BTreeMap<String, Postion>) -> Result<u64> {
    let mut uncompacted = 0u64;
    let mut i = 0u64;
    for reader in readers {
        loop {
            let mut buf = String::new();
            let position = reader.stream_position()?;
            let size = reader.read_line(&mut buf)?;
            if size == 0 {
                break;
            }
            match serde_json::from_str::<Command>(buf.trim_end())? {
                Command::Set{key, value: _} => {
                    if let Some(pos) = index.insert(key, Postion {
                        file: i,
                        position: position,
                        size: size as u64,
                    }) {
                        uncompacted += pos.size;
                    }
                }
                Command::Rm{key} => {
                    if let Some(pos) = index.remove(&key) {
                        uncompacted += pos.size;
                    }
                }
            }
        }
        i += 1;
    }

    Ok(uncompacted)
}
