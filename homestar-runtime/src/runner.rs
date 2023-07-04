//! General [Runner] interface for working across multiple workers
//! and executing workflows.

#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{db::Database, network::swarm, Event, EventHandler, Settings};
#[cfg(feature = "websocket-server")]
use crate::{
    event_handler::channel::{BoundedChannel, BoundedChannelReceiver},
    network::ws::{self, WebSocketServer},
};
use anyhow::Result;
use dashmap::DashMap;
use libipld::Cid;
use std::sync::Arc;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
    sync::{mpsc, oneshot},
    task::AbortHandle,
};
use tracing::info;

/// Type alias for a [DashMap] containing running task information.
pub type RunningSet = DashMap<Cid, Vec<AbortHandle>>;

/// Trait for managing a [DashMap] of running task information.
pub trait ModifiedSet {
    /// Append or insert a new [AbortHandle] into the [RunningSet].
    fn append_or_insert(&mut self, cid: Cid, handles: Vec<AbortHandle>);
}

impl ModifiedSet for RunningSet {
    fn append_or_insert(&mut self, cid: Cid, mut handles: Vec<AbortHandle>) {
        self.entry(cid)
            .and_modify(|prev_handles| {
                prev_handles.append(&mut handles);
            })
            .or_insert_with(|| handles);
    }
}

/// Runner interface.
/// Used to manage [Workers] and execute/run [Workflows].
///
/// [Workers]: crate::Worker
/// [Workflows]: homestar_core::Workflow
#[cfg(feature = "websocket-server")]
#[derive(Debug)]
pub struct Runner {
    command_sender: oneshot::Sender<Event>,
    command_receiver: oneshot::Receiver<Event>,
    event_sender: Arc<mpsc::Sender<Event>>,
    running_set: RunningSet,
    #[allow(dead_code)]
    ws_sender: ws::WsSender,
    ws_receiver: BoundedChannelReceiver<ws::WsMessage>,
}

/// Runner interface.
/// Used to manage [Workers] and execute/run [Workflows].
///
/// [Workers]: crate::Worker
/// [Workflows]: homestar_core::Workflow
#[cfg(not(feature = "websocket-server"))]
#[derive(Debug)]
pub struct Runner {
    command_sender: oneshot::Sender<Event>,
    command_receiver: oneshot::Receiver<Event>,
    event_sender: Arc<mpsc::Sender<Event>>,
    running_set: RunningSet,
}

impl Runner {
    /// Start the Homestar runner context.
    pub async fn start(settings: Arc<Settings>, db: impl Database + 'static) -> Result<Self> {
        let (command_sender, command_receiver) = oneshot::channel();
        let map = DashMap::new();
        let swarm = swarm::new(settings.node()).await?;

        let event_handler = EventHandler::new(swarm, db.clone(), settings.node());
        let event_sender = event_handler.sender();

        #[cfg(feature = "ipfs")]
        tokio::spawn({
            let ipfs = IpfsCli::default();
            event_handler.start(ipfs)
        });

        #[cfg(not(feature = "ipfs"))]
        tokio::spawn(event_handler.start());

        #[cfg(feature = "websocket-server")]
        {
            // Setup websocket communication.
            let (tx, _rx) =
                WebSocketServer::setup_channel(settings.node().network().websocket_capacity);
            let ws_tx = Arc::new(tx);
            let ws_channel = BoundedChannel::oneshot();
            let oneshot_sender = ws_channel.tx;
            let oneshot_receiver = ws_channel.rx;

            tokio::spawn({
                let settings = settings.node().network().clone();
                WebSocketServer::start(settings, ws_tx.clone(), oneshot_sender.into())
            });

            Ok(Self {
                command_sender,
                command_receiver,
                event_sender,
                running_set: map,
                ws_sender: ws_tx,
                ws_receiver: oneshot_receiver,
            })
        }

        #[cfg(not(feature = "websocket-server"))]
        Ok(Self {
            command_sender,
            command_receiver,
            event_sender,
            running_set: map,
        })
    }

    /// Sequence for shutting down a [Runner], including:
    /// a) [EventHandler] channels,
    /// b) Running workers
    /// c) [Runner] channels.
    ///
    /// [EventHandler]: crate::EventHandler
    pub async fn shutdown(&mut self) -> Result<()> {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();
        self.event_sender
            .send(Event::Shutdown(shutdown_sender))
            .await?;

        shutdown_receiver.await?;

        // TODO: shutdown workers

        info!("shutting down runner's channels");
        self.command_receiver.close();
        self.command_sender.closed().await;
        Ok(())
    }

    /// Captures shutdown signals for [Runner] and other sub-processes like
    /// the [webSocket server].
    ///
    /// [websocket server]: WebSocketServer
    pub async fn shutdown_signal() -> Result<()> {
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;

        select! {
            _ = tokio::signal::ctrl_c() => info!("CTRL-C received, shutting down"),
            _ = sigint.recv() => info!("SIGINT received, shutting down"),
            _ = sigterm.recv() => info!("SIGTERM received, shutting down"),
        }

        Ok(())
    }

