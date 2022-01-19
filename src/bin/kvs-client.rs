
use std::net;

use clap::AppSettings;
use clap::Parser;

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
    let _opt = Opt::parse();
}