use std::process::exit;

use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            author = env!("CARGO_PKG_AUTHORS"),
            about = env!("CARGO_PKG_DESCRIPTION"),
            setting = AppSettings::DisableHelpSubcommand,
            setting = AppSettings::SubcommandRequiredElseHelp,
            setting = AppSettings::VersionlessSubcommands,
        )]
struct Opt {
    #[structopt(subcommand)]
    sub_command: SubCommand,
}

#[derive(StructOpt, Debug)]
enum SubCommand {
    /// Set the value of a string key to a string
    Set {
        /// A string key
        #[structopt(name = "KEY")]
        key: String,
        /// A string value of the key
        #[structopt(name = "VALUE")]
        value: String,
    },
    /// Get the string value of a given string key
    Get {
        /// A string key
        #[structopt(name = "KEY")]
        key: String,
    },
    /// Remove a given key
    Rm {
        /// A string key
        #[structopt(name = "KEY")]
        key: String,
    },
}

fn main() {
    let opt = Opt::from_args();
    match opt.sub_command {
        SubCommand::Set { key: _, value: _ } => {
            eprintln!("unimplemented");
            exit(1);
        }
        SubCommand::Get { key: _ } => {
            eprintln!("unimplemented");
            exit(1);
        }
        SubCommand::Rm { key: _ } => {
            eprintln!("unimplemented");
            exit(1);
        }
    }
}
