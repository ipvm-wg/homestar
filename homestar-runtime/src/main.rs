use clap::Parser;
use homestar_runtime::{
    cli::{Cli, Command},
    daemon,
    db::Database,
    Db, FileLogger, Logger, Runner, Settings,
};
use miette::Result;
use tracing::info;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
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
                FileLogger::init(daemon_dir, &settings.monitoring())
            } else {
                Logger::init(&settings.monitoring())
            };

            info!(
                subject = "settings",
                category = "homestar_init",
                "starting with settings: {:?}",
                settings,
            );

            let db = Db::setup_connection_pool(settings.node(), database_url)
                .expect("to setup database pool");

            info!(
                subject = "database",
                category = "homestar_init",
                "starting with database: {}",
                Db::url().expect("database url to be provided"),
            );

            info!("starting Homestar runtime...");
            Runner::start(settings, db).expect("Failed to start runtime")
        }
        cmd => cmd.handle_rpc_command()?,
    }
    Ok(())
}
