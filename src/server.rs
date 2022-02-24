use std::{net::{ToSocketAddrs, TcpListener}, io::{Read, BufReader, BufWriter, Write}};

use crate::KvsEngine;
use slog::{Logger, error};

use crate::Result;

/// Key value store server
pub struct KvsServer<'ks, E: KvsEngine> {
    logger: &'ks Logger,
    engine: E,
}

impl<'ks, E: KvsEngine> KvsServer<'ks, E> {
    /// Create a instance of `KvsServer`
    pub fn new(logger: &'ks Logger, engine: E) -> Result<Self> {
        Ok(KvsServer { logger, engine })
    }

    /// Run the server by listening the `ip-port`
    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut buf = String::new();
                    let mut reader = BufReader::new(&stream);
                    reader.read_to_string(&mut buf)?;
                    buf = buf + " Get it!";
                    
                    let mut writer = BufWriter::new(&stream);
                    writer.write(buf.as_bytes())?;
                    writer.flush()?;
                }
                Err(e) => {
                    error!(self.logger, "Accept connection failed: {}", e);
                }
            }
        }

        Ok(())
    }
}
