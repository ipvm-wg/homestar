//! General [Runner] interface for working across multiple workers
//! and executing workflows.

#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    channel::{AsyncChannel, AsyncChannelReceiver, AsyncChannelSender},
    db::Database,
    event_handler::{Event, EventHandler},
    network::{rpc, swarm, webserver},
    settings,
    tasks::Fetch,
    worker::WorkerMessage,
    workflow::{self, Resource},
    Db, Receipt, Settings, Worker,
};
use anyhow::{anyhow, Context, Result};
use atomic_refcell::AtomicRefCell;
use chrono::NaiveDateTime;
use dashmap::DashMap;
use faststr::FastStr;
use fnv::FnvHashSet;
use futures::{future::poll_fn, FutureExt};
use homestar_invocation::Pointer;
use homestar_wasm::io::Arg;
use homestar_workflow::Workflow;
use jsonrpsee::server::ServerHandle;
use libipld::Cid;
use metrics_exporter_prometheus::PrometheusHandle;
#[cfg(not(test))]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{collections::HashMap, ops::ControlFlow, rc::Rc, sync::Arc, task::Poll};
#[cfg(not(windows))]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(windows)]
use tokio::signal::windows;
use tokio::{
    runtime, select,
    task::{AbortHandle, JoinHandle},
    time,
};
use tokio_util::time::{delay_queue, DelayQueue};
use tracing::{debug, error, info, info_span, instrument, warn, Instrument};

mod error;
pub(crate) mod file;
mod nodeinfo;
pub mod response;
pub(crate) use error::Error;
pub use nodeinfo::NodeInfo;
pub(crate) use nodeinfo::{DynamicNodeInfo, StaticNodeInfo};

/// Name of the thread used for the [Runner] / runtime.
#[cfg(not(test))]
const HOMESTAR_THREAD: &str = "homestar-runtime";

/// Type alias for a [DashMap] containing running worker [JoinHandle]s.
pub(crate) type RunningWorkerSet = DashMap<Cid, (JoinHandle<Result<()>>, delay_queue::Key)>;

/// Type alias for a [DashMap] containing running task [AbortHandle]s.
pub(crate) type RunningTaskSet = DashMap<Cid, Vec<AbortHandle>>;

/// Trait for managing a [DashMap] of running task information.
pub(crate) trait ModifiedSet {
    /// Append or insert a new [AbortHandle] into the [RunningTaskSet].
    fn append_or_insert(&self, cid: Cid, handles: Vec<AbortHandle>);
}

/// [AsyncChannelSender] for RPC server messages.
pub(crate) type RpcSender = AsyncChannelSender<(
    rpc::ServerMessage,
    Option<AsyncChannelSender<rpc::ServerMessage>>,
)>;

/// [AsyncChannelReceiver] for RPC server messages.
pub(crate) type RpcReceiver = AsyncChannelReceiver<(
    rpc::ServerMessage,
    Option<AsyncChannelSender<rpc::ServerMessage>>,
)>;

/// Type alias for a tuple containing a receipt Cid and associated `ran` and `instruction` values.
pub(crate) type WorkflowReceiptInfo = (Cid, Option<(String, Pointer)>);

/// [AsyncChannelSender] for sending messages WebSocket server clients.
pub(crate) type WsSender = AsyncChannelSender<(
    webserver::Message,
    Option<AsyncChannelSender<webserver::Message>>,
)>;

/// [AsyncChannelReceiver] for receiving messages from WebSocket server clients.
pub(crate) type WsReceiver = AsyncChannelReceiver<(
    webserver::Message,
    Option<AsyncChannelSender<webserver::Message>>,
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
/// [Workflows]: homestar_workflow::Workflow
#[derive(Debug)]
pub struct Runner {
    event_sender: Arc<AsyncChannelSender<Event>>,
    expiration_queue: Rc<AtomicRefCell<DelayQueue<Cid>>>,
    node_info: StaticNodeInfo,
    running_tasks: Arc<RunningTaskSet>,
    running_workers: RunningWorkerSet,
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) settings: Arc<Settings>,
    webserver: Arc<webserver::Server>,
}

