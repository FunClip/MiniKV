#![deny(missing_docs)]

//! A simle key-value store

mod client;
mod err;
mod kvse;
mod proto;
mod serde;
mod server;

pub use client::KvsClient;
pub use err::KvsError;
pub use err::Result;
pub use kvse::KvStore;
pub use kvse::KvsEngine;
pub use proto::*;
pub use server::KvsServer;
