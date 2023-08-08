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
            let settings = if let Some(file) = runtime_config {
                Settings::load_from_file(file)
            } else {
                Settings::load()
            }
            .expect("Failed to load settings");

            let _guard = if daemonize {
                daemon::start(daemon_dir.clone()).expect("Failed to daemonize homestar runner");
                FileLogger::init(daemon_dir)
            } else {
                Logger::init()
            };

            info!("starting with settings: {:?}", settings,);
            Db::set_url(database_url).expect("Failed to set DB url");
            let db = Db::setup_connection_pool(settings.node()).expect("Failed to setup DB pool");

            info!("starting Homestar runtime...");
            Runner::start(settings, db).expect("Failed to start runtime")
        }
        cmd => cmd.handle_rpc_command()?,
    }
    Ok(())
}
