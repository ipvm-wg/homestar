//! General [Runner] interface for working across multiple workers
//! and executing workflows.

#[cfg(feature = "websocket-server")]
use crate::network::ws;
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    channel::BoundedChannelSender,
    db::Database,
    event_handler::{Event, EventHandler},
    network::{rpc, swarm},
    Settings,
};
use anyhow::Result;
use dashmap::DashMap;
use libipld::Cid;
#[cfg(not(test))]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::{
    runtime, select,
    signal::unix::{signal, SignalKind},
    sync::{mpsc, oneshot},
    task::{AbortHandle, JoinHandle},
    time,
};
use tokio_util::time::DelayQueue;
use tracing::info;

#[cfg(not(test))]
const HOMESTAR_THREAD: &str = "homestar-runtime";

/// Type alias for a [DashMap] containing running worker [JoinHandle]s.
pub type RunningWorkerSet = DashMap<Cid, JoinHandle<Result<()>>>;

/// Type alias for a [DashMap] containing running task [AbortHandle]s.
pub type RunningTaskSet = DashMap<Cid, Vec<AbortHandle>>;

/// Trait for managing a [DashMap] of running task information.
pub trait ModifiedSet {
    /// Append or insert a new [AbortHandle] into the [RunningTaskSet].
    fn append_or_insert(&mut self, cid: Cid, handles: Vec<AbortHandle>);
}

impl ModifiedSet for RunningTaskSet {
    fn append_or_insert(&mut self, cid: Cid, mut handles: Vec<AbortHandle>) {
        self.entry(cid)
            .and_modify(|prev_handles| {
                prev_handles.append(&mut handles);
            })
            .or_insert_with(|| handles);
    }
}

/// Runner interface.
/// Used to manage workers and execute/run [Workflows].
///
/// [Workflows]: homestar_core::Workflow
#[cfg(feature = "websocket-server")]
#[allow(dead_code)]
#[derive(Debug)]
pub struct Runner {
    message_buffer_len: usize,
    event_sender: Arc<mpsc::Sender<Event>>,
    expiration_queue: DelayQueue<Cid>,
    running_tasks: RunningTaskSet,
    running_workers: RunningWorkerSet,
    runtime: tokio::runtime::Runtime,
    ws_msg_sender: Arc<ws::Sender>,
    ws_mpsc_sender: mpsc::Sender<ws::Message>,
}

/// Runner interface.
/// Used to manage workers and execute/run [Workflows].
///
/// [Workflows]: homestar_core::Workflow
#[cfg(not(feature = "websocket-server"))]
#[allow(dead_code)]
#[derive(Debug)]
pub struct Runner {
    message_buffer_len: usize,
    event_sender: Arc<mpsc::Sender<Event>>,
    expiration_queue: DelayQueue<Cid>,
    running_tasks: RunningTaskSet,
    running_workers: RunningWorkerSet,
    runtime: tokio::runtime::Runtime,
}

impl Runner {
    /// Setup bounded, MPSC channel for top-level RPC communication.
    pub(crate) fn setup_channel(
        capacity: usize,
    ) -> (
        mpsc::Sender<rpc::ServerMessage>,
        mpsc::Receiver<rpc::ServerMessage>,
    ) {
        mpsc::channel(capacity)
    }

    /// Initialize and start the Homestar [Runner] / runtime.
    #[cfg(not(test))]
    pub fn start(settings: Arc<Settings>, db: impl Database + 'static) -> Result<Runner> {
        let runtime = runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name_fn(|| {
                static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
                let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
                format!("{HOMESTAR_THREAD}-{id}")
            })
            .build()?;

        let runner = Self::init(settings, db, runtime)?;

        Ok(runner)
    }

    /// Initialize and start the Homestar [Runner] / runtime.
    #[cfg(test)]
    pub fn start(settings: Arc<Settings>, db: impl Database + 'static) -> Result<Runner> {
        let runtime = runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let runner = Self::init(settings, db, runtime)?;

        Ok(runner)
    }

