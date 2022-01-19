use std::net;

use clap::Parser;

const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";

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
    #[clap(arg_enum, long, value_name = "ENGINE-NAME", default_value = "kvs")]
    engine: Engine,
}

#[derive(Debug, clap::ArgEnum, Clone)]
enum Engine {
    Kvs,
    Sled,
}

fn main() {
    let _opt = Opt::parse();
}
