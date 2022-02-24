use std::{net::{TcpStream, ToSocketAddrs}, io::{Write, Read}};

use crate::Result;

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
        let mut msg = "get ".to_owned();
        msg = msg + &key;

        self.stream.write(msg.as_bytes())?;
        self.stream.flush()?;

        let mut buf = String::new();
        self.stream.read_to_string(&mut buf)?;
        println!("{}",buf);

        Ok(Some("test".to_owned()))
    }

    /// Set the value of a given key in the server
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let mut msg = "set ".to_owned();
        msg = msg + &key + " " + &value;

        self.stream.write(msg.as_bytes())?;
        self.stream.flush()?;

        let mut buf = String::new();
        self.stream.read_to_string(&mut buf)?;
        println!("{}",buf);

        Ok(())
    }

    /// Remove the given key in the server
    pub fn remove(&mut self, key: String) -> Result<()> {
        let mut msg = "remove ".to_owned();
        msg = msg + &key;

        self.stream.write(msg.as_bytes())?;
        self.stream.flush()?;

        let mut buf = String::new();
        self.stream.read_to_string(&mut buf)?;
        println!("{}",buf);

        Ok(())
    }
}
