use std::env::current_dir;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::{fs, net};

use clap::ArgEnum;
use clap::Parser;
use kvs::{KvStore, KvsError, KvsServer, Result};

use slog::{info, Logger};
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
            "sled" => Ok(Engine::Sled),
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

fn main() {
    // Initialize logger
    let mut builder = TerminalLoggerBuilder::new();
    builder.destination(Destination::Stderr);

    let logger = builder.build().unwrap_or_else(|e| {
        eprintln!("Create logger failed: {}", e);
        exit(1);
    });

    let mut opt = Opt::parse();

    if let Err(e) = default_engine(&mut opt) {
        eprintln!("Create engine failed: {}", e);
        exit(1);
    };

    if let Err(e) = run_server(logger, opt) {
        eprintln!("Run server failed: {}", e);
        exit(1);
    };
}

fn default_engine(opt: &mut Opt) -> Result<()> {
    let engine_file = current_dir()?.join("engine");

    if !engine_file.exists() && opt.engine.is_none() {
        opt.engine = Some(DEFAULT_ENGINE);
        save_selected_engine(&engine_file, &DEFAULT_ENGINE)?;
        return Ok(());
    }

    match fs::read_to_string(engine_file)?.parse::<Engine>() {
        Ok(en) => {
            if opt.engine.is_none() {
                opt.engine = Some(en);
                Ok(())
            } else if opt.engine != Some(en) {
                eprintln!(
                    "currently selected engine is inconsistent with the persisted data engine"
                );
                exit(1);
            } else {
                Ok(())
            }
        }
        Err(_) => {
            eprintln!("The current persistent data engine is invalid");
            exit(1);
        }
    }
}

fn save_selected_engine(path: impl Into<PathBuf>, engine: &Engine) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path.into())?);
    writer.write_all(format!("{}", engine).as_bytes())?;
    Ok(())
}

fn run_server(logger: Logger, opt: Opt) -> Result<()> {
    let addr = opt.addr;
    let engine = opt.engine.unwrap();

    info!(logger, "kvs-server version: {}", env!("CARGO_PKG_VERSION"));
    info!(logger, "Start on `{}` with engine `{}`", addr, engine);

    let dir = current_dir()?.join("log");

    match engine {
        Engine::Kvs => {
            KvsServer::new(&logger, KvStore::open(dir)?)?.run(addr)?;
        }
        Engine::Sled => {
            KvsServer::new(&logger, KvStore::open(dir)?)?.run(addr)?;
        }
    }

    Ok(())
}