    /// Listen loop for [Runner] signals and messages.
    pub fn serve(self, settings: Arc<Settings>) -> Result<()> {
        let (tx, mut rx) = Self::setup_channel(self.message_buffer_len);
        let shutdown_timeout = settings.node.shutdown_timeout;
        let rpc_server = rpc::Server::new(settings.node.network.clone(), tx.into());
        let rpc_sender = rpc_server.sender();
        self.runtime
            .block_on(rpc_server.spawn(self.runtime.handle().clone()))?;

        let shutdown_time_left = self.runtime.block_on(async {
            loop {
                select! {
                    biased;
                    // Duplicate inner-shutdown code here, as tokio::select!
                    // doesn't allow for either-or patterns like matches.
                    Some(rpc::ServerMessage::ShutdownCmd) = rx.recv() => {
                            info!("RPC shutdown signal received, shutting down runner");
                            let now = time::Instant::now();
                            let drain_timeout = now + shutdown_timeout;
                            select! {
                                Ok(()) = self.shutdown(rpc_sender) => {
                                    break now.elapsed();
                                },
                                _ = time::sleep_until(drain_timeout) => {
                                    info!("shutdown timeout reached, shutting down runner anyway");
                                    break now.elapsed();
                                }
                            }
                    },
                    _ = Self::shutdown_signal() => {
                        info!("gracefully shutting down runner");

                        let now = time::Instant::now();
                        let drain_timeout = now + shutdown_timeout;
                        select! {
                            Ok(()) = self.shutdown(rpc_sender) => {
                                break now.elapsed();
                            },
                            _ = time::sleep_until(drain_timeout) => {
                                info!("shutdown timeout reached, shutting down runner anyway");
                                break now.elapsed();
                            }
                        }
                    }
                }
            }
        });

        if shutdown_time_left < shutdown_timeout {
            self.runtime
                .shutdown_timeout(shutdown_timeout - shutdown_time_left);
            info!("runner shutdown complete");
        }

        Ok(())
    }

    /// [mpsc::Sender] of the event-handler.
    ///
    /// [EventHandler]: crate::EventHandler
    pub fn event_sender(&self) -> Arc<mpsc::Sender<Event>> {
        self.event_sender.clone()
    }

    /// [tokio::sync::broadcast::Sender] for sending messages through the
    /// webSocket server to subscribers.
    #[cfg(feature = "websocket-server")]
    pub fn ws_msg_sender(&self) -> &ws::Sender {
        &self.ws_msg_sender
    }

    /// Garbage-collect task [AbortHandle]s in the [RunningTaskSet] and
    /// workers in the [RunningWorkerSet].
    #[allow(dead_code)]
    pub(crate) fn gc(&mut self) {
        self.running_tasks.retain(|_cid, handles| {
            handles.retain(|handle| !handle.is_finished());
            !handles.is_empty()
        });

        self.running_workers
            .retain(|_cid, handle| !handle.is_finished());
    }

    /// Garbage-collect task [AbortHandle]s in the [RunningTaskSet] and a
    /// worker's [JoinHandle] in the [RunningWorkerSet] for a specific workflow
    /// [Cid], running on a worker.
    #[allow(dead_code)]
    pub(crate) fn gc_worker(&mut self, cid: Cid) {
        if let Some(mut handles) = self.running_tasks.get_mut(&cid) {
            handles.retain(|handle| !handle.is_finished());
        }

        self.running_tasks
            .retain(|_cid, handles| !handles.is_empty());

        if let Some(handle) = self.running_workers.get_mut(&cid) {
            if handle.is_finished() {
                self.running_workers.remove(&cid);
            }
        }
    }

    /// Abort all workers.
    #[allow(dead_code)]
    pub(crate) fn abort_workers(&mut self) {
        self.running_workers
            .iter_mut()
            .for_each(|handle| handle.abort());
    }

