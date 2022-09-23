use std::{
    io::{BufReader, Read, Write},
    net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs, SocketAddr},
};

use crate::{serde, thread_pool::ThreadPool, KvsEngine, Request, Response};
use slog::{error, info, Logger};

use crate::Result;

/// Key value store server
pub struct KvsServer<E: KvsEngine, P: ThreadPool> {
    logger: Logger,
    engine: E,
    pool: P,
    listener: TcpListener,
}

impl<E: KvsEngine, P: ThreadPool> KvsServer<E, P> {
    /// Create a instance of `KvsServer`
    pub fn new<A: ToSocketAddrs>(logger: Logger, engine: E, pool: P, addr: A) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(KvsServer {
            logger,
            engine,
            pool,
            listener,
        })
    }

    /// Run the server by listening the `ip-port`
    pub fn run(&mut self) -> Result<()> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(peer) => {
                    info!(
                        self.logger,
                        "Accept connection from: {}",
                        &peer.peer_addr().unwrap()
                    );
                    let engine = self.engine.clone();
                    let logger = self.logger.clone();
                    self.pool.spawn(move || {
                        if let Err(e) = handler(engine, peer, &logger) {
                            error!(&logger, "Error in TCP handler: {}", e);
                        }
                    })
                }
                Err(e) => {
                    error!(self.logger, "Accept connection failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Get `ip:port` address
    pub fn get_address(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }
}

/// Tcp handle
fn handler<E: KvsEngine>(engine: E, mut stream: TcpStream, logger: &Logger) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    info!(logger, "Recieved: {:?}", &buf);

    let response = match execute(engine, serde::from_str(&buf)?) {
        Ok(result) => Response::Success { result },
        Err(e) => Response::Fail {
            message: format!("{}", e),
        },
    };

    stream.write_all(serde::to_string(&response)?.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;
    Ok(())
}

/// Execute command on store engine
fn execute<E: KvsEngine>(engine: E, request: Request) -> Result<Option<String>> {
    match request {
        Request::Get { key } => Ok(engine.get(key)?),
        Request::Set { key, value } => {
            engine.set(key, value)?;
            Ok(None)
        }
        Request::Rm { key } => {
            engine.remove(key)?;
            Ok(None)
        }
    }
}
