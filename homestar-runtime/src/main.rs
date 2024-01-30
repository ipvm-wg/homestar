use std::{
    fs::File,
    io::{self, ErrorKind, Write},
};

use clap::Parser;
use homestar_runtime::{
    cli::{Cli, Command, ConsoleTable},
    daemon,
    db::Database,
    runner::response,
    Db, FileLogger, Logger, Runner, Settings,
};
use inquire::Confirm;
use miette::{miette, Result};
use tracing::info;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init {
            runtime_config,
            dry_run,
            ..
        } => {
            let mut settings_sink: Box<dyn Write> = if dry_run {
                Box::new(io::stdout())
            } else {
                let settings_path = runtime_config.unwrap_or_else(Settings::path);
                let settings_file = File::options()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(&settings_path);

                // This seemingly convoluted match is to avoid the risk of a
                // TOCTOU race condition, where another process creates the file
                // in between this one checking for its existence and opening it.
                //
                // TODO: there should probably be a flag for non-interactive use
                // that automatically overwrites the file.
                let settings_file = match settings_file {
                    Ok(file) => file,
                    Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                        let should_overwrite = Confirm::new(&format!(
                            "Settings file already exists at {:?}, overwrite?",
                            settings_path
                        ))
                        .with_default(false)
                        .prompt()
                        .expect("to prompt for overwrite");

                        if !should_overwrite {
                            println!("Aborting.");
                            return Ok(());
                        }

                        File::options()
                            .read(true)
                            .write(true)
                            .create_new(false)
                            .open(&settings_path)
                            .expect("to open settings file")
                    }
                    err => err.expect("to open settings file"),
                };

                println!("Writing settings to {:?}", settings_path);

                Box::new(settings_file)
            };

            let settings = Settings::default();
            let settings_toml = toml::to_string_pretty(&settings).expect("to serialize settings");

            settings_sink
                .write_all(settings_toml.as_bytes())
                .expect("to write settings file");
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
