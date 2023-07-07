use anyhow::Result;
use clap::Parser;
#[cfg(feature = "websocket-server")]
use homestar_runtime::ws;
use homestar_runtime::{
    cli::{Cli, Command},
    db::Database,
    logger, Db, Runner, Settings,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::{runtime, select, time};
use tracing::info;

fn main() {
    let (stdout_writer, _stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
    logger::init(stdout_writer).expect("Failed to initialize logger");

    let cli = Cli::parse();

    let settings = if let Some(file) = cli.runtime_config {
        Settings::load_from_file(file)
    } else {
        Settings::load()
    }
    .expect("Failed to load settings");

    info!("starting with settings: {:?}", settings,);

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("runtime-{}", id)
        })
        .build()
        .expect("Failed to start multi-threaded runtime");

    let db = Db::setup_connection_pool(settings.node())
        .expect("Failed to setup database connection pool");

    match cli.command {
        Command::Start => {
            runtime
                .block_on(runner(Arc::new(settings), db))
                .expect("Failed to run initialization");
        }
    }

    drop(runtime);
}

async fn runner(settings: Arc<Settings>, db: impl Database + 'static) -> Result<()> {
    let mut runner = Runner::start(settings.clone(), db).await?;

    loop {
        select! {
            biased;
            Ok(_event) = runner.command_receiver() => info!("Connected to the Network"),
            _ = Runner::shutdown_signal() => {
                info!("gracefully shutting down runner");
                let drain_timeout = time::Instant::now() + settings.node().shutdown_timeout();

                select! {
                    Ok(()) = runner.shutdown() => {
                        #[cfg(feature = "websocket-server")]
                        match runner.ws_receiver().recv() {
                            Ok(ws::WsMessage::GracefulShutdown) => (),
                            Err(err) => info!(error=?err, "runner shutdown complete, but with error"),
                        }
                        info!("runner shutdown complete");
                        break;
                    },
                    _ = time::sleep_until(drain_timeout) => {
                        info!("shutdown timeout reached, shutting down runner anyway");
                        break;
                    }
                }
            }
        }
    }

    //drop(db);

    Ok(())
}
