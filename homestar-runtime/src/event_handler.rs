//! [EventHandler] implementation for handling network events and messages.

#[cfg(feature = "websocket-server")]
use crate::network::ws;
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::Database,
    network::swarm::{ComposedBehaviour, RequestResponseKey},
    settings,
};
use anyhow::Result;
use async_trait::async_trait;
use fnv::FnvHashMap;
use libp2p::{
    core::ConnectedPoint, futures::StreamExt, kad::QueryId, rendezvous::Cookie,
    request_response::RequestId, swarm::Swarm, PeerId,
};
use std::{sync::Arc, time::Duration};
use swarm_event::ResponseEvent;
use tokio::{select, sync::mpsc};

pub mod channel;
pub(crate) mod error;
pub(crate) mod event;
pub(crate) mod swarm_event;
pub(crate) use error::RequestResponseError;
pub(crate) use event::Event;

type P2PSender = channel::BoundedChannelSender<ResponseEvent>;

/// Handler trait for [EventHandler] events.
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
#[cfg(feature = "websocket-server")]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "websocket-server", feature = "websocket-notify")))
)]
#[allow(missing_debug_implementations, dead_code)]
pub(crate) struct EventHandler<DB: Database> {
    receipt_quorum: usize,
    workflow_quorum: usize,
    p2p_provider_timeout: Duration,
    db: DB,
    swarm: Swarm<ComposedBehaviour>,
    sender: Arc<mpsc::Sender<Event>>,
    receiver: mpsc::Receiver<Event>,
    query_senders: FnvHashMap<QueryId, (RequestResponseKey, P2PSender)>,
    connected_peers: FnvHashMap<PeerId, ConnectedPoint>,
    request_response_senders: FnvHashMap<RequestId, (RequestResponseKey, P2PSender)>,
    rendezvous_cookies: FnvHashMap<PeerId, Cookie>,
    ws_msg_sender: ws::Notifier,
    external_address_limit: usize,
}

/// Event loop handler for [libp2p] network events and commands.
#[cfg(not(feature = "websocket-server"))]
#[allow(missing_debug_implementations, dead_code)]
pub(crate) struct EventHandler<DB: Database> {
    receipt_quorum: usize,
    workflow_quorum: usize,
    p2p_provider_timeout: Duration,
    db: DB,
    swarm: Swarm<ComposedBehaviour>,
    sender: Arc<mpsc::Sender<Event>>,
    receiver: mpsc::Receiver<Event>,
    query_senders: FnvHashMap<QueryId, (RequestResponseKey, P2PSender)>,
    connected_peers: FnvHashMap<PeerId, ConnectedPoint>,
    rendezvous_cookies: FnvHashMap<PeerId, Cookie>,
    request_response_senders: FnvHashMap<RequestId, (RequestResponseKey, P2PSender)>,
    external_address_limit: usize,
}

impl<DB> EventHandler<DB>
where
    DB: Database,
{
    fn setup_channel(settings: &settings::Node) -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
        mpsc::channel(settings.network.events_buffer_len)
    }

    /// Create an [EventHandler] with channel sender/receiver defaults.
    #[cfg(feature = "websocket-server")]
    pub(crate) fn new(
        swarm: Swarm<ComposedBehaviour>,
        db: DB,
        settings: &settings::Node,
        ws_msg_sender: ws::Notifier,
    ) -> Self {
        let (sender, receiver) = Self::setup_channel(settings);
        Self {
            receipt_quorum: settings.network.receipt_quorum,
            workflow_quorum: settings.network.workflow_quorum,
            p2p_provider_timeout: settings.network.p2p_provider_timeout,
            db,
            swarm,
            sender: Arc::new(sender),
            receiver,
            query_senders: FnvHashMap::default(),
            request_response_senders: FnvHashMap::default(),
            connected_peers: FnvHashMap::default(),
            rendezvous_cookies: FnvHashMap::default(),
            ws_msg_sender,
            external_address_limit: settings.network.max_announce_addresses,
        }
    }

    /// Create an [EventHandler] with channel sender/receiver defaults.
    #[cfg(not(feature = "websocket-server"))]
    pub(crate) fn new(swarm: Swarm<ComposedBehaviour>, db: DB, settings: &settings::Node) -> Self {
        let (sender, receiver) = Self::setup_channel(settings);
        Self {
            receipt_quorum: settings.network.receipt_quorum,
            workflow_quorum: settings.network.workflow_quorum,
            p2p_provider_timeout: settings.network.p2p_provider_timeout,
            db,
            swarm,
            sender: Arc::new(sender),
            receiver,
            query_senders: FnvHashMap::default(),
            connected_peers: FnvHashMap::default(),
            rendezvous_cookies: FnvHashMap::default(),
            request_response_senders: FnvHashMap::default(),
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

    /// [tokio::sync::broadcast::Sender] for sending messages through the
    /// webSocket server to subscribers.
    #[cfg(all(feature = "websocket-server", feature = "websocket-notify"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "websocket-server", feature = "websocket-notify")))
    )]
    #[allow(dead_code)]
    pub(crate) fn ws_sender(&self) -> ws::Notifier {
        self.ws_msg_sender.clone()
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
