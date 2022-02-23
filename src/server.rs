use std::net::ToSocketAddrs;

use crate::KvsEngine;
use slog::Logger;

use crate::Result;

pub struct KvsServer<'ks, E: KvsEngine> {
    logger: &'ks Logger,
    engine: E,
}

impl<'ks, E: KvsEngine> KvsServer<'ks, E> {
    pub fn new(logger: &'ks Logger, engine: E) -> Result<Self> {
        Ok(KvsServer { logger, engine })
    }

    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        todo!()
    }
}
