use std::io::{stdout, IsTerminal};

use clap::Parser;
use homestar_runtime::{
    cli::{handle_init_command, Cli, Command, ConsoleTable, KeyArg, OutputMode},
    daemon,
    db::Database,
    runner::response,
    Db, FileLogger, Logger, Runner, Settings,
};
use miette::{miette, Result};
use tracing::info;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init {
            output_path,
            dry_run,
            quiet,
            force,
            no_input,
            key_type,
            key_file,
            key_seed,
        } => {
            let output_mode = if dry_run {
                OutputMode::StdOut
            } else {
                OutputMode::File {
                    path: output_path.unwrap_or_else(Settings::path),
                    force,
                }
            };

            let key_arg = key_file
                .map(|key_file| KeyArg::File { path: key_file })
                .or_else(|| key_seed.map(|key_seed| KeyArg::Seed { seed: key_seed }));

            // Run non-interactively if the input device is not a TTY
            // or if the `--no-input` flag is passed.
            let no_input = no_input || !stdout().is_terminal();

            handle_init_command(output_mode, key_arg, key_type, quiet, no_input)?
        }
        Command::Start {
            runtime_config,
            daemonize,
            daemon_dir,
            database_url,
        } => {
            // Load settings first, so we can daemonize before starting the
            // runtime.
            let settings = if let Some(file) = runtime_config {
                Settings::load_from_file(file)
            } else {
                Settings::load()
            }
            .expect("runtime settings to be loaded");

            let _guard = if daemonize {
                daemon::start(daemon_dir.clone())
                    .expect("runner to be started as a daemon process");
                FileLogger::init(daemon_dir, settings.node().monitoring())
            } else {
                Logger::init(settings.node().monitoring())
            };

            info!(
                subject = "settings",
                category = "homestar.init",
                "starting with settings: {:?}",
                settings,
            );

            let db = Db::setup_connection_pool(settings.node(), database_url)
                .expect("to setup database pool");

            info!(
                subject = "database",
                category = "homestar.init",
                "starting with database: {}",
                Db::url().expect("database url to be provided"),
            );

            info!("starting Homestar runtime...");
            Runner::start(settings, db).expect("Failed to start runtime")
        }
        Command::Info => {
            let response = response::Info::default();
            response
                .echo_table()
                .map_err(|_| miette!("failed to extract binary information"))?
        }
        cmd => cmd.handle_rpc_command()?,
    }
    Ok(())
}