    /// Garbage-collect task [AbortHandle]s in the [RunningSet].
    pub fn gc(&mut self) {
        self.running_set.retain(|_cid, handles| {
            handles.retain(|handle| !handle.is_finished());
            !handles.is_empty()
        });
    }

    /// Garbage-collect task [AbortHandle]s in the [RunningSet] for a specific
    /// [Worker]-workflow [Cid].
    ///
    /// [Worker]: crate::Worker
    pub fn gc_worker(&mut self, cid: Cid) {
        if let Some(mut handles) = self.running_set.get_mut(&cid) {
            handles.retain(|handle| !handle.is_finished());
        }
        self.running_set.retain(|_cid, handles| !handles.is_empty());
    }

    /// Abort all [Workers].
    ///
    /// [Workers]: crate::Worker
    pub fn abort_all_tasks(&mut self) {
        self.running_set.iter_mut().for_each(|handles| {
            for abort_handle in &*handles {
                abort_handle.abort();
            }
        });
    }

    /// Abort a specific [Worker]'s tasks given a [Cid].
    ///
    /// [Worker]: crate::Worker
    pub fn abort_worker_tasks(&mut self, cid: Cid) {
        if let Some(handles) = self.running_set.get_mut(&cid) {
            for abort_handle in &*handles {
                abort_handle.abort();
            }
        }
    }

    /// [mpsc::Sender] of the [EventHandler].
    ///
    /// [EventHandler]: crate::EventHandler
    pub fn event_sender(&self) -> Arc<mpsc::Sender<Event>> {
        self.event_sender.clone()
    }

    /// [tokio::broadcast::Sender] for sending messages through the
    /// [webSocket server] to subscribers.
    ///
    /// [websocket server]: WebSocketServer
    #[cfg(feature = "websocket-server")]
    pub fn ws_sender(&self) -> &ws::WsSender {
        &self.ws_sender
    }

    /// [BoundedChannel] for receiving [messages] back from the
    /// [webSocket server].
    ///
    /// [messages]: ws::WsMessage
    /// [websocket server]: WebSocketServer
    #[cfg(feature = "websocket-server")]
    pub fn ws_receiver(&mut self) -> &mut BoundedChannelReceiver<ws::WsMessage> {
        &mut self.ws_receiver
    }

    /// [oneshot::Sender] for sending commands to the [Runner].
    pub fn command_sender(&self) -> &oneshot::Sender<Event> {
        &self.command_sender
    }

    /// [oneshot::Receiver] for Runner to receive commands.
    pub fn command_receiver(&mut self) -> &mut oneshot::Receiver<Event> {
        &mut self.command_receiver
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use homestar_core::test_utils;
    use rand::thread_rng;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    static ATOMIC_PORT: AtomicUsize = AtomicUsize::new(1338);

    async fn setup() -> Runner {
        let mut settings = Settings::load().unwrap();
        settings.node.network.websocket_port = ATOMIC_PORT.fetch_add(1, Ordering::SeqCst) as u16;
        let db = crate::test_utils::db::MemoryDb::setup_connection_pool(
            Settings::load().unwrap().node(),
        )
        .unwrap();

        Runner::start(settings.into(), db).await.unwrap()
    }

    #[tokio::test]
    async fn shutdown() {
        let mut runner = setup().await;

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            // Send SIGINT signal
            let _ = nix::sys::signal::kill(nix::unistd::getpid(), nix::sys::signal::Signal::SIGINT);
        });

        select! {
            result = Runner::shutdown_signal() => {
                assert!(result.is_ok());
                select! {
                    Ok(()) = runner.shutdown() => {
                        assert!(runner.command_sender().is_closed());
                        #[cfg(feature = "websocket-server")]
                        assert_eq!(runner.ws_receiver().recv().unwrap(), ws::WsMessage::GracefulShutdown);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn abort_all_tasks() {
        let mut runner = setup().await;

        let mut set = tokio::task::JoinSet::new();

        for i in 0..3 {
            let handle = set.spawn(async move { i });
            runner.running_set.append_or_insert(
                test_utils::cid::generate_cid(&mut thread_rng()),
                vec![handle],
            );
        }

        runner.abort_all_tasks();
        assert!(!runner.running_set.is_empty());

        while set.join_next().await.is_some() {}
        runner.gc();
        assert!(runner.running_set.is_empty());
    }

    #[tokio::test]
    async fn abort_one_task() {
        let mut runner = setup().await;

        let mut set = tokio::task::JoinSet::new();
        let mut cids = vec![];

        for i in 0..3 {
            let handle = set.spawn(async move { i });
            let cid = test_utils::cid::generate_cid(&mut thread_rng());
            runner.running_set.append_or_insert(cid, vec![handle]);
            cids.push(cid);
        }

        runner.abort_worker_tasks(cids[0]);
        assert!(runner.running_set.len() == 3);

        while set.join_next().await.is_some() {}

        runner.gc_worker(cids[0]);

        assert!(runner.running_set.len() == 2);
    }
}
