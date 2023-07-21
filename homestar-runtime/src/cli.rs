//! CLI commands/arguments.

use crate::network::rpc::Client;
use anyhow::anyhow;
use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

mod error;
mod show;
pub use error::Error;

const TMP_DIR: &str = "/tmp";
const HELP_TEMPLATE: &str = "{about} {version}

USAGE:
    {usage}

{all-args}
";

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, help_template = HELP_TEMPLATE)]
pub struct Cli {
    /// Homestar [Command].
    #[clap(subcommand)]
    pub command: Command,
}

/// CLI Argument types.
#[derive(Debug, Parser)]
pub enum Command {
    /// Start the Homestar runtime.
    Start {
        /// Database url, defaults to sqlite://homestar.db.
        #[arg(
            long = "db",
            value_name = "DB",
            env = "DATABASE_URL",
            help = "SQLite database url"
        )]
        database_url: Option<String>,
        /// Optional runtime configuration file, otherwise use defaults.
        #[arg(
            short = 'c',
            long = "config",
            value_name = "CONFIG",
            help = "runtime configuration file"
        )]
        runtime_config: Option<String>,
        /// Daemonize the runtime, false by default.
        #[arg(
            short = 'd',
            long = "daemonize",
            default_value_t = false,
            help = "daemonize the runtime"
        )]
        daemonize: bool,
        /// Directory to place daemon files, defaults to /tmp.
        #[arg(
            long = "daemon_dir",
            default_value = TMP_DIR,
            value_hint = clap::ValueHint::DirPath,
            value_name = "DIR",
            help = "directory to place daemon files"
        )]
        daemon_dir: PathBuf,
    },
    /// Stop the Homestar runtime.
    Stop {
        #[arg(
            long = "host",
            default_value_t = String::from("::1"),
            value_hint = clap::ValueHint::Hostname
        )]
        /// RPC Homestar runtime host to ping.
        host: String,
        #[arg(short = 'p', long = "port", default_value_t = 3030)]
        /// RPC Homestar runtime port to ping.
        port: u16,
    },
    /// Ping the Homestar runtime.
    Ping {
        #[arg(
            long = "host",
            default_value_t = String::from("::1"),
            value_hint = clap::ValueHint::Hostname
        )]
        /// RPC Homestar runtime host to ping.
        host: String,
        #[arg(short = 'p', long = "port", default_value_t = 3030)]
        /// RPC Homestar runtime port to ping.
        port: u16,
    },
    /// Run a workflow, given a workflow json file.
    Run {
        /// Path to workflow json file.
        #[arg(
            short='w',
            long = "workflow",
            value_hint = clap::ValueHint::FilePath,
            value_name = "FILE",
            help = "path to workflow file"
        )]
        workflow: PathBuf,
    },
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Command::Start { .. } => "start",
            Command::Stop { .. } => "stop",
            Command::Ping { .. } => "ping",
            Command::Run { .. } => "run",
        }
    }

    /// Handle CLI commands related to [Client] RPC calls.
    #[allow(clippy::unnecessary_wraps)]
    pub fn handle_rpc_command(&self) -> Result<(), Error> {
        // Spin up a new tokio runtime on the current thread.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        match self {
            Command::Ping { host, port } => {
                let host = IpAddr::from_str(host).map_err(anyhow::Error::new)?;
                let addr = SocketAddr::new(host, *port);
                let response = rt.block_on(async {
                    let client = Client::new(addr).await?;
                    let response = client.ping().await?;
                    Ok::<String, Error>(response)
                })?;

                show::Ping::table(addr, response).echo()?;
                Ok(())
            }
            Command::Stop { host, port } => {
                let host = IpAddr::from_str(host).map_err(anyhow::Error::new)?;
                let addr = SocketAddr::new(host, *port);
                rt.block_on(async {
                    let client = Client::new(addr).await?;
                    let _ = client.stop().await?;
                    Ok::<(), Error>(())
                })?;

                Ok(())
            }
            _ => Err(anyhow!("Invalid command {}", self.name()).into()),
        }
    }
}
