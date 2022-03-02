use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{serde, KvsError, Request, Response, Result};

/// Key value store client
pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    /// Create a instance of `KvsClient` by connect to a given address of `KvsServer`
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<KvsClient> {
        let stream = TcpStream::connect(addr)?;
        Ok(KvsClient { stream })
    }

    /// Get the value of a given key from the server
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let request = Request::Get { key };

        self.stream
            .write_all(serde::to_string(&request)?.as_bytes())?;
        self.stream.flush()?;

        let mut buf = String::new();
        self.stream.read_to_string(&mut buf)?;
        match serde::from_str(&buf)? {
            Response::Success { result } => Ok(result),
            Response::Fail { message } => Err(KvsError::Server(message)),
        }
    }

    /// Set the value of a given key in the server
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let request = Request::Set { key, value };

        self.stream
            .write_all(serde::to_string(&request)?.as_bytes())?;
        self.stream.flush()?;

        let mut buf = String::new();
        self.stream.read_to_string(&mut buf)?;
        match serde::from_str(&buf)? {
            Response::Success { result: _ } => Ok(()),
            Response::Fail { message } => Err(KvsError::Server(message)),
        }
    }

    /// Remove the given key in the server
    pub fn remove(&mut self, key: String) -> Result<()> {
        let request = Request::Rm { key };

        self.stream
            .write_all(serde::to_string(&request)?.as_bytes())?;
        self.stream.flush()?;

        let mut buf = String::new();
        self.stream.read_to_string(&mut buf)?;
        match serde::from_str(&buf)? {
            Response::Success { result: _ } => Ok(()),
            Response::Fail { message } => Err(KvsError::Server(message)),
        }
    }
}
