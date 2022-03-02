use std::net;
use std::process::exit;

use clap::AppSettings;
use clap::Parser;
use kvs::KvsError;

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";

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
        /// Accepts an IP address, either v4 or v6, and a port number, with the format 'IP:PORT'.
        #[clap(
            long,
            value_name = "IP-PORT",
            default_value = DEFAULT_SERVER_ADDR)]
        addr: net::SocketAddr,
    },
    /// Get the string value of a given string key
    Get {
        /// A string key
        #[clap(name = "KEY")]
        key: String,
        /// Accepts an IP address, either v4 or v6, and a port number, with the format 'IP:PORT'.
        #[clap(
            long,
            value_name = "IP-PORT",
            default_value = DEFAULT_SERVER_ADDR)]
        addr: net::SocketAddr,
    },
    /// Remove a given key
    Rm {
        /// A string key
        #[clap(name = "KEY")]
        key: String,
        /// Accepts an IP address, either v4 or v6, and a port number, with the format 'IP:PORT'.
        #[clap(
            long,
            value_name = "IP-PORT",
            default_value = DEFAULT_SERVER_ADDR)]
        addr: net::SocketAddr,
    },
}
fn main() {
    let opt = Opt::parse();
    if let Err(e) = run_client(opt) {
        eprint!("{}", e);
        exit(1);
    }
}

fn run_client(opt: Opt) -> kvs::Result<()> {
    match opt.sub_command {
        SubCommand::Set { key, value, addr } => {
            let mut client = kvs::KvsClient::new(addr)?;
            client.set(key, value)?;
            println!("Set success!");
        }
        SubCommand::Get { key, addr } => {
            let mut client = kvs::KvsClient::new(addr)?;
            if let Some(value) = client.get(key)? {
                println!("The value is: {}", value);
            } else {
                return Err(KvsError::KeyNotFound);
            }
        }
        SubCommand::Rm { key, addr } => {
            let mut client = kvs::KvsClient::new(addr)?;
            client.remove(key)?;
            println!("Remove success!");
        }
    }
    Ok(())
}