    /// Abort a specific worker given a [Cid].
    #[allow(dead_code)]
    pub(crate) fn abort_worker(&mut self, cid: Cid) {
        if let Some(handle) = self.running_workers.get_mut(&cid) {
            handle.abort()
        }
    }

    /// Abort all tasks running within all workers.
    #[allow(dead_code)]
    pub(crate) fn abort_tasks(&mut self) {
        self.running_tasks.iter_mut().for_each(|handles| {
            for abort_handle in &*handles {
                abort_handle.abort();
            }
        });
    }

    /// Abort a specific worker's tasks given a [Cid].
    #[allow(dead_code)]
    pub(crate) fn abort_worker_tasks(&mut self, cid: Cid) {
        if let Some(handles) = self.running_tasks.get_mut(&cid) {
            for abort_handle in &*handles {
                abort_handle.abort();
            }
        }
    }

    /// Captures shutdown signals for [Runner].
    async fn shutdown_signal() -> Result<()> {
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;

        select! {
            _ = tokio::signal::ctrl_c() => info!("CTRL-C received, shutting down"),
            _ = sigint.recv() => info!("SIGINT received, shutting down"),
            _ = sigterm.recv() => info!("SIGTERM received, shutting down"),
        }

        Ok(())
    }

    /// Sequence for shutting down a [Runner], including:
    /// a) event-handler channels,
    /// b) Running workers
    /// c) [Runner] channels.
    async fn shutdown(
        &self,
        rpc_sender: Arc<BoundedChannelSender<rpc::ServerMessage>>,
    ) -> Result<()> {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        rpc_sender.try_send(rpc::ServerMessage::GracefulShutdown(shutdown_sender))?;
        shutdown_receiver.await?;

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        self.event_sender
            .send(Event::Shutdown(shutdown_sender))
            .await?;
        shutdown_receiver.await?;

        #[cfg(feature = "websocket-server")]
        {
            let (shutdown_sender, shutdown_receiver) = oneshot::channel();
            self.ws_mpsc_sender
                .send(ws::Message::GracefulShutdown(shutdown_sender))
                .await?;
            shutdown_receiver.await?;
        }

        // TODO: shutdown workers

        Ok(())
    }

