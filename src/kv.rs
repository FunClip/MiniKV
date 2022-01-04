use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, copy, remove_dir_all, remove_file, File};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
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
    path: PathBuf,
}

impl KvStore {
    /// Set the value of a string key to a string.
    ///
    /// If the key exists, the value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let mut cmd = serde_json::to_string(&Command::Set {
            key: key.clone(),
            value,
        })?;
        cmd += "\n";
        let position = self.writer.stream_position()?;
        let size = self.writer.write(cmd.as_bytes())?;
        let file = self.current - 1;
        self.writer.flush()?;

        if let Some(pos) = self.index.insert(
            key,
            Postion {
                file,
                position,
                size: size as u64,
            },
        ) {
            self.uncompacted += pos.size;
        }

        self.check()?;

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
                    Command::Set { key: _, value } => Ok(Some(value)),
                    Command::Rm { key: _ } => unreachable!(),
                }
            }
            None => Ok(None),
        }
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.index.remove(&key) {
            Some(pos) => {
                let mut cmd = serde_json::to_string(&Command::Rm { key: key.clone() })?;
                cmd += "\n";
                self.writer.stream_position()?;
                let size = self.writer.write(cmd.as_bytes())?;
                self.writer.flush()?;
                self.uncompacted += pos.size + size as u64;
                self.check()?;
                Ok(())
            }
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
            readers.push(BufReader::new(File::open(file)?))
        }

        // build index from readers
        uncompacted += load_index_from_readers(&mut readers, &mut index)?;

        // create BufWriter
        let mut writer = if current == 0
            || readers[(current - 1) as usize].stream_position()? >= BLOCK_THRESHOLD
        {
            current += 1;
            let f_w = File::create(&path.join(format!("{}.log", current)))?;
            let f_r = File::open(&path.join(format!("{}.log", current)))?;
            readers.push(BufReader::new(f_r));
            BufWriter::new(f_w)
        } else {
            BufWriter::new(
                File::options()
                    .write(true)
                    .open(&files[(current - 1) as usize])?,
            )
        };

        writer.seek(SeekFrom::End(0))?;

        Ok(KvStore {
            writer,
            readers,
            current,
            index,
            uncompacted,
            path,
        })
    }

    /// Check if compaction or new block(file) is needed
    fn check(&mut self) -> Result<()> {
        if self.uncompacted >= COMPACTION_THRESHOLD {
            self.compact()?;
        }

        if self.writer.stream_position()? >= BLOCK_THRESHOLD {
            self.current += 1;
            let f_w = File::create(&self.path.join(format!("{}.log", self.current)))?;
            let f_r = File::open(&self.path.join(format!("{}.log", self.current)))?;
            self.readers.push(BufReader::new(f_r));
            self.writer = BufWriter::new(f_w)
        }

        Ok(())
    }

    /// Compact current log blocks(files)
    fn compact(&mut self) -> Result<()> {
        let tmp_path = self.path.join("tmp");
        fs::create_dir_all(&tmp_path)?;

        let mut block_num = 1;
        self.writer = BufWriter::new(File::create(&tmp_path.join(format!("{}.log", block_num)))?);

        // Write datas currently in `index` to new logs file
        for pos in &mut self.index.values_mut() {
            if self.writer.stream_position()? >= BLOCK_THRESHOLD {
                block_num += 1;
                self.writer =
                    BufWriter::new(File::create(&tmp_path.join(format!("{}.log", block_num)))?);
            }
            let mut buf = String::new();
            let reader = self.readers.get_mut(pos.file as usize).unwrap();
            reader.seek(SeekFrom::Start(pos.position))?;
            reader.read_line(&mut buf)?;
            self.writer.write_all(buf.as_bytes())?;
            self.writer.flush()?;
        }

        self.readers.clear();
        self.index.clear();

        // Delete old log files
        for file in get_log_files(&self.path)? {
            remove_file(file)?;
        }

        // Copy new log files to workdir
        for file in get_log_files(&tmp_path)? {
            copy(&file, &self.path.join(&file.file_name().unwrap()))?;
        }

        // Setup from new log files
        self.current = 0;
        let files = get_log_files(&self.path)?;
        for file in &files {
            self.current += 1;
            self.readers.push(BufReader::new(File::open(file)?))
        }

        self.uncompacted = load_index_from_readers(&mut self.readers, &mut self.index)?;

        self.writer =
            if self.readers[(self.current - 1) as usize].stream_position()? >= BLOCK_THRESHOLD {
                self.current += 1;
                let f_w = File::create(&self.path.join(format!("{}.log", self.current)))?;
                let f_r = File::open(&self.path.join(format!("{}.log", self.current)))?;
                self.readers.push(BufReader::new(f_r));
                BufWriter::new(f_w)
            } else {
                BufWriter::new(
                    File::options()
                        .write(true)
                        .open(&files[(self.current - 1) as usize])?,
                )
            };

        self.writer.seek(SeekFrom::End(0))?;

        self.uncompacted = 0;

        remove_dir_all(tmp_path)?;

        Ok(())
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

fn load_index_from_readers(
    readers: &mut Vec<BufReader<File>>,
    index: &mut BTreeMap<String, Postion>,
) -> Result<u64> {
    let mut uncompacted = 0u64;
    for (i, reader) in readers.iter_mut().enumerate() {
        loop {
            let mut buf = String::new();
            let position = reader.stream_position()?;
            let size = reader.read_line(&mut buf)?;
            if size == 0 {
                break;
            }
            match serde_json::from_str::<Command>(buf.trim_end())? {
                Command::Set { key, value: _ } => {
                    if let Some(pos) = index.insert(
                        key,
                        Postion {
                            file: i as u64,
                            position,
                            size: size as u64,
                        },
                    ) {
                        uncompacted += pos.size;
                    }
                }
                Command::Rm { key } => {
                    if let Some(pos) = index.remove(&key) {
                        uncompacted += pos.size;
                    }
                }
            }
        }
    }

    Ok(uncompacted)
}
