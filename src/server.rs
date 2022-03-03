use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs},
};

use crate::{serde, KvsEngine, Request, Response};
use slog::{error, info, Logger};

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
                    self.handler(&peer)?;
                }
                Err(e) => {
                    error!(self.logger, "Accept connection failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Tcp handle
    fn handler(&mut self, stream: &TcpStream) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;

        info!(self.logger, "Recieved: {:?}", &buf);

        let response = match self.execute(serde::from_str(&buf)?) {
            Ok(result) => Response::Success { result },
            Err(e) => Response::Fail {
                message: format!("{}", e),
            },
        };

        let mut writer = BufWriter::new(stream);
        writer.write_all(serde::to_string(&response)?.as_bytes())?;
        writer.flush()?;
        stream.shutdown(Shutdown::Write)?;
        Ok(())
    }

    /// Execute command on store engine
    fn execute(&mut self, request: Request) -> Result<Option<String>> {
        match request {
            Request::Get { key } => Ok(self.engine.get(key)?),
            Request::Set { key, value } => {
                self.engine.set(key, value)?;
                Ok(None)
            }
            Request::Rm { key } => {
                self.engine.remove(key)?;
                Ok(None)
            }
        }
    }
}