impl Runner {
    /// Setup bounded, MPSC channel for top-level RPC communication.
    pub(crate) fn setup_rpc_channel(capacity: usize) -> (RpcSender, RpcReceiver) {
        AsyncChannel::with(capacity)
    }

    /// Setup bounded, MPSC channel for top-level Worker communication.
    pub(crate) fn setup_worker_channel(
        capacity: usize,
    ) -> (
        AsyncChannelSender<WorkerMessage>,
        AsyncChannelReceiver<WorkerMessage>,
    ) {
        AsyncChannel::with(capacity)
    }

    /// MPSC channel for sending and receiving messages through to/from
    /// WebSocket server clients.
    pub(crate) fn setup_ws_mpsc_channel(capacity: usize) -> (WsSender, WsReceiver) {
        AsyncChannel::with(capacity)
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
    pub fn start(settings: Settings, db: impl Database + 'static) -> Result<Self> {
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
    ) -> Result<Self> {
        let swarm = runtime.block_on(swarm::new(settings.node().network()))?;
        let peer_id = *swarm.local_peer_id();

        let webserver = webserver::Server::new(settings.node().network().webserver())?;

        #[cfg(feature = "websocket-notify")]
        let (ws_msg_tx, ws_evt_tx) = {
            let ws_msg_tx = webserver.workflow_msg_notifier();
            let ws_evt_tx = webserver.evt_notifier();

            (ws_msg_tx, ws_evt_tx)
        };

        #[cfg(feature = "websocket-notify")]
        let event_handler =
            EventHandler::new(swarm, db, settings.node().network(), ws_evt_tx, ws_msg_tx);
        #[cfg(not(feature = "websocket-notify"))]
        let event_handler = EventHandler::new(swarm, db, settings.node().network());

        let event_sender = event_handler.sender();

        #[cfg(feature = "ipfs")]
        let _event_handler_hdl = runtime.spawn({
            let ipfs = IpfsCli::new(settings.node.network.ipfs())?;
            event_handler.start(ipfs)
        });

        #[cfg(not(feature = "ipfs"))]
        let _event_handler_hdl = runtime.spawn(event_handler.start());

        Ok(Self {
            event_sender,
            expiration_queue: Rc::new(AtomicRefCell::new(DelayQueue::new())),
            node_info: StaticNodeInfo::new(peer_id),
            running_tasks: DashMap::new().into(),
            running_workers: DashMap::new(),
            runtime,
            settings: settings.into(),
            webserver: webserver.into(),
        })
    }

    /// Listen loop for [Runner] signals and messages.
    #[allow(dead_code)]
    fn serve(self, db: impl Database + 'static) -> Result<()> {
        let message_buffer_len = self.settings.node.network.events_buffer_len;

        #[cfg(feature = "monitoring")]
        let metrics_hdl: PrometheusHandle = self.runtime.block_on(crate::metrics::start(
            self.settings.node.monitoring(),
            self.settings.node.network(),
        ))?;

        #[cfg(not(feature = "monitoring"))]
        let metrics_hdl: PrometheusHandle = self
            .runtime
            .block_on(crate::metrics::start(self.settings.node.network()))?;

        let (ws_receiver, ws_hdl) = {
            let (mpsc_ws_tx, mpsc_ws_rx) = Self::setup_ws_mpsc_channel(message_buffer_len);
            let ws_hdl =
                self.runtime
                    .block_on(self.webserver.start(mpsc_ws_tx, metrics_hdl, db.clone()))?;
            (mpsc_ws_rx, ws_hdl)
        };

        let (rpc_tx, rpc_rx) = Self::setup_rpc_channel(message_buffer_len);
        let (runner_worker_tx, runner_worker_rx) = Self::setup_worker_channel(message_buffer_len);

        let shutdown_timeout = self.settings.node.shutdown_timeout;
        let rpc_server = rpc::Server::new(self.settings.node.network(), rpc_tx.into());
        let rpc_sender = rpc_server.sender();
        self.runtime.block_on(rpc_server.spawn())?;

        let shutdown_time_left = self.runtime.block_on(async {
            let mut gc_interval = tokio::time::interval(self.settings.node.gc_interval);
            loop {
                select! {
                    // Handle RPC messages.
                    Ok((rpc_message, Some(oneshot_tx))) = rpc_rx.recv_async() => {
                        let now = time::Instant::now();
                        let handle = self.handle_command_message(
                            rpc_message,
                            Channels {
                                rpc: rpc_sender.clone(),
                                runner: runner_worker_tx.clone(),
                            },
                            ws_hdl.clone(),
                            db.clone(),
                            self.settings.node.network().libp2p().dht(),
                            now
                        ).await;


                        match handle {
                            Ok(ControlFlow::Break(())) => break now.elapsed(),
                            Ok(ControlFlow::Continue(rpc::ServerMessage::Skip)) => {},
                            Ok(ControlFlow::Continue(msg @ rpc::ServerMessage::NodeInfoAck(_))) => {
                                debug!(subject = "rpc.ack",
                                       category = "rpc",
                                       "sending node_info message to rpc server");
                                let _ = oneshot_tx.send_async(msg).await;
                            },
                            Ok(ControlFlow::Continue(msg @ rpc::ServerMessage::RunAck(_))) => {
                                debug!(subject = "rpc.ack",
                                       category = "rpc",
                                       "sending workflow_run message to rpc server");
                                let _ = oneshot_tx.send_async(msg).await;
                            },
                            Err(err) => {
                                error!(subject = "rpc.err",
                                       category = "rpc",
                                       err=?err,
                                       "error handling rpc message");
                                let _ = oneshot_tx.send_async(rpc::ServerMessage::RunErr(err.into())).await;
                            },
                             _ => {}
                        }
                    }
                    Ok(msg) = ws_receiver.recv_async() => {
                        match msg {
                            (webserver::Message::RunWorkflow((name, workflow)), Some(oneshot_tx)) => {
                                info!(subject = "workflow",
                                      category = "workflow.run",
                                      "running workflow: {}", name);
                                // TODO: Parse this from the workflow data itself.
                                let workflow_settings = workflow::Settings::default();
                                match self.run_worker(
                                    workflow,
                                    workflow_settings,
                                    self.settings.node.network().libp2p().dht(),
                                    Some(name),
                                    runner_worker_tx.clone(),
                                    db.clone(),
                                ).await {
                                    Ok(data) => {
                                        debug!(subject = "jsonrpc.ack",
                                               category = "jsonrpc",
                                               "sending message to jsonrpc server");
                                        let _ = oneshot_tx.send_async(webserver::Message::AckWorkflow((data.info.cid, data.name))).await;
                                    }
                                    Err(err) => {
                                        error!(subject = "jsonrpc.err",
                                               category = "jsonrpc",
                                               err=?err,
                                               "error handling ws message");
                                        let _ = oneshot_tx.send_async(webserver::Message::RunErr(err.into())).await;
                                    }
                                }

                            }
                            (webserver::Message::GetNodeInfo, Some(oneshot_tx)) => {
                                debug!(subject = "jsonrpc.nodeinfo",
                                       category = "jsonrpc",
                                       "getting node info");
                                let (tx, rx) = AsyncChannel::oneshot();
                                let _ = self.event_sender.send_async(Event::GetNodeInfo(tx)).await;
                                let dyn_node_info = if let Ok(info) = rx.recv_async().await {
                                    info
                                } else {
                                    DynamicNodeInfo::default()
                                };
                                let _ = oneshot_tx.send_async(webserver::Message::AckNodeInfo((self.node_info.clone(), dyn_node_info))).await;
                            }
                            _ => ()
                        }
                    }

                    // Handle messages from the worker.
                    Ok(msg) = runner_worker_rx.recv_async() => {
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
                        info!(subject = "worker.expired",
                              category = "worker",
                              "worker expired, aborting");
                        let _ = self.abort_worker(*expired.get_ref());
                    },
                    // Handle shutdown signal.
                    _ = Self::shutdown_signal() => {
                        info!(subject = "shutdown",
                              category = "homestar.shutdown",
                              "gracefully shutting down runner");

                        let now = time::Instant::now();
                        let drain_timeout = now + shutdown_timeout;
                        select! {
                            // Graceful shutdown.
                            Ok(()) = self.shutdown(rpc_sender, ws_hdl) => {
                                break now.elapsed();
                            },
                            // Force shutdown upon drain timeout.
                            _ = time::sleep_until(drain_timeout) => {
                                info!(subject = "shutdown",
                                      category = "homestar.shutdown",
                                      "shutdown timeout reached, shutting down runner anyway");
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
            info!(
                subject = "shutdown",
                category = "homestar.shutdown",
                "runner shutdown complete"
            );
        }

        Ok(())
    }

    /// [AsyncChannelSender] of the event-handler.
    ///
    /// [EventHandler]: crate::EventHandler
    pub(crate) fn event_sender(&self) -> Arc<AsyncChannelSender<Event>> {
        self.event_sender.clone()
    }

    /// Getter for the [RunningTaskSet], cloned as an [Arc].
    pub(crate) fn running_tasks(&self) -> Arc<RunningTaskSet> {
        self.running_tasks.clone()
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
    /// set of task [AbortHandle]s given a Cid.
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

    /// Abort a specific worker's tasks given a Cid.
    fn abort_worker_tasks(&self, cid: Cid) {
        if let Some((_cid, handles)) = self.running_tasks.remove(&cid) {
            for abort_handle in &*handles {
                abort_handle.abort();
            }
        }
    }

    /// Captures shutdown signals for [Runner].
    #[allow(dead_code)]
    #[cfg(not(windows))]
    async fn shutdown_signal() -> Result<()> {
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;

        select! {
            _ = tokio::signal::ctrl_c() =>
                info!(subject = "shutdown",
                      category = "homestar.shutdown",
                      "CTRL-C received, shutting down"),
            _ = sigint.recv() =>
                info!(subject = "shutdown",
                      category = "homestar.shutdown",
                      "SIGINT received, shutting down"),
            _ = sigterm.recv() =>
                info!(subject = "shutdown",
                      category = "homestar.shutdown",
                      "SIGTERM received, shutting down"),
        }
        Ok(())
    }

    #[allow(dead_code)]
    #[cfg(windows)]
    async fn shutdown_signal() -> Result<()> {
        let mut sigint = windows::ctrl_close()?;
        let mut sigterm = windows::ctrl_shutdown()?;
        let mut sighup = windows::ctrl_break()?;

        select! {
            _ = tokio::signal::ctrl_c() =>
                info!(subject = "shutdown",
                      category = "homestar.shutdown",
                      "CTRL-C received, shutting down"),
            _ = sigint.recv() =>
                info!(subject = "shutdown",
                      category = "homestar.shutdown",
                      "SIGINT received, shutting down"),
            _ = sigterm.recv() =>
                info!(subject = "shutdown",
                      category = "homestar.shutdown",
                      "SIGTERM received, shutting down"),
            _ = sighup.recv() =>
                info!(subject = "shutdown",
                      category = "homestar.shutdown",
                      "SIGHUP received, shutting down")
        }
        Ok(())
    }

    /// Sequence for shutting down a [Runner], including:
    /// a) RPC (CLI)
    /// b) Webserver
    /// b) Event-handler channels
    /// c) Running workers
    async fn shutdown(
        &self,
        rpc_sender: Arc<AsyncChannelSender<rpc::ServerMessage>>,
        ws_hdl: ServerHandle,
    ) -> Result<()> {
        let (shutdown_sender, shutdown_receiver) = AsyncChannel::oneshot();
        let _ = rpc_sender
            .send_async(rpc::ServerMessage::GracefulShutdown(shutdown_sender))
            .await;
        let _ = shutdown_receiver;

        info!(
            subject = "shutdown",
            category = "homestar.shutdown",
            "shutting down webserver"
        );

        let _ = ws_hdl.stop();
        ws_hdl.stopped().await;

        let (shutdown_sender, shutdown_receiver) = AsyncChannel::oneshot();
        let _ = self
            .event_sender
            .send_async(Event::Shutdown(shutdown_sender))
            .await;
        let _ = shutdown_receiver;

        // abort all workers
        self.abort_workers();

        Ok(())
    }

    async fn handle_command_message(
        &self,
        msg: rpc::ServerMessage,
        channels: Channels,
        ws_hdl: ServerHandle,
        db: impl Database + 'static,
        network_settings: &settings::Dht,
        now: time::Instant,
    ) -> Result<ControlFlow<(), rpc::ServerMessage>> {
        match msg {
            rpc::ServerMessage::NodeInfo => {
                info!(
                    subject = "rpc.command",
                    category = "rpc",
                    "RPC node command received, sending node info"
                );

                let (tx, rx) = AsyncChannel::oneshot();
                let _ = self.event_sender.send_async(Event::GetNodeInfo(tx)).await;

                let dyn_node_info = if let Ok(info) = rx.recv_async().await {
                    info
                } else {
                    DynamicNodeInfo::default()
                };

                Ok(ControlFlow::Continue(rpc::ServerMessage::NodeInfoAck(
                    response::AckNodeInfo::new(self.node_info.clone(), dyn_node_info),
                )))
            }
            rpc::ServerMessage::ShutdownCmd => {
                info!(
                    subject = "rpc.command",
                    category = "rpc",
                    "RPC shutdown signal received, shutting down runner"
                );
                let drain_timeout = now + self.settings.node.shutdown_timeout;
                select! {
                    // we can unwrap here b/c we know we have a sender based
                    // on the feature flag.
                    Ok(()) = self.shutdown(channels.rpc, ws_hdl) => {
                        Ok(ControlFlow::Break(()))
                    },
                    _ = time::sleep_until(drain_timeout) => {
                        info!(subject = "shutdown",
                              category = "homestar.shutdown",
                              "shutdown timeout reached, shutting down runner anyway");
                        Ok(ControlFlow::Break(()))
                    }
                }
            }
            rpc::ServerMessage::Run((name, workflow_file)) => {
                info!(
                    subject = "rpc.command",
                    category = "rpc",
                    "RPC run command received, running workflow"
                );
                let (workflow, workflow_settings) =
                    workflow_file.validate_and_parse().await.with_context(|| {
                        format!("failed to validate/parse workflow @ path: {workflow_file}",)
                    })?;

                let data = self
                    .run_worker(
                        workflow,
                        workflow_settings,
                        network_settings,
                        name,
                        channels.runner,
                        db.clone(),
                    )
                    .await?;

                Ok(ControlFlow::Continue(rpc::ServerMessage::RunAck(Box::new(
                    response::AckWorkflow::new(
                        data.info,
                        data.replayed_receipt_info,
                        data.name,
                        data.timestamp,
                    ),
                ))))
            }
            msg => {
                warn!(
                    subject = "rpc.command",
                    category = "rpc",
                    "received unexpected message: {:?}",
                    msg
                );
                Ok(ControlFlow::Continue(rpc::ServerMessage::Skip))
            }
        }
    }

    #[instrument(skip_all)]
    async fn run_worker<S: Into<FastStr>>(
        &self,
        workflow: Workflow<'static, Arg>,
        workflow_settings: workflow::Settings,
        network_settings: &settings::Dht,
        name: Option<S>,
        runner_sender: AsyncChannelSender<WorkerMessage>,
        db: impl Database + 'static,
    ) -> Result<WorkflowData> {
        let worker = {
            Worker::new(
                workflow,
                workflow_settings,
                network_settings.clone().to_owned(),
                name,
                self.event_sender(),
                runner_sender,
                db.clone(),
            )
            .await?
        };

        // Deliberate use of Arc::clone for readability, could just be
        // `clone`, as the underlying type is an `Arc`.
        let initial_info = Arc::clone(&worker.workflow_info);
        let workflow_timeout = worker.workflow_settings.timeout;
        let workflow_name = worker.workflow_name.clone();
        let workflow_settings = worker.workflow_settings.clone();
        let timestamp = worker.workflow_started;

        // Spawn worker, which initializees the scheduler and runs
        // the workflow.
        info!(
            subject = "workflow.run",
            category = "workflow",
            cid = worker.workflow_info.cid.to_string(),
            "running workflow with settings: {:#?}",
            worker.workflow_settings
        );

        // Provide workflow to network.
        //
        // This essentially says, I'm running this workflow Cid.
        self.event_sender
            .send_async(Event::ProvideRecord(
                worker.workflow_info.cid,
                None,
                swarm::CapsuleTag::Workflow,
            ))
            .await?;

        #[cfg(feature = "ipfs")]
        let fetch_fn = {
            let settings = Arc::clone(&self.settings);
            let ipfs = IpfsCli::new(settings.node.network.ipfs())?;
            move |rscs: FnvHashSet<Resource>| {
                async move { Fetch::get_resources(rscs, workflow_settings, ipfs).await }.boxed()
            }
        };

        #[cfg(not(feature = "ipfs"))]
        let fetch_fn = |rscs: FnvHashSet<Resource>| {
            async move { Fetch::get_resources(rscs, workflow_settings).await }.boxed()
        };

        let handle = self.runtime.spawn(
            worker
                .run(self.running_tasks(), fetch_fn)
                .instrument(info_span!("run").or_current()),
        );

        // Add Cid to expirations timing wheel
        let delay_key = self
            .expiration_queue
            .try_borrow_mut()
            .map_err(|e| anyhow!("failed to borrow expiration queue: {e}"))?
            .insert(initial_info.cid, workflow_timeout);

        // Insert handle into running workers map
        self.running_workers
            .insert(initial_info.cid, (handle, delay_key));

        // Gather receipt info
        let receipt_pointers = initial_info
            .progress
            .iter()
            .map(|cid| Pointer::new(*cid))
            .collect();
        let replayed_receipt_info = find_receipt_info_by_pointers(&receipt_pointers, db)?;

        // Log replayed receipts if any
        if !replayed_receipt_info.is_empty() {
            info!(
                subject = "workflow.receipts",
                category = "workflow",
                receipt_cids = replayed_receipt_info
                    .iter()
                    .map(|info| info.0.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
                "replaying receipts",
            );
        };

        Ok(WorkflowData {
            info: initial_info,
            name: workflow_name,
            timestamp,
            replayed_receipt_info,
        })
    }
}

/// Find receipts given a batch of [Receipt] [Pointer]s, and return them as [WorkflowReceiptInfo]s.
fn find_receipt_info_by_pointers(
    pointers: &Vec<Pointer>,
    db: impl Database + 'static,
) -> Result<Vec<WorkflowReceiptInfo>> {
    let receipts: HashMap<Cid, Receipt> = Db::find_receipt_pointers(pointers, &mut db.conn()?)?
        .into_iter()
        .map(|receipt| (receipt.cid(), receipt))
        .collect();

    let receipt_info = pointers
        .iter()
        .map(|pointer| match receipts.get(&pointer.cid()) {
            Some(receipt) => (
                pointer.cid(),
                Some((receipt.ran(), receipt.instruction().clone())),
            ),
            None => (pointer.cid(), None),
        })
        .collect();

    Ok(receipt_info)
}

/// Internal Workflow data used for wrapper.
struct WorkflowData {
    info: Arc<workflow::Info>,
    name: FastStr,
    timestamp: NaiveDateTime,
    replayed_receipt_info: Vec<WorkflowReceiptInfo>,
}

/// Channels for sending messages to/from the RPC server and the runner.
#[derive(Debug)]
struct Channels {
    rpc: Arc<AsyncChannelSender<rpc::ServerMessage>>,
    runner: AsyncChannelSender<WorkerMessage>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        network::rpc::Client,
        test_utils::{db::MemoryDb, WorkerBuilder},
    };
    use metrics_exporter_prometheus::PrometheusBuilder;
    use rand::thread_rng;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tarpc::context;
    use tokio::net::TcpStream;

    #[homestar_runtime_proc_macro::runner_test]
    fn shutdown() {
        let TestRunner { runner, settings } = TestRunner::start();
        let (tx, _rx) = Runner::setup_rpc_channel(1);
        let (runner_tx, _runner_rx) = Runner::setup_ws_mpsc_channel(1);
        let db = MemoryDb::setup_connection_pool(settings.node(), None).unwrap();
        let rpc_server = rpc::Server::new(settings.node.network(), Arc::new(tx));
        let rpc_sender = rpc_server.sender();

        let addr = SocketAddr::new(
            settings.node.network.rpc.host,
            settings.node.network.rpc.port,
        );

        let ws_hdl = runner.runtime.block_on(async {
            rpc_server.spawn().await.unwrap();

            let port = port_selector::random_free_tcp_port().unwrap();
            let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
            let (recorder, _exporter) = PrometheusBuilder::new()
                .with_http_listener(socket)
                .build()
                .expect("failed to install recorder/exporter");
            let metrics_hdl = recorder.handle();

            let ws_hdl = runner
                .webserver
                .start(runner_tx, metrics_hdl, db)
                .await
                .unwrap();
            let _stream = TcpStream::connect(addr).await.expect("Connection error");
            let _another_stream = TcpStream::connect(addr).await.expect("Connection error");

            ws_hdl
        });

        runner.runtime.block_on(async {
            match runner.shutdown(rpc_sender, ws_hdl).await {
                Ok(()) => {
                    // with shutdown, we should not be able to connect to the server(s)
                    let stream_error = TcpStream::connect(addr).await;
                    assert!(stream_error.is_err());
                    assert!(matches!(
                        stream_error.unwrap_err().kind(),
                        std::io::ErrorKind::ConnectionRefused
                    ));

                    let ws_error =
                        tokio_tungstenite::connect_async("ws://localhost:1337".to_string()).await;
                    assert!(ws_error.is_err());
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
                settings.node.network.rpc.host,
                settings.node.network.rpc.port,
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
            let builder = WorkerBuilder::new(settings.node);
            let fetch_fn = builder.fetch_fn();
            let worker = builder.build().await;
            let workflow_cid = worker.workflow_info.cid;
            let workflow_timeout = worker.workflow_settings.timeout;
            let handle = runner
                .runtime
                .spawn(worker.run(runner.running_tasks(), fetch_fn));
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
            let builder = WorkerBuilder::new(settings.node);
            let fetch_fn = builder.fetch_fn();
            let worker = builder.build().await;
            let workflow_cid = worker.workflow_info.cid;
            let workflow_timeout = worker.workflow_settings.timeout;
            let handle = runner
                .runtime
                .spawn(worker.run(runner.running_tasks(), fetch_fn));
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
            let builder = WorkerBuilder::new(settings.node);
            let fetch_fn = builder.fetch_fn();
            let worker = builder.build().await;
            let workflow_cid = worker.workflow_info.cid;
            let workflow_timeout = worker.workflow_settings.timeout;
            let handle = runner
                .runtime
                .spawn(worker.run(runner.running_tasks(), fetch_fn));
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
            let builder = WorkerBuilder::new(settings.node);
            let fetch_fn = builder.fetch_fn();
            let worker = builder.build().await;
            let _ = worker.run(runner.running_tasks(), fetch_fn).await;
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
                    homestar_invocation::test_utils::cid::generate_cid(&mut thread_rng()),
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
                let cid = homestar_invocation::test_utils::cid::generate_cid(&mut thread_rng());
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
