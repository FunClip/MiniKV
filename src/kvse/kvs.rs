use evmap::{ReadHandle, ShallowCopy, WriteHandle};
use serde::{Deserialize, Serialize};
use std::fs::{self, remove_dir_all, File};
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::err::KvsError;
use crate::KvsEngine;
use crate::Result;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;
const BLOCK_THRESHOLD: u64 = 1024 * 1024 * 256;

/// The `KvStore` stores key-value pairs
///
/// Key-value pairs are stored in memory by `HashMap` and not persisted in disk.
/// - Support concorrent access with lock-free read operation
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let store = KvStore::new();
/// store.set(String::from("key"), String::from("value"));
/// let val = store.get(String::from("key"));
/// assert_eq!(val, Some(String::from("value")));
/// ```
#[derive(Clone)]
pub struct KvStore {
    writer: Arc<Mutex<KvStoreWriter>>,
    reader: KvStoreReader,
    path: PathBuf,
}

/// `KvStoreReader` hold the read handle of index
/// which could shared by thread with `clone()` method
#[derive(Clone)]
struct KvStoreReader(ReadHandle<String, Position>);

/// `KvStoreWriter` hold the write handle of index
/// Only synchronous access
struct KvStoreWriter {
    writer: BufWriter<File>,
    current_block: u64,
    uncompacted: u64,
    gen: u64,
    index: WriteHandle<String, Position>,
    reader: KvStoreReader,
    path: PathBuf,
}

impl Deref for KvStoreReader {
    type Target = ReadHandle<String, Position>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl KvStoreReader {
    fn get(&self, key: String, path: &Path) -> Result<Option<String>> {
        match self.0.get_one(&key) {
            Some(pos) => {
                let buf = read_string_from(
                    path.join(get_file_path(pos.gen, pos.file)),
                    pos.position,
                    pos.size,
                )?;

                match serde_json::from_str(&buf).unwrap() {
                    Command::Set { key: _, value } => Ok(Some(value)),
                    Command::Rm { key: _ } => unreachable!(),
                }
            }
            None => Ok(None),
        }
    }
}

impl KvStoreWriter {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let position = self.write_cmd_to(Command::Set {
            key: key.clone(),
            value,
        })?;

        self.index.update(key.clone(), position);

        if let Some(pos) = self.reader.get_one(&key) {
            self.uncompacted += pos.size;
        }

        self.index.refresh();
        self.try_compact()?;

        Ok(())
    }

    fn new_block(&mut self) -> Result<()> {
        self.current_block += 1;

        self.writer = BufWriter::new(File::create(
            self.path.join(get_file_path(self.gen, self.current_block)),
        )?);

        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let exist_size = if let Some(pos) = self.reader.get_one(&key) {
            pos.size
        } else {
            0
        };
        if exist_size != 0 {
            let pos = self.write_cmd_to(Command::Rm { key: key.clone() })?;
            self.index.empty(key);

            self.uncompacted += exist_size + pos.size as u64;

            self.index.refresh();
            self.try_compact()?;

            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    fn write_cmd_to(&mut self, cmd: Command) -> Result<Position> {
        let mut cmd = serde_json::to_string(&cmd)?;
        cmd += "\n";

        self.write_string_to(cmd)
    }

    fn write_string_to(&mut self, s: String) -> Result<Position> {
        if s.len() as u64 + self.writer.stream_position()? > BLOCK_THRESHOLD {
            self.new_block()?;
        }

        let position = self.writer.stream_position()?;
        let size = self.writer.write(s.as_bytes())?;
        let file = self.current_block;

        self.writer.flush()?;

        Ok(Position {
            file,
            position,
            size: size as u64,
            gen: self.gen,
        })
    }

    /// Compaction steps:
    /// 1. current `gen` add 1, and create a new directory
    /// 2. reset some pointer and counter
    /// 3. traverse index and write them into new directory
    /// 4. delete second last backup
    fn try_compact(&mut self) -> Result<()> {
        if self.uncompacted > COMPACTION_THRESHOLD {
            // Create new genaration of store
            self.gen += 1;
            let new_generation_path = self.path.join(get_store_dir_by(self.gen));
            if new_generation_path.exists() {
                fs::remove_dir_all(&new_generation_path)?;
            }
            fs::create_dir_all(&new_generation_path)?;

            // reset some "pointer"
            self.uncompacted = 0;
            self.current_block = 0;
            self.writer = BufWriter::new(File::create(
                self.path.join(get_file_path(self.gen, self.current_block)),
            )?);

            // read from old index and write them into new generation store
            let index_reader = self.reader.clone();
            for (key, value) in index_reader.read().unwrap().iter() {
                let pos = value.get_one().unwrap();
                let buf = read_string_from(
                    self.path.join(get_file_path(pos.gen, pos.file)),
                    pos.position,
                    pos.size,
                )?;
                let new_position = self.write_string_to(buf)?;
                self.index.update(key.clone(), new_position);
            }

            self.index.refresh();

            if self.gen > 1 && self.path.join(get_store_dir_by(self.gen - 2)).exists() {
                remove_dir_all(self.path.join(get_store_dir_by(self.gen - 2)))?;
            }
        }
        Ok(())
    }
}

impl KvsEngine for KvStore {
    /// Set the value of a string key to a string.
    ///
    /// If the key exists, the value will be overwritten.
    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().unwrap().set(key, value)
    }

    /// Get the string value of a given string key
    ///
    /// Return `None` if the key doesn't exist.
    fn get(&self, key: String) -> Result<Option<String>> {
        self.reader.get(key, &self.path)
    }

    /// Remove a given key.
    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }

