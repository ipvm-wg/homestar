use anyhow::Result;
use clap::Parser;
#[cfg(feature = "ipfs")]
use homestar_runtime::network::ipfs::IpfsCli;
use homestar_runtime::{
    cli::{Args, Argument},
    db::{Database, Db},
    logger,
    network::{eventloop::EventLoop, swarm, ws::WebSocket},
    Settings,
};
use std::sync::Arc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let (stdout_writer, _stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
    logger::init(stdout_writer)?;

    let opts = Args::parse();

    #[cfg(feature = "ipfs")]
    let ipfs = IpfsCli::default();

    match opts.argument {
        Argument::Run { runtime_config } => {
            let settings = if let Some(file) = runtime_config {
                Settings::load_from_file(file)
            } else {
                Settings::load()
            }?;

            let db = Db::setup_connection_pool(settings.node())?;
            let swarm = swarm::new(settings.node()).await?;

            let (_tx, rx) = EventLoop::setup_channel(settings.node());
            // instantiate and start event-loop for events
            let eventloop = EventLoop::new(swarm, rx, settings.node());

            #[cfg(not(feature = "ipfs"))]
            tokio::spawn(eventloop.run(db));

            #[cfg(feature = "ipfs")]
            tokio::spawn(eventloop.run(db, ipfs));

            let (ws_tx, ws_rx) = WebSocket::setup_channel(settings.node());
            let ws_sender = Arc::new(ws_tx);
            let ws_receiver = Arc::new(ws_rx);
            WebSocket::start_server(ws_sender, ws_receiver, settings.node()).await?;
        }
    }

    Ok(())
}