    fn init(
        settings: Arc<Settings>,
        db: impl Database + 'static,
        runtime: tokio::runtime::Runtime,
    ) -> Result<Runner> {
        let swarm = runtime.block_on(swarm::new(settings.node()))?;

        let event_handler = EventHandler::new(swarm, db, settings.node());
        let event_sender = event_handler.sender();

        #[cfg(feature = "ipfs")]
        let _event_handler_hdl = runtime.spawn({
            let ipfs = IpfsCli::default();
            event_handler.start(ipfs)
        });

        #[cfg(not(feature = "ipfs"))]
        let _event_handler_hdl = runtime.spawn(event_handler.start());

        #[cfg(feature = "websocket-server")]
        {
            // Setup websocket communication.
            let ws_server = ws::Server::new(settings.node.network.clone())?;
            let ws_msg_tx = ws_server.sender();

            let (ws_tx, ws_rx) = mpsc::channel(settings.node.network.websocket_capacity);
            let _ws_hdl = runtime.spawn(ws_server.start(ws_rx));

            Ok(Self {
                message_buffer_len: settings.node.network.events_buffer_len,
                event_sender,
                expiration_queue: DelayQueue::new(),
                running_tasks: DashMap::new(),
                running_workers: DashMap::new(),
                runtime,
                ws_msg_sender: ws_msg_tx,
                ws_mpsc_sender: ws_tx,
            })
        }

        #[cfg(not(feature = "websocket-server"))]
        Ok(Self {
            message_buffer_len: settings.node.network.events_buffer_len,
            event_sender,
            expiration_queue: DelayQueue::new(),
            running_tasks: DashMap::new(),
            running_workers: DashMap::new(),
            runtime,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::network::rpc::Client;
    use homestar_core::test_utils;
    use rand::thread_rng;
    use std::net::SocketAddr;
    use tokio::net::TcpStream;

    fn setup() -> (Runner, Settings) {
        let mut settings = Settings::load().unwrap();
        settings.node.network.websocket_port = test_utils::ports::get_port() as u16;
        settings.node.network.rpc_port = test_utils::ports::get_port() as u16;
        let db = crate::test_utils::db::MemoryDb::setup_connection_pool(&settings.node).unwrap();

        let runner = Runner::start(settings.clone().into(), db).unwrap();
        (runner, settings)
    }

    #[test]
    fn shutdown() {
        let (runner, settings) = setup();

        let (tx, _rx) = Runner::setup_channel(1);
        let rpc_server = rpc::Server::new(settings.node.network.clone(), Arc::new(tx));
        let rpc_sender = rpc_server.sender();

        let addr = SocketAddr::new(
            settings.node.network.rpc_host,
            settings.node.network.rpc_port,
        );

        runner.runtime.block_on(async {
            rpc_server
                .spawn(runner.runtime.handle().clone())
                .await
                .unwrap();

            let _stream = TcpStream::connect(addr).await.expect("Connection error");
            let _another_stream = TcpStream::connect(addr).await.expect("Connection error");
        });

        runner.runtime.block_on(async {
            match runner.shutdown(rpc_sender).await {
                Ok(()) => {
                    // with shutdown, we should not be able to connect to the server(s)
                    let stream_error = TcpStream::connect(addr).await;
                    assert!(stream_error.is_err());
                    assert!(matches!(
                        stream_error.unwrap_err().kind(),
                        std::io::ErrorKind::ConnectionRefused
                    ));

                    #[cfg(feature = "websocket-server")]
                    {
                        let ws_error =
                            tokio_tungstenite::connect_async("ws://localhost:1337".to_string())
                                .await;
                        assert!(ws_error.is_err());
                    }
                }
                _ => panic!("Shutdown failed."),
            }
        });
    }

    #[test]
    fn spawn_rpc_server_and_ping() {
        let (runner, settings) = setup();

        let (tx, _rx) = Runner::setup_channel(1);
        let rpc_server = rpc::Server::new(settings.node.network.clone(), Arc::new(tx));

        runner
            .runtime
            .block_on(rpc_server.spawn(runner.runtime.handle().clone()))
            .unwrap();

        runner.runtime.spawn(async move {
            let addr = SocketAddr::new(
                settings.node.network.rpc_host,
                settings.node.network.rpc_port,
            );

            let client = Client::new(addr).await.unwrap();
            let response = client.ping().await.unwrap();
            assert_eq!(response, "pong".to_string());
        });
    }

    #[test]
    fn abort_all_tasks() {
        let (mut runner, _) = setup();
        let mut set = tokio::task::JoinSet::new();

        runner.runtime.block_on(async {
            for i in 0..3 {
                let handle = set.spawn(async move { i });
                runner.running_tasks.append_or_insert(
                    test_utils::cid::generate_cid(&mut thread_rng()),
                    vec![handle],
                );
            }

            while set.join_next().await.is_some() {}
        });

        runner.abort_tasks();
        assert!(!runner.running_tasks.is_empty());

        runner.gc();
        assert!(runner.running_tasks.is_empty());
    }

    #[test]
    fn abort_one_task() {
        let (mut runner, _) = setup();
        let mut set = tokio::task::JoinSet::new();
        let mut cids = vec![];
        runner.runtime.block_on(async {
            for i in 0..3 {
                let handle = set.spawn(async move { i });
                let cid = test_utils::cid::generate_cid(&mut thread_rng());
                runner.running_tasks.append_or_insert(cid, vec![handle]);
                cids.push(cid);
            }

            while set.join_next().await.is_some() {}
        });

        runner.abort_worker_tasks(cids[0]);
        assert!(runner.running_tasks.len() == 3);

        runner.gc_worker(cids[0]);
        assert!(runner.running_tasks.len() == 2);
    }
}
