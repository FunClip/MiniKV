#![deny(missing_docs)]

//! A simle key-value store

mod client;
mod err;
mod kv;
mod kvse;
mod server;

pub use err::KvsError;
pub use err::Result;
pub use kv::KvStore;
pub use kvse::KvsEngine;
