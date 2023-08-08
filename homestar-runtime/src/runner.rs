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
    worker::WorkerMessage,
    Settings, Worker,
};
use anyhow::{anyhow, Context, Result};
use atomic_refcell::AtomicRefCell;
use dashmap::DashMap;
use futures::future::poll_fn;
use libipld::Cid;
#[cfg(not(test))]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{ops::ControlFlow, rc::Rc, sync::Arc, task::Poll};
use tokio::{
    runtime, select,
    signal::unix::{signal, SignalKind},
    sync::{mpsc, oneshot},
    task::{AbortHandle, JoinHandle},
    time,
};
use tokio_util::time::{delay_queue, DelayQueue};
use tracing::{error, info, warn};

mod error;
pub(crate) mod file;
pub(crate) mod response;
pub(crate) use error::Error;

#[cfg(not(test))]
const HOMESTAR_THREAD: &str = "homestar-runtime";

/// Type alias for a [DashMap] containing running worker [JoinHandle]s.
pub type RunningWorkerSet = DashMap<Cid, (JoinHandle<Result<()>>, delay_queue::Key)>;

/// Type alias for a [DashMap] containing running task [AbortHandle]s.
pub type RunningTaskSet = DashMap<Cid, Vec<AbortHandle>>;

/// Trait for managing a [DashMap] of running task information.
pub trait ModifiedSet {
    /// Append or insert a new [AbortHandle] into the [RunningTaskSet].
    fn append_or_insert(&self, cid: Cid, handles: Vec<AbortHandle>);
}

type RpcSender = mpsc::Sender<(
    rpc::ServerMessage,
    Option<oneshot::Sender<rpc::ServerMessage>>,
)>;

type RpcReceiver = mpsc::Receiver<(
    rpc::ServerMessage,
    Option<oneshot::Sender<rpc::ServerMessage>>,
)>;

