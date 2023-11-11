//! CLI commands/arguments.

use crate::{
    network::rpc::Client,
    runner::{file, response},
};
use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    time::{Duration, SystemTime},
};
use tarpc::context;

mod error;
pub use error::Error;
pub(crate) mod show;
pub(crate) use show::ConsoleTable;

const DEFAULT_SETTINGS_FILE: &str = "./config/settings.toml";
const DEFAULT_DB_PATH: &str = "homestar.db";
const TMP_DIR: &str = "/tmp";
const HELP_TEMPLATE: &str = "{name} {version}

{about}

Usage: {usage}

{all-args}
";

/// CLI arguments.
#[derive(Debug, Parser)]
#[command(bin_name = "homestar", name = "homestar", author, version, about,
          long_about = None, help_template = HELP_TEMPLATE)]
pub struct Cli {
    /// Homestar [Command].
    #[clap(subcommand)]
    pub command: Command,
}

/// General RPC arguments for [Client] commands.
///
/// [Client]: crate::network::rpc::Client
#[derive(Debug, Clone, PartialEq, Args, Serialize, Deserialize)]
pub struct RpcArgs {
    /// Homestar RPC host.
    #[clap(
            long = "host",
            default_value = "::1",
            value_hint = clap::ValueHint::Hostname
        )]
    host: IpAddr,
    /// Homestar RPC port.
    #[clap(short = 'p', long = "port", default_value_t = 3030)]
    port: u16,
    /// Homestar RPC timeout.
    #[clap(long = "timeout", default_value = "60s", value_parser = humantime::parse_duration)]
    timeout: Duration,
}

impl Default for RpcArgs {
    fn default() -> Self {
        Self {
            host: Ipv6Addr::LOCALHOST.into(),
            port: 3030,
            timeout: Duration::from_secs(60),
        }
    }
}

/// CLI Argument types.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the Homestar runtime.
    Start {
        /// Database URL, defaults to homestar.db.
        #[arg(
            long = "db",
            value_name = "DB",
            env = "DATABASE_PATH",
            value_hint = clap::ValueHint::AnyPath,
            value_name = "DATABASE_PATH",
            default_value = DEFAULT_DB_PATH,
            help = "Database path (SQLite) [optional]"
        )]
        database_url: Option<String>,
        /// Runtime configuration file (.toml), defaults to ./config/settings.toml.
        #[arg(
            short = 'c',
            long = "config",
            value_hint = clap::ValueHint::FilePath,
            value_name = "CONFIG",
            default_value = DEFAULT_SETTINGS_FILE,
            help = "Runtime configuration file (.toml) [optional]"
        )]
        runtime_config: Option<PathBuf>,
        /// Daemonize the runtime, false by default.
        #[arg(
            short = 'd',
            long = "daemonize",
            default_value = "false",
            help = "Daemonize the runtime"
        )]
        daemonize: bool,
        /// Directory to place daemon files, defaults to /tmp.
        #[arg(
            long = "daemon_dir",
            default_value = TMP_DIR,
            value_hint = clap::ValueHint::DirPath,
            value_name = "DIR",
            help = "Directory to place daemon file(s)"
        )]
        daemon_dir: PathBuf,
    },
    /// Stop the Homestar runtime.
    Stop(RpcArgs),
    /// Ping the Homestar runtime to see if it's running.
    Ping(RpcArgs),
    /// Run an IPVM-configured workflow file on the Homestar runtime.
    Run {
        /// RPC host / port arguments.
        #[clap(flatten)]
        args: RpcArgs,
        /// Local name associated with a workflow (optional).
        #[arg(
            short = 'n',
            long = "name",
            value_name = "NAME",
            help = "Local name given to a workflow (optional)"
        )]
        name: Option<String>,
        /// IPVM-configured workflow file to run.
        /// Supported:
        ///   - JSON (.json).
        #[arg(
            short='w',
            long = "workflow",
            value_hint = clap::ValueHint::FilePath,
            value_name = "FILE",
            value_parser = clap::value_parser!(file::ReadWorkflow),
            help = r#"IPVM-configured workflow file to run.
Supported:
  - JSON (.json)"#
        )]
        workflow: file::ReadWorkflow,
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
    pub fn handle_rpc_command(self) -> Result<(), Error> {
        // Spin up a new tokio runtime on the current thread.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        match self {
            Command::Ping(args) => {
                let (client, response) = rt.block_on(async {
                    let client = args.client().await?;
                    let response = client.ping().await?;
                    Ok::<(Client, String), Error>((client, response))
                })?;

                let response = response::Ping::new(client.addr(), response);
                response.echo_table()?;
                Ok(())
            }
            Command::Stop(args) => rt.block_on(async {
                let client = args.client().await?;
                client.stop().await??;
                Ok(())
            }),
            Command::Run {
                args,
                name,
                workflow: workflow_file,
            } => {
                let response = rt.block_on(async {
                    let client = args.client().await?;
                    let response = client.run(name.map(|n| n.into()), workflow_file).await??;
                    Ok::<Box<response::AckWorkflow>, Error>(response)
                })?;

                response.echo_table()?;
                Ok(())
            }
            _ => Err(anyhow!("Invalid command {}", self.name()).into()),
        }
    }
}

impl RpcArgs {
    async fn client(&self) -> Result<Client, Error> {
        let addr = SocketAddr::new(self.host, self.port);
        let mut ctx = context::current();
        ctx.deadline = SystemTime::now() + self.timeout;
        let client = Client::new(addr, ctx).await?;
        Ok(client)
    }
}
