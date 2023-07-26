//! CLI commands/arguments.

use crate::{
    network::rpc::Client,
    runner::{file, response},
};
use anyhow::anyhow;
use clap::{Args, Parser};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    time::{Duration, SystemTime},
};
use tarpc::context;

mod error;
pub use error::Error;
pub(crate) mod show;
pub(crate) use show::ConsoleTable;

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

/// General RPC arguments for [Client] commands.
///
/// [Client]: crate::network::rpc::Client
#[derive(Debug, Clone, Args)]
pub struct RpcArgs {
    /// RPC Homestar runtime host to ping.
    #[clap(
            long = "host",
            default_value_t = String::from("::1"),
            value_hint = clap::ValueHint::Hostname
        )]
    host: String,
    /// RPC Homestar runtime port to ping.
    #[clap(short = 'p', long = "port", default_value_t = 3030)]
    port: u16,
    /// RPC Homestar runtime port to ping.
    #[clap(long = "timeout", default_value = "60s", value_parser = humantime::parse_duration)]
    timeout: Duration,
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
    Stop(RpcArgs),
    /// Ping the Homestar runtime.
    Ping(RpcArgs),
    /// Run a workflow, given a workflow file.
    Run {
        /// RPC host / port arguments.
        #[clap(flatten)]
        args: RpcArgs,
        #[arg(
            short='w',
            long = "workflow",
            value_hint = clap::ValueHint::FilePath,
            value_name = "FILE",
            value_parser = clap::value_parser!(file::ReadWorkflow),
            help = "path to workflow file"
        )]
        /// Workflow file to run.
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
    pub fn handle_rpc_command(&self) -> Result<(), Error> {
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
                workflow: workflow_file,
            } => {
                let response = rt.block_on(async {
                    let client = args.client().await?;
                    let response = client.run(workflow_file.to_owned()).await??;
                    Ok::<response::AckWorkflow, Error>(response)
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
        let host = IpAddr::from_str(&self.host).map_err(anyhow::Error::new)?;
        let addr = SocketAddr::new(host, self.port);
        let mut ctx = context::current();
        ctx.deadline = SystemTime::now() + self.timeout;
        let client = Client::new(addr, ctx).await?;
        Ok(client)
    }
}
