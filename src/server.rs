use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{TcpListener, ToSocketAddrs},
};

use crate::KvsEngine;
use slog::{debug, error, info, Logger};

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
                Ok(peer) => {
                    info!(
                        self.logger,
                        "Accept connection from: {}",
                        &peer.peer_addr().unwrap()
                    );
                    let mut buff = [0; 50];
                    let mut reader = BufReader::new(&peer);
                    let n = reader.read(&mut buff)?;

                    let mut buf = String::from_utf8_lossy(&buff[0..n]);

                    debug!(self.logger, "Recieved message: {}", &buf);
                    buf += " Get it!";
                    let mut writer = BufWriter::new(&peer);
                    writer.write_all(buf.as_bytes())?;
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
