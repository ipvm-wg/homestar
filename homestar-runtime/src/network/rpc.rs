//! RPC server implementation.

use crate::{
    channel::{AsyncBoundedChannel, AsyncBoundedChannelReceiver, AsyncBoundedChannelSender},
    runner::{self, file::ReadWorkflow, response, RpcSender},
    settings,
};
use faststr::FastStr;
use futures::{future, StreamExt};
use std::{io, net::SocketAddr, sync::Arc, time::Duration};
use stream_cancel::Valved;
use tarpc::{
    client::{self, RpcError},
    context,
    server::{self, incoming::Incoming, Channel},
};
use tokio::{runtime::Handle, select, time};
use tokio_serde::formats::MessagePack;
use tracing::{info, warn};

mod error;
pub use error::Error;

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
    GracefulShutdown(AsyncBoundedChannelSender<()>),
    /// Message sent to start a [Workflow] run by reading a [Workflow] file.
    ///
    /// [Workflow]: homestar_core::Workflow
    Run((Option<FastStr>, ReadWorkflow)),
    /// Acknowledgement of a [Workflow] run.
    ///
    /// [Workflow]: homestar_core::Workflow
    RunAck(Box<response::AckWorkflow>),
    /// Error attempting to run a [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    RunErr(runner::Error),
    /// For skipping server messages.
    Skip,
}

/// RPC interface definition for CLI-server interaction.
#[tarpc::service]
pub(crate) trait Interface {
    /// Returns a greeting for name.
    async fn run(
        name: Option<FastStr>,
        workflow_file: ReadWorkflow,
    ) -> Result<Box<response::AckWorkflow>, Error>;
    /// Ping the server.
    async fn ping() -> String;
    /// Stop the server.
    async fn stop() -> Result<(), Error>;
}

/// RPC server state information.
#[derive(Debug, Clone)]
pub(crate) struct Server {
    /// [SocketAddr] of the RPC server.
    pub(crate) addr: SocketAddr,
    /// Sender for messages to be sent to the RPC server.
    pub(crate) sender: Arc<AsyncBoundedChannelSender<ServerMessage>>,
    /// Receiver for messages sent to the RPC server.
    pub(crate) receiver: AsyncBoundedChannelReceiver<ServerMessage>,
    /// Sender for messages to be sent to the [Runner].
    ///
    /// [Runner]: crate::Runner
    pub(crate) runner_sender: Arc<RpcSender>,
    /// Maximum number of connections to the RPC server.
    pub(crate) max_connections: usize,
    /// Timeout for the RPC server.
    pub(crate) timeout: Duration,
}

/// RPC client wrapper.
#[derive(Debug, Clone)]
pub struct Client {
    cli: InterfaceClient,
    addr: SocketAddr,
    ctx: context::Context,
}

/// RPC server state information.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ServerHandler {
    addr: SocketAddr,
    runner_sender: Arc<RpcSender>,
    timeout: Duration,
}

impl ServerHandler {
    fn new(addr: SocketAddr, runner_sender: Arc<RpcSender>, timeout: Duration) -> Self {
        Self {
            addr,
            runner_sender,
            timeout,
        }
    }
}

#[tarpc::server]
impl Interface for ServerHandler {
    async fn run(
        self,
        _: context::Context,
        name: Option<FastStr>,
        workflow_file: ReadWorkflow,
    ) -> Result<Box<response::AckWorkflow>, Error> {
        let (tx, rx) = AsyncBoundedChannel::oneshot();
        self.runner_sender
            .send_async((ServerMessage::Run((name, workflow_file)), Some(tx)))
            .await
            .map_err(|e| Error::FailureToSendOnChannel(e.to_string()))?;

        let now = time::Instant::now();
        select! {
            Ok(msg) = rx.recv_async() => {
                match msg {
                    ServerMessage::RunAck(response) => {
                        Ok(response)
                    }
                    ServerMessage::RunErr(err) => Err(err).map_err(|e| Error::FromRunner(e.to_string()))?,
                    _ => Err(Error::FailureToSendOnChannel("unexpected message".into())),
                }
            },
            _ = time::sleep_until(now + self.timeout) => {
                let s = format!("server timeout of {} ms reached", self.timeout.as_millis());
                info!("{s}");
                Err(Error::FailureToReceiveOnChannel(s))
            }

        }
    }
    async fn ping(self, _: context::Context) -> String {
        "pong".into()
    }
    async fn stop(self, _: context::Context) -> Result<(), Error> {
        self.runner_sender
            .send_async((ServerMessage::ShutdownCmd, None))
            .await
            .map_err(|e| Error::FailureToSendOnChannel(e.to_string()))
    }
}

impl Server {
    /// Create a new instance of the RPC server.
    pub(crate) fn new(settings: &settings::Network, runner_sender: Arc<RpcSender>) -> Self {
        let (tx, rx) = AsyncBoundedChannel::oneshot();
        Self {
            addr: SocketAddr::new(settings.rpc_host, settings.rpc_port),
            sender: tx.into(),
            receiver: rx,
            runner_sender,
            max_connections: settings.rpc_max_connections,
            timeout: settings.rpc_server_timeout,
        }
    }

    /// Return a RPC server channel sender.
    pub(crate) fn sender(&self) -> Arc<AsyncBoundedChannelSender<ServerMessage>> {
        self.sender.clone()
    }

    /// Start the RPC server and connect the client.
    pub(crate) async fn spawn(self) -> anyhow::Result<()> {
        let mut listener =
            tarpc::serde_transport::tcp::listen(self.addr, MessagePack::default).await?;
        listener.config_mut().max_frame_length(usize::MAX);

        info!("RPC server listening on {}", self.addr);

        // setup valved listener for cancellation
        let (exit, incoming) = Valved::new(listener);

        let runtime_handle = Handle::current();
        runtime_handle.spawn(async move {
            let fut = incoming
                // Ignore accept errors.
                .filter_map(|r| future::ready(r.ok()))
                .map(server::BaseChannel::with_defaults)
                // Limit channels to 1 per IP.
                .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap_or(self.addr).ip())
                .map(|channel| {
                    let handler =
                        ServerHandler::new(self.addr, self.runner_sender.clone(), self.timeout);
                    channel.execute(handler.serve())
                })
                .buffer_unordered(self.max_connections)
                .for_each(|_| async {});

            select! {
                Ok(ServerMessage::GracefulShutdown(tx)) = self.receiver.recv_async() => {
                    info!("RPC server shutting down");
                    drop(exit);
                    let _ = tx.send(());
                }
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
    pub async fn new(addr: SocketAddr, ctx: context::Context) -> Result<Self, io::Error> {
        let transport = tarpc::serde_transport::tcp::connect(addr, MessagePack::default).await?;
        let client = InterfaceClient::new(client::Config::default(), transport).spawn();
        Ok(Client {
            cli: client,
            addr,
            ctx,
        })
    }

    /// Return the [SocketAddr] of the RPC server.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Ping the server.
    pub async fn ping(&self) -> Result<String, RpcError> {
        self.cli.ping(self.ctx).await
    }

    /// Stop the server.
    pub async fn stop(&self) -> Result<Result<(), Error>, RpcError> {
        self.cli.stop(self.ctx).await
    }

    /// Run a [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    pub async fn run(
        &self,
        name: Option<FastStr>,
        workflow_file: ReadWorkflow,
    ) -> Result<Result<Box<response::AckWorkflow>, Error>, RpcError> {
        self.cli.run(self.ctx, name, workflow_file).await
    }
}
