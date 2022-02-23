use std::net::{TcpStream, ToSocketAddrs};

use crate::Result;

pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<KvsClient> {
        let stream = TcpStream::connect(addr)?;
        Ok(KvsClient { stream })
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        todo!()
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        todo!()
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        todo!()
    }
}
