use std::env::current_dir;
use std::fmt::Display;
use std::process::exit;
use std::str::FromStr;
use std::{fs, net};

use clap::ArgEnum;
use clap::Parser;
use kvs::{KvsError, Result};

use slog::info;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::Build;

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";
const DEFAULT_ENGINE: Engine = Engine::Kvs;

#[derive(Parser, Debug)]
#[clap(name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            author = env!("CARGO_PKG_AUTHORS"),
            about = env!("CARGO_PKG_DESCRIPTION"),
        )]
struct Opt {
    /// Sets the listening address
    #[clap(
        long,
        value_name = "IP-PORT",
        default_value = DEFAULT_SERVER_ADDR)]
    addr: net::SocketAddr,

    /// Sets the storage engine
    #[clap(arg_enum, long, value_name = "ENGINE-NAME")]
    engine: Option<Engine>,
}

#[derive(Debug, ArgEnum, Clone, PartialEq, Eq)]
enum Engine {
    Kvs,
    Sled,
}

impl FromStr for Engine {
    type Err = KvsError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "kvs" => Ok(Engine::Kvs),
            "seld" => Ok(Engine::Sled),
            _ => Err(KvsError::InValidEngine),
        }
    }
}

impl Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Engine::Kvs => write!(f, "kvs"),
            Engine::Sled => write!(f, "sled"),
        }
    }
}

fn main() -> Result<()> {
    // Initialize logger
    let mut builder = TerminalLoggerBuilder::new();
    builder.destination(Destination::Stderr);

    let logger = builder.build()?;

    let mut opt = Opt::parse();
    default_engine(&mut opt)?;

    info!(logger, "kvs-server version: {}", env!("CARGO_PKG_VERSION"));
    info!(
        logger,
        "Start on `{}` with engine `{}`",
        opt.addr,
        opt.engine.unwrap()
    );

    Ok(())
}

fn default_engine(opt: &mut Opt) -> Result<()> {
    let engine_file = current_dir()?.join("engine");

    if !engine_file.exists() && opt.engine.is_none() {
        opt.engine = Some(DEFAULT_ENGINE);
        return Ok(());
    }

    match fs::read_to_string(engine_file)?.parse::<Engine>() {
        Ok(en) => {
            if opt.engine.is_none() {
                opt.engine = Some(en);
                Ok(())
            } else if opt.engine != Some(en) {
                exit(1);
            } else {
                Ok(())
            }
        }
        Err(e) => {
            if opt.engine.is_none() {
                opt.engine = Some(DEFAULT_ENGINE);
            }
            Err(e)
        }
    }
}
