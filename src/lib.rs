#![deny(missing_docs)]

//! A simle key-value store

mod kv;
mod err;

pub use kv::KvStore;
pub use err::Result;
pub use err::KvsError;
