//! RPC server implementation.

use crate::{
    channel::{BoundedChannel, BoundedChannelReceiver, BoundedChannelSender},
    settings,
};
use futures::{future, StreamExt};
use std::{io, net::SocketAddr, path::PathBuf, sync::Arc};
use stream_cancel::Valved;
use tarpc::{
    client::{self, RpcError},
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Bincode,
};
use tokio::{
    runtime::Handle,
    select,
    sync::{mpsc, oneshot},
};
use tracing::{info, warn};

/// Message type for messages sent back from the
/// websocket server to the [runner] for example.
///
/// [runner]: crate::Runner
#[derive(Debug)]
pub(crate) enum ServerMessage {
    /// Notify the [Runner] that the RPC server was given a `stop` command.
    ///
    /// [Runner]: crate::Runner
    ShutdownCmd,
    /// Message sent by the [Runner] to start a graceful shutdown.
    ///
    /// [Runner]: crate::Runner
    GracefulShutdown(oneshot::Sender<()>),
}

/// RPC interface definition for CLI-server interaction.
#[tarpc::service]
pub(crate) trait Interface {
    /// Returns a greeting for name.
    async fn run(workflow_file: PathBuf);
    /// Ping the server.
    async fn ping() -> String;
    /// Stop the server.
    async fn stop() -> Result<(), String>;
}

/// RPC server state information.
#[derive(Debug, Clone)]
pub(crate) struct Server {
    /// [SocketAddr] of the RPC server.
    pub(crate) addr: SocketAddr,
    /// Sender for messages to be sent to the RPC server.
    pub(crate) sender: Arc<BoundedChannelSender<ServerMessage>>,
    /// Receiver for messages sent to the RPC server.
    pub(crate) receiver: BoundedChannelReceiver<ServerMessage>,
    /// Sender for messages to be sent to the [Runner].
    ///
    /// [Runner]: crate::Runner
    pub(crate) runner_sender: Arc<mpsc::Sender<ServerMessage>>,
    /// Maximum number of connections to the RPC server.
    pub(crate) max_connections: usize,
}

/// RPC client wrapper.
#[derive(Debug, Clone)]
pub struct Client(InterfaceClient);

/// RPC server state information.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ServerHandler {
    addr: SocketAddr,
    runner_sender: Arc<mpsc::Sender<ServerMessage>>,
}

impl ServerHandler {
    fn new(addr: SocketAddr, runner_sender: Arc<mpsc::Sender<ServerMessage>>) -> Self {
        Self {
            addr,
            runner_sender,
        }
    }
}

#[tarpc::server]
impl Interface for ServerHandler {
    async fn run(self, _: context::Context, _workflow_file: PathBuf) {}
    async fn ping(self, _: context::Context) -> String {
        "pong".into()
    }
    async fn stop(self, _: context::Context) -> Result<(), String> {
        let _ = self.runner_sender.send(ServerMessage::ShutdownCmd).await;
        Ok(())
    }
}

impl Server {
    /// Create a new instance of the RPC server.
    pub(crate) fn new(
        settings: settings::Network,
        runner_sender: Arc<mpsc::Sender<ServerMessage>>,
    ) -> Self {
        let (tx, rx) = BoundedChannel::oneshot();
        Self {
            addr: SocketAddr::new(settings.rpc_host, settings.rpc_port),
            sender: tx.into(),
            receiver: rx,
            runner_sender,
            max_connections: settings.rpc_max_connections,
        }
    }

    /// Return a RPC server channel sender.
    pub(crate) fn sender(&self) -> Arc<BoundedChannelSender<ServerMessage>> {
        self.sender.clone()
    }

    /// Start the RPC server and connect the client.
    pub(crate) async fn spawn(self, runtime_handle: Handle) -> anyhow::Result<()> {
        let mut listener = tarpc::serde_transport::tcp::listen(self.addr, Bincode::default).await?;
        listener.config_mut().max_frame_length(usize::MAX);

        info!("RPC server listening on {}", self.addr);

        // setup valved listener for cancellation
        let (exit, incoming) = Valved::new(listener);

        runtime_handle.spawn(async move {
            let fut = incoming
                // Ignore accept errors.
                .filter_map(|r| future::ready(r.ok()))
                .map(server::BaseChannel::with_defaults)
                // Limit channels to 1 per IP.
                .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap_or(self.addr).ip())
                .map(|channel| {
                    let handler = ServerHandler::new(self.addr, self.runner_sender.clone());
                    channel.execute(handler.serve())
                })
                .buffer_unordered(self.max_connections)
                .for_each(|_| async {});

            select! {
                biased;
                Ok(msg) = tokio::task::spawn_blocking(move || self.receiver.recv()) =>
                    if let Ok(ServerMessage::GracefulShutdown(tx)) = msg {
                        info!("RPC server shutting down");
                        drop(exit);
                        let _ = tx.send(());
                    },
                _ = fut => warn!("RPC server exited unexpectedly"),
            }
        });

        Ok(())
    }
}

impl Client {
    /// Instantiate a new [Client] with a [tcp] connection to a running Homestar
    /// runner/server.
    ///
    /// [tcp]: tarpc::serde_transport::tcp
    pub async fn new(addr: SocketAddr) -> Result<Self, io::Error> {
        let transport = tarpc::serde_transport::tcp::connect(addr, Bincode::default).await?;
        let client = InterfaceClient::new(client::Config::default(), transport).spawn();
        Ok(Client(client))
    }

    /// Ping the server.
    pub async fn ping(&self) -> Result<String, RpcError> {
        self.0.ping(context::current()).await
    }

    /// Stop the server.
    pub async fn stop(&self) -> Result<Result<(), String>, RpcError> {
        self.0.stop(context::current()).await
    }
}