    /// Open or create a `KvStore`
    fn open(path: impl Into<PathBuf>) -> Result<Self> {
        KvStore::open(path)
    }
}

impl KvStore {
    /// Open a `KvStore` with the given path.
    ///
    /// If the given path doesn't exist, it will create one.
    ///
    /// # Errors
    ///
    ///
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let (index_r, mut index) = evmap::new();
        let mut uncompacted = 0u64;
        let path = path.into();
        // get store generation of given path
        let gen = get_generation(&path)?;
        // get block num of current generation
        let files = get_log_files(&path.join(get_store_dir_by(gen)))?;
        let current_block = if files.is_empty() {
            0
        } else {
            (files.len() - 1) as u64
        };

        // build index from files
        if !files.is_empty() {
            uncompacted += load_index_from_files(&files, &mut index, gen)?;
        }

        // create BufWriter
        let mut writer = if files.is_empty() {
            BufWriter::new(
                File::options()
                    .create(true)
                    .write(true)
                    .open(path.join(get_file_path(gen, 0)))?,
            )
        } else {
            BufWriter::new(
                File::options()
                    .write(true)
                    .open(&files[current_block as usize])?,
            )
        };

        writer.seek(SeekFrom::End(0))?;

        let reader = KvStoreReader(index_r);

        Ok(KvStore {
            writer: Arc::new(Mutex::new(KvStoreWriter {
                writer,
                current_block,
                uncompacted,
                gen,
                index,
                path: path.to_owned(),
                reader: reader.clone(),
            })),
            reader,
            path,
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

/// Log entry's position in files
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct Position {
    gen: u64,
    file: u64,
    position: u64,
    size: u64,
}

impl ShallowCopy for Position {
    unsafe fn shallow_copy(&self) -> ManuallyDrop<Self> {
        ManuallyDrop::new(*self)
    }
}

fn get_file_path(gen: u64, file: u64) -> String {
    format!("gen_{}/{}.log", gen, file)
}

fn get_store_dir_by(gen: u64) -> String {
    format!("gen_{}", gen)
}

fn read_string_from(file_path: PathBuf, position: u64, size: u64) -> Result<String> {
    let mut buf = String::new();
    let mut reader = File::open(file_path)?;

    reader.seek(SeekFrom::Start(position))?;
    reader.take(size).read_to_string(&mut buf)?;

    Ok(buf)
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

/// Get generation from existent store path
/// - case 1: 0 generation dir, a new store
/// - case 2: 1 generation dir, no compaction happened
/// - case 3: 2 generation dirs, compaction happend and succeed
/// - case 4: 3 generation dirs, compaction happend but not completed
fn get_generation(path: &Path) -> Result<u64> {
    if !path.exists() {
        fs::create_dir_all(path.join(get_store_dir_by(0)))?;
        return Ok(0);
    }
    let mut gens = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.path().file_name().is_some())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|e| e.strip_prefix("gen_").and_then(|s| s.parse::<u64>().ok()))
        .collect::<Vec<u64>>();
    gens.sort_unstable();

    match gens.len() {
        0 => {
            fs::create_dir_all(path.join(get_store_dir_by(0)))?;
            Ok(0)
        }
        1 => Ok(gens[0]),
        2 => Ok(gens[1]),
        _ => Ok(gens[gens.len() - 2]),
    }
}

/// Traverse log files and execute the command, build the index in memory
fn load_index_from_files(
    files: &[PathBuf],
    index: &mut WriteHandle<String, Position>,
    gen: u64,
) -> Result<u64> {
    let mut uncompacted = 0u64;
    for (i, file) in files.iter().enumerate() {
        let mut reader = BufReader::new(File::open(file)?);
        loop {
            let mut buf = String::new();
            let position = reader.stream_position()?;
            let size = reader.read_line(&mut buf)?;
            if size == 0 {
                break;
            }
            match serde_json::from_str::<Command>(buf.trim_end())? {
                Command::Set { key, value: _ } => {
                    index.update(
                        key.clone(),
                        Position {
                            gen,
                            file: i as u64,
                            position,
                            size: size as u64,
                        },
                    );

                    if let Some(pos) = index.get_one(&key) {
                        uncompacted += pos.size;
                    }
                }
                Command::Rm { key } => {
                    let exist_size = if let Some(pos) = index.get_one(&key) {
                        pos.size
                    } else {
                        0
                    };
                    index.empty(key);
                    uncompacted += size as u64 + exist_size;
                }
            }
            index.refresh();
        }
    }

    Ok(uncompacted)
}
