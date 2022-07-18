use std::env::current_dir;
use std::process::exit;

use clap::AppSettings;
use clap::Parser;
use kvs::KvsEngine;

#[derive(Parser, Debug)]
#[clap(name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            author = env!("CARGO_PKG_AUTHORS"),
            about = env!("CARGO_PKG_DESCRIPTION"),
            setting = AppSettings::DisableHelpSubcommand,
            setting = AppSettings::SubcommandRequiredElseHelp,
        )]
struct Opt {
    #[clap(subcommand)]
    sub_command: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Set the value of a string key to a string
    Set {
        /// A string key
        #[clap(name = "KEY")]
        key: String,
        /// A string value of the key
        #[clap(name = "VALUE")]
        value: String,
    },
    /// Get the string value of a given string key
    Get {
        /// A string key
        #[clap(name = "KEY")]
        key: String,
    },
    /// Remove a given key
    Rm {
        /// A string key
        #[clap(name = "KEY")]
        key: String,
    },
}

fn main() -> kvs::Result<()> {
    let opt = Opt::parse();
    match opt.sub_command {
        SubCommand::Set { key, value } => {
            let store = kvs::KvStore::open(current_dir()?)?;
            store.set(key, value)?;
        }
        SubCommand::Get { key } => {
            let store = kvs::KvStore::open(current_dir()?)?;
            match store.get(key) {
                Ok(Some(value)) => print!("{}", value),
                Ok(None) | Err(kvs::KvsError::KeyNotFound) => {
                    print!("Key not found");
                    exit(0);
                }
                Err(e) => return Err(e),
            }
        }
        SubCommand::Rm { key } => {
            let store = kvs::KvStore::open(current_dir()?)?;
            match store.remove(key) {
                Err(kvs::KvsError::KeyNotFound) => {
                    print!("Key not found");
                    exit(1);
                }
                other => return other,
            }
        }
    }

    Ok(())
}
