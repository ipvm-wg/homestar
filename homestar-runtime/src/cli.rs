//! CLI commands/arguments.

use crate::{
    network::rpc::Client,
    runner::{file, response},
    KeyType,
};
use anyhow::anyhow;
use clap::{ArgGroup, Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    time::{Duration, SystemTime},
};
use tarpc::context;

mod error;
pub use error::Error;
mod init;
pub use init::{handle_init_command, KeyArg, OutputMode};
pub(crate) mod show;
pub use show::ConsoleTable;

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
#[clap(group(ArgGroup::new("init_sink").args(&["config", "dry-run"])))]
#[clap(group(ArgGroup::new("init_key_arg").args(&["key-file", "key-seed"])))]
pub struct Cli {
    /// Homestar [Command].
    #[clap(subcommand)]
    pub command: Command,
}

/// Arguments for `init` command.
#[derive(Debug, Clone, PartialEq, Args)]
pub struct InitArgs {
    /// Runtime configuration file (.toml).
    #[arg(
            short = 'o',
            long = "output",
            value_hint = clap::ValueHint::FilePath,
            value_name = "OUTPUT",
            help = "Path to write initialized configuration file (.toml) [optional]",
            group = "init_sink"
        )]
    pub output_path: Option<PathBuf>,
    /// Skip writing to disk.
    #[arg(
        long = "dry-run",
        help = "Skip writing to disk",
        default_value = "false",
        help = "Skip writing to disk, instead writing configuration to stdout [optional]",
        group = "init_sink"
    )]
    pub dry_run: bool,
    /// Suppress auxiliary output.
    #[arg(
        short = 'q',
        long = "quiet",
        default_value = "false",
        help = "Suppress auxiliary output [optional]"
    )]
    pub quiet: bool,
    /// Force destructive operations without prompting.
    #[arg(
        short = 'f',
        long = "force",
        default_value = "false",
        help = "Force destructive operations without prompting [optional]"
    )]
    pub force: bool,
    /// Run in non-interactive mode by disabling all prompts.
    #[arg(
        long = "no-input",
        default_value = "false",
        help = "Run in non-interactive mode [optional]"
    )]
    pub no_input: bool,
    /// The type of key to use for libp2p
    #[arg(
        long = "key-type",
        value_name = "KEY_TYPE",
        help = "The type of key to use for libp2p [optional]"
    )]
    pub key_type: Option<KeyType>,
    /// The file to load the key from
    #[arg(
        long = "key-file",
        value_name = "KEY_FILE",
        help = "The path to the key file. A key will be generated if the file does not exist [optional]",
        group = "init_key_arg"
    )]
    pub key_file: Option<PathBuf>,
    /// The seed to use for generating the key
    #[arg(
        long = "key-seed",
        value_name = "KEY_SEED",
        help = "The seed to use for generating the key [optional]",
        group = "init_key_arg"
    )]
    pub key_seed: Option<Option<String>>,
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
    /// Initialize a Homestar configuration.
    Init(InitArgs),
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
        /// Runtime configuration file (.toml).
        #[arg(
            short = 'c',
            long = "config",
            value_hint = clap::ValueHint::FilePath,
            value_name = "CONFIG",
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
            value_hint = clap::ValueHint::FilePath,
            value_name = "FILE",
            value_parser = clap::value_parser!(file::ReadWorkflow),
            index = 1,
            required = true,
            help = r#"IPVM-configured workflow file to run.
Supported:
  - JSON (.json)"#
        )]
        workflow: file::ReadWorkflow,
    },
    /// Get node identity / information.
    Node {
        /// RPC host / port arguments.
        #[clap(flatten)]
        args: RpcArgs,
    },
    /// Get Homestar binary and other information.
    Info,
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Command::Init { .. } => "init",
            Command::Start { .. } => "start",
            Command::Stop { .. } => "stop",
            Command::Ping { .. } => "ping",
            Command::Run { .. } => "run",
            Command::Node { .. } => "node",
            Command::Info => "info",
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
            Command::Node { args } => {
                let response = rt.block_on(async {
                    let client = args.client().await?;
                    let response = client.node_info().await??;
                    Ok::<response::AckNodeInfo, Error>(response)
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
