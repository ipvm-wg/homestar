//! [EventHandler] implementation for handling network events and messages.

#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{db::Database, network::swarm::ComposedBehaviour, settings};
use anyhow::Result;
use async_trait::async_trait;
use fnv::FnvHashMap;
use libp2p::{futures::StreamExt, kad::QueryId, swarm::Swarm};
use std::sync::Arc;
use tokio::{select, sync::mpsc};

pub(crate) mod channel;
pub(crate) mod event;
pub(crate) mod swarm_event;

pub(crate) use event::Event;

type P2PSender = channel::BoundedChannelSender<swarm_event::FoundEvent>;

#[async_trait]
pub(crate) trait Handler<THandlerErr, DB>
where
    DB: Database,
{
    #[cfg(not(feature = "ipfs"))]
    async fn handle_event(self, event_loop: &mut EventHandler<DB>);
    #[cfg(feature = "ipfs")]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>, ipfs: IpfsCli);
}

/// Event loop handler for [libp2p] network events and commands.
#[allow(dead_code)]
#[allow(missing_debug_implementations)]
pub struct EventHandler<DB: Database> {
    db: DB,
    sender: Arc<mpsc::Sender<Event>>,
    receiver: mpsc::Receiver<Event>,
    receipt_quorum: usize,
    swarm: Swarm<ComposedBehaviour>,
    workflow_quorum: usize,
    worker_swarm_senders: FnvHashMap<QueryId, P2PSender>,
}

impl<DB> EventHandler<DB>
where
    DB: Database,
{
    fn setup_channel(settings: &settings::Node) -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
        mpsc::channel(settings.network.events_buffer_len)
    }

    /// Create an [EventHandler] with channel sender/receiver defaults.
    pub(crate) fn new(swarm: Swarm<ComposedBehaviour>, db: DB, settings: &settings::Node) -> Self {
        let (sender, receiver) = Self::setup_channel(settings);
        Self {
            db,
            sender: Arc::new(sender),
            receiver,
            receipt_quorum: settings.network.receipt_quorum,
            swarm,
            workflow_quorum: settings.network.workflow_quorum,
            worker_swarm_senders: FnvHashMap::default(),
        }
    }

    /// Sequence for shutting down [EventHandler].
    pub(crate) async fn shutdown(&mut self) {
        self.receiver.close();
        self.sender.closed().await
    }

    /// Get a [Arc]'ed copy of the [EventHandler] channel sender.
    pub(crate) fn sender(&self) -> Arc<mpsc::Sender<Event>> {
        self.sender.clone()
    }

    /// Start [EventHandler] that matches on swarm and pubsub [events].
    ///
    /// [events]: libp2p::swarm::SwarmEvent
    #[cfg(not(feature = "ipfs"))]
    pub(crate) async fn start(mut self) -> Result<()> {
        loop {
            select! {
                swarm_event = self.swarm.select_next_some() =>
                    swarm_event.handle_event(&mut self).await,
                runtime_event = self.receiver.recv() =>
                    if let Some(ev) = runtime_event { ev.handle_event(&mut self).await },
            }
        }
    }
    /// Start [EventHandler] that matches on swarm and pubsub [events].
    ///
    /// [events]: libp2p::swarm::SwarmEvent
    #[cfg(feature = "ipfs")]
    pub(crate) async fn start(mut self, ipfs: IpfsCli) -> Result<()> {
        loop {
            select! {
                swarm_event = self.swarm.select_next_some() =>
                    swarm_event.handle_event(&mut self, ipfs.clone()).await,
                runtime_event = self.receiver.recv() =>
                    if let Some(ev) = runtime_event { ev.handle_event(&mut self, ipfs.clone()).await },
            }
        }
    }
}