impl ModifiedSet for RunningTaskSet {
    fn append_or_insert(&self, cid: Cid, mut handles: Vec<AbortHandle>) {
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
#[cfg_attr(docsrs, doc(cfg(feature = "websocket-server")))]
#[allow(dead_code)]
#[derive(Debug)]
pub struct Runner {
    message_buffer_len: usize,
    event_sender: Arc<mpsc::Sender<Event>>,
    expiration_queue: Rc<AtomicRefCell<DelayQueue<Cid>>>,
    running_tasks: Arc<RunningTaskSet>,
    running_workers: RunningWorkerSet,
    runtime: tokio::runtime::Runtime,
    settings: Arc<Settings>,
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
    expiration_queue: Rc<AtomicRefCell<DelayQueue<Cid>>>,
    running_tasks: Arc<RunningTaskSet>,
    running_workers: RunningWorkerSet,
    runtime: tokio::runtime::Runtime,
    settings: Arc<Settings>,
}

impl Runner {
    /// Setup bounded, MPSC channel for top-level RPC communication.
    pub(crate) fn setup_rpc_channel(capacity: usize) -> (RpcSender, RpcReceiver) {
        mpsc::channel(capacity)
    }

    /// Setup bounded, MPSC channel for top-level Worker communication.
    pub(crate) fn setup_worker_channel(
        capacity: usize,
    ) -> (mpsc::Sender<WorkerMessage>, mpsc::Receiver<WorkerMessage>) {
        mpsc::channel(capacity)
    }

    /// Initialize and start the Homestar [Runner] / runtime.
    #[cfg(not(test))]
    pub fn start(settings: Settings, db: impl Database + 'static) -> Result<()> {
        let runtime = runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name_fn(|| {
                static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
                let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
                format!("{HOMESTAR_THREAD}-{id}")
            })
            .build()?;

        Self::init(settings, db.clone(), runtime)?.serve(db)
    }

    /// Initialize and start the Homestar [Runner] / runtime.
    #[cfg(test)]
    pub fn start(settings: Settings, db: impl Database + 'static) -> Result<Runner> {
        let runtime = runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let runner = Self::init(settings, db, runtime)?;
        Ok(runner)
    }

    fn init(
        settings: Settings,
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
            let ws_server = ws::Server::new(settings.node().network())?;
            let ws_msg_tx = ws_server.sender();

            let (ws_tx, ws_rx) = mpsc::channel(settings.node.network.websocket_capacity);
            let _ws_hdl = runtime.spawn(ws_server.start(ws_rx));

            Ok(Self {
                message_buffer_len: settings.node.network.events_buffer_len,
                event_sender,
                expiration_queue: Rc::new(AtomicRefCell::new(DelayQueue::new())),
                running_tasks: DashMap::new().into(),
                running_workers: DashMap::new(),
                runtime,
                settings: settings.into(),
                ws_msg_sender: ws_msg_tx,
                ws_mpsc_sender: ws_tx,
            })
        }

        #[cfg(not(feature = "websocket-server"))]
        Ok(Self {
            message_buffer_len: settings.node.network.events_buffer_len,
            event_sender,
            expiration_queue: Rc::new(AtomicRefCell::new(DelayQueue::new())),
            running_tasks: DashMap::new().into(),
            running_workers: DashMap::new(),
            runtime,
            settings: settings.into(),
        })
    }

    /// Listen loop for [Runner] signals and messages.
    #[allow(dead_code)]
    fn serve(self, db: impl Database + 'static) -> Result<()> {
        let (rpc_tx, mut rpc_rx) = Self::setup_rpc_channel(self.message_buffer_len);
        let (runner_tx, mut runner_rx) = Self::setup_worker_channel(self.message_buffer_len);

        let shutdown_timeout = self.settings.node.shutdown_timeout;
        let rpc_server = rpc::Server::new(self.settings.node.network(), rpc_tx.into());
        let rpc_sender = rpc_server.sender();
        self.runtime.block_on(rpc_server.spawn())?;

        let shutdown_time_left = self.runtime.block_on(async {
            let mut gc_interval = tokio::time::interval(self.settings.node.gc_interval);
            loop {
                select! {
                    biased;
                    // Handle RPC messages.
                    Some((rpc_message, Some(oneshot_tx))) = rpc_rx.recv() => {
                        let now = time::Instant::now();
                        match self.handle_command_message(
                            rpc_message,
                            rpc_sender.clone(),
                            runner_tx.clone(),
                            db.clone(),
                            now
                        ).await {
                            Ok(ControlFlow::Break(())) => break now.elapsed(),
                            Ok(ControlFlow::Continue(rpc::ServerMessage::Skip)) => {},
                            Ok(ControlFlow::Continue(msg @ rpc::ServerMessage::RunAck(_))) => {
                                info!("sending message to rpc server");
                                let _ = oneshot_tx.send(msg);
                            },
                            Err(err) => {
                                error!("error handling rpc message: {}", err);
                                let _ = oneshot_tx.send(rpc::ServerMessage::RunErr(err.into()));
                            },
                             _ => {}
                        }
                    }
                    // Handle messages from the worker.
                    Some(msg) = runner_rx.recv() => {
                        match msg {
                            WorkerMessage::Dropped(cid) => {
                                let _ = self.abort_worker(cid);
                            },
                        }
                    }
                    // Handle GC interval tick.
                    _ = gc_interval.tick() => {
                        let _ = self.gc();
                    },
                    // Handle expired workflows.
                    Some(expired) = poll_fn(
                        |ctx| match self.expiration_queue.try_borrow_mut() {
                            Ok(mut queue) => queue.poll_expired(ctx),
                            Err(_) => Poll::Pending,
                        }
                    ) => {
                        info!("worker expired, aborting");
                        let _ = self.abort_worker(*expired.get_ref());
                    },
                    // Handle shutdown signal.
                    _ = Self::shutdown_signal() => {
                        info!("gracefully shutting down runner");

                        let now = time::Instant::now();
                        let drain_timeout = now + shutdown_timeout;
                        // Sub-select handling of runner `shutdown`.
                        select! {
                            // Graceful shutdown.
                            Ok(()) = self.shutdown(rpc_sender) => {
                                break now.elapsed();
                            },
                            // Force shutdown upon drain timeout.
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

    /// Getter for the [RunningTaskSet], cloned as an [Arc].
    pub fn running_tasks(&self) -> Arc<RunningTaskSet> {
        self.running_tasks.clone()
    }

    /// [tokio::sync::broadcast::Sender] for sending messages through the
    /// webSocket server to subscribers.
    #[cfg(feature = "websocket-server")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket-server")))]
    pub fn ws_msg_sender(&self) -> &ws::Sender {
        &self.ws_msg_sender
    }

    /// Garbage-collect task [AbortHandle]s in the [RunningTaskSet] and
    /// workers in the [RunningWorkerSet].
    #[allow(dead_code)]
    fn gc(&self) -> Result<()> {
        self.running_tasks.retain(|_cid, handles| {
            handles.retain(|handle| !handle.is_finished());
            !handles.is_empty()
        });

        let mut expiration_q = self
            .expiration_queue
            .try_borrow_mut()
            .map_err(|e| anyhow!("failed to borrow expiration queue: {e}"))?;

        for worker in self.running_workers.iter_mut() {
            let (handle, delay_key) = worker.value();
            if handle.is_finished() {
                let _ = expiration_q.try_remove(delay_key);
            }
        }

        self.running_workers
            .retain(|_cid, (handle, _delay_key)| !handle.is_finished());

        Ok(())
    }

    /// Abort and gc/cleanup all workers and tasks.
    #[allow(dead_code)]
    fn abort_and_cleanup_workers(&self) -> Result<()> {
        self.abort_workers();
        self.cleanup_workers()?;

        Ok(())
    }

    /// Abort all workers.
    #[allow(dead_code)]
    fn abort_workers(&self) {
        self.running_workers.iter_mut().for_each(|data| {
            let (handle, _delay_key) = data.value();
            handle.abort()
        });
        self.abort_tasks();
    }

    /// Cleanup all workers, tasks, and the expiration queue.
    #[allow(dead_code)]
    fn cleanup_workers(&self) -> Result<()> {
        self.running_workers.clear();
        self.expiration_queue
            .try_borrow_mut()
            .map_err(|e| anyhow!("failed to borrow expiration queue: {e}"))?
            .clear();
        self.cleanup_tasks();
        Ok(())
    }

    /// Cleanup all tasks in the [RunningTaskSet].
    #[allow(dead_code)]
    fn cleanup_tasks(&self) {
        self.running_tasks.clear();
    }

    /// Aborts and garbage-collects a set of task [AbortHandle]s running for all
    /// workers.
    #[allow(dead_code)]
    fn abort_tasks(&self) {
        self.running_tasks.iter_mut().for_each(|handles| {
            for abort_handle in &*handles {
                abort_handle.abort();
            }
        });
    }

    /// Aborts and removes a specific worker's [JoinHandle] and
    /// set of task [AbortHandle]s given a [Cid].
    #[allow(dead_code)]
    fn abort_worker(&self, cid: Cid) -> Result<()> {
        let mut expiration_q = self
            .expiration_queue
            .try_borrow_mut()
            .map_err(|e| anyhow!("failed to borrow expiration queue: {e}"))?;

        if let Some((cid, (handle, delay_key))) = self.running_workers.remove(&cid) {
            let _ = expiration_q.try_remove(&delay_key);
            handle.abort();
            self.abort_worker_tasks(cid);
        }

        Ok(())
    }

    /// Abort a specific worker's tasks given a [Cid].
    fn abort_worker_tasks(&self, cid: Cid) {
        if let Some((_cid, handles)) = self.running_tasks.remove(&cid) {
            for abort_handle in &*handles {
                abort_handle.abort();
            }
        }
    }

    /// Captures shutdown signals for [Runner].
    #[allow(dead_code)]
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
    /// a) RPC and runner-related channels.
    /// b) Event-handler channels.
    /// c) Running workers
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

        // abort all workers
        self.abort_workers();

        Ok(())
    }

    #[allow(dead_code)]
    async fn handle_command_message(
        &self,
        msg: rpc::ServerMessage,
        rpc_sender: Arc<BoundedChannelSender<rpc::ServerMessage>>,
        runner_sender: mpsc::Sender<WorkerMessage>,
        db: impl Database + 'static,
        now: time::Instant,
    ) -> Result<ControlFlow<(), rpc::ServerMessage>> {
        info!("received message: {:?}", msg);
        match msg {
            rpc::ServerMessage::ShutdownCmd => {
                info!("RPC shutdown signal received, shutting down runner");

                let drain_timeout = now + self.settings.node.shutdown_timeout;
                select! {
                    Ok(()) = self.shutdown(rpc_sender) => {
                        Ok(ControlFlow::Break(()))
                    },
                    _ = time::sleep_until(drain_timeout) => {
                        info!("shutdown timeout reached, shutting down runner anyway");
                        Ok(ControlFlow::Break(()))
                    }
                }
            }
            rpc::ServerMessage::Run(workflow_file) => {
                let (workflow, workflow_settings) =
                    workflow_file.validate_and_parse().await.with_context(|| {
                        format!("failed to validate/parse workflow @ path: {workflow_file}",)
                    })?;

                #[cfg(feature = "ipfs")]
                let ipfs = IpfsCli::default();

                #[cfg(feature = "ipfs")]
                let worker = {
                    Worker::new(
                        workflow,
                        workflow_settings,
                        self.event_sender(),
                        runner_sender,
                        db.clone(),
                        ipfs,
                    )
                    .await?
                };

                #[cfg(not(feature = "ipfs"))]
                let worker = Worker::new(
                    workflow,
                    workflow_settings,
                    self.event_sender(),
                    runner_sender.into(),
                    db.clone(),
                )
                .await?;

                // Deliberate use of Arc::clone for readability, could just be
                // `clone`, as the underlying type is an `Arc`.
                let initial_info = Arc::clone(&worker.workflow_info);
                let workflow_timeout = worker.workflow_settings.timeout;
                let timestamp = worker.workflow_started;

                // Spawn worker, which schedules execution graph and runs it.
                info!(
                    cid = worker.workflow_info.cid.to_string(),
                    "running workflow with settings: {:#?}", worker.workflow_settings
                );
                let handle = self.runtime.spawn(worker.run(self.running_tasks()));

                // Add Cid to expirations timing wheel
                let delay_key = self
                    .expiration_queue
                    .try_borrow_mut()
                    .map_err(|e| anyhow!("failed to borrow expiration queue: {e}"))?
                    .insert(initial_info.cid, workflow_timeout);

                // Insert handle into running workers map
                self.running_workers
                    .insert(initial_info.cid, (handle, delay_key));

                Ok(ControlFlow::Continue(rpc::ServerMessage::RunAck(
                    response::AckWorkflow::new(initial_info, timestamp),
                )))
            }
            msg => {
                warn!("received unexpected message: {:?}", msg);
                Ok(ControlFlow::Continue(rpc::ServerMessage::Skip))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{network::rpc::Client, test_utils::WorkerBuilder};
    use homestar_core::test_utils as core_test_utils;
    use rand::thread_rng;
    use std::net::SocketAddr;
    use tarpc::context;
    use tokio::net::TcpStream;

    #[homestar_runtime_proc_macro::runner_test]
    fn shutdown() {
        let TestRunner { runner, settings } = TestRunner::start();

        let (tx, _rx) = Runner::setup_rpc_channel(1);
        let rpc_server = rpc::Server::new(settings.node.network(), Arc::new(tx));
        let rpc_sender = rpc_server.sender();

        let addr = SocketAddr::new(
            settings.node.network.rpc_host,
            settings.node.network.rpc_port,
        );

        runner.runtime.block_on(async {
            rpc_server.spawn().await.unwrap();

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

    #[homestar_runtime_proc_macro::runner_test]
    fn spawn_rpc_server_and_ping() {
        let TestRunner { runner, settings } = TestRunner::start();

        let (tx, _rx) = Runner::setup_rpc_channel(1);
        let rpc_server = rpc::Server::new(settings.node.network(), tx.into());

        runner.runtime.block_on(rpc_server.spawn()).unwrap();

        runner.runtime.spawn(async move {
            let addr = SocketAddr::new(
                settings.node.network.rpc_host,
                settings.node.network.rpc_port,
            );

            let client = Client::new(addr, context::current()).await.unwrap();
            let response = client.ping().await.unwrap();
            assert_eq!(response, "pong".to_string());
        });
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn abort_all_workers() {
        let TestRunner { runner, settings } = TestRunner::start();

        runner.runtime.block_on(async {
            let worker = WorkerBuilder::new(settings.node).build().await;
            let workflow_cid = worker.workflow_info.cid;
            let workflow_timeout = worker.workflow_settings.timeout;
            let handle = runner.runtime.spawn(worker.run(runner.running_tasks()));
            let delay_key = runner
                .expiration_queue
                .try_borrow_mut()
                .unwrap()
                .insert(workflow_cid, workflow_timeout);
            runner
                .running_workers
                .insert(workflow_cid, (handle, delay_key));
        });

        runner.abort_workers();
        runner.runtime.block_on(async {
            for (_, (handle, _)) in runner.running_workers {
                assert!(!handle.is_finished());
                assert!(handle.await.unwrap_err().is_cancelled());
            }
        });
        runner.running_tasks.iter().for_each(|handles| {
            for handle in &*handles {
                assert!(handle.is_finished());
            }
        });
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn abort_and_cleanup_all_workers() {
        let TestRunner { runner, settings } = TestRunner::start();

        runner.runtime.block_on(async {
            let worker = WorkerBuilder::new(settings.node).build().await;
            let workflow_cid = worker.workflow_info.cid;
            let workflow_timeout = worker.workflow_settings.timeout;
            let handle = runner.runtime.spawn(worker.run(runner.running_tasks()));
            let delay_key = runner
                .expiration_queue
                .try_borrow_mut()
                .unwrap()
                .insert(workflow_cid, workflow_timeout);
            runner
                .running_workers
                .insert(workflow_cid, (handle, delay_key));
        });

        runner.abort_and_cleanup_workers().unwrap();
        assert!(runner.running_workers.is_empty());
        assert!(runner.running_tasks.is_empty());
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn gc_while_workers_still_running() {
        let TestRunner { runner, settings } = TestRunner::start();

        runner.runtime.block_on(async {
            let worker = WorkerBuilder::new(settings.node).build().await;
            let workflow_cid = worker.workflow_info.cid;
            let workflow_timeout = worker.workflow_settings.timeout;
            let handle = runner.runtime.spawn(worker.run(runner.running_tasks()));
            let delay_key = runner
                .expiration_queue
                .try_borrow_mut()
                .unwrap()
                .insert(workflow_cid, workflow_timeout);

            runner
                .running_workers
                .insert(workflow_cid, (handle, delay_key));
        });

        runner.gc().unwrap();
        assert!(!runner.running_workers.is_empty());

        runner.runtime.block_on(async {
            for (_, (handle, _)) in runner.running_workers {
                assert!(!handle.is_finished());
                let _ = handle.await.unwrap();
            }
        });

        runner.running_tasks.iter().for_each(|handles| {
            for handle in &*handles {
                assert!(handle.is_finished());
            }
        });

        assert!(!runner.running_tasks.is_empty());
        assert!(!runner.expiration_queue.try_borrow_mut().unwrap().is_empty());
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn gc_while_workers_finished() {
        let TestRunner { runner, settings } = TestRunner::start();

        runner.runtime.block_on(async {
            let worker = WorkerBuilder::new(settings.node).build().await;
            let _ = worker.run(runner.running_tasks()).await;
        });

        runner.running_tasks.iter().for_each(|handles| {
            for handle in &*handles {
                assert!(handle.is_finished());
            }
        });

        runner.gc().unwrap();
        assert!(runner.running_tasks.is_empty());
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn abort_all_tasks() {
        let TestRunner { runner, .. } = TestRunner::start();
        let mut set = tokio::task::JoinSet::new();
        runner.runtime.block_on(async {
            for i in 0..3 {
                let handle = set.spawn(async move { i });
                runner.running_tasks.append_or_insert(
                    core_test_utils::cid::generate_cid(&mut thread_rng()),
                    vec![handle],
                );
            }

            while set.join_next().await.is_some() {}
        });

        runner.abort_tasks();
        runner.cleanup_tasks();
        assert!(runner.running_tasks.is_empty());
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn abort_one_task() {
        let TestRunner { runner, .. } = TestRunner::start();
        let mut set = tokio::task::JoinSet::new();
        let mut cids = vec![];
        runner.runtime.block_on(async {
            for i in 0..3 {
                let handle = set.spawn(async move { i });
                let cid = core_test_utils::cid::generate_cid(&mut thread_rng());
                runner.running_tasks.append_or_insert(cid, vec![handle]);
                cids.push(cid);
            }

            while set.join_next().await.is_some() {}
        });

        assert!(runner.running_tasks.len() == 3);
        runner.abort_worker_tasks(cids[0]);
        assert!(runner.running_tasks.len() == 2);
    }
}
