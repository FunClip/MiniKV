#![deny(missing_docs)]

//! A simle key-value store

mod err;
mod kv;

pub use err::KvsError;
pub use err::Result;
pub use kv::KvStore;
