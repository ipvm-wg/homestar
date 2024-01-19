//! [EventHandler] implementation for handling network events and messages.

#[cfg(feature = "websocket-notify")]
use crate::network::webserver::{self, notifier};
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    channel,
    db::Database,
    network::swarm::{ComposedBehaviour, PeerDiscoveryInfo, RequestResponseKey},
    settings,
};
use anyhow::Result;
use async_trait::async_trait;
use fnv::FnvHashMap;
use libp2p::{
    core::ConnectedPoint, futures::StreamExt, kad::QueryId, rendezvous::Cookie,
    request_response::OutboundRequestId as RequestId, swarm::Swarm, PeerId,
};
use moka::future::Cache;
use std::{sync::Arc, time::Duration};
use swarm_event::ResponseEvent;
use tokio::{runtime::Handle, select};

pub(crate) mod cache;
pub(crate) mod error;
pub(crate) mod event;
#[cfg(feature = "websocket-notify")]
pub(crate) mod notification;
pub(crate) mod swarm_event;
pub(crate) use cache::{setup_cache, CacheValue};
pub(crate) use error::RequestResponseError;
pub(crate) use event::Event;

type P2PSender = channel::AsyncChannelSender<ResponseEvent>;

/// Handler trait for [EventHandler] events.
#[async_trait]
pub(crate) trait Handler<DB>
where
    DB: Database,
{
    #[cfg(not(feature = "ipfs"))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>);
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>, ipfs: IpfsCli);
}

/// Event loop handler for libp2p network events and commands.
#[cfg(feature = "websocket-notify")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
#[allow(missing_debug_implementations, dead_code)]
pub(crate) struct EventHandler<DB: Database> {
    /// Minimum number of peers required to receive a receipt.
    receipt_quorum: usize,
    /// Minimum number of peers required to receive workflow information.
    workflow_quorum: usize,
    /// Timeout for p2p workflow info record requests.
    p2p_workflow_info_timeout: Duration,
    /// Timeout for p2p workflow info record requests from a provider.
    p2p_provider_timeout: Duration,
    /// Accessible database instance.
    db: DB,
    /// [libp2p::swarm::Swarm] swarm instance.
    swarm: Swarm<ComposedBehaviour>,
    /// [moka::future::Cache] instance, used for retry logic.
    cache: Arc<Cache<String, CacheValue>>,
    /// [channel::AsyncChannelSender] for sending [Event]s to the [EventHandler].
    sender: Arc<channel::AsyncChannelSender<Event>>,
    /// [channel::AsyncChannelReceiver] for receiving [Event]s from the [EventHandler].
    receiver: channel::AsyncChannelReceiver<Event>,
    /// [QueryId] to [RequestResponseKey] and [P2PSender] mapping.
    query_senders: FnvHashMap<QueryId, (RequestResponseKey, Option<P2PSender>)>,
    /// [PeerId] to [ConnectedPoint] connections mapping.
    connections: Connections,
    /// [RequestId] to [RequestResponseKey] and [P2PSender] mapping.
    request_response_senders: FnvHashMap<RequestId, (RequestResponseKey, P2PSender)>,
    /// Rendezvous protocol configurations and state (cookies).
    rendezvous: Rendezvous,
    /// Whether or not to enable pubsub.
    pubsub_enabled: bool,
    /// [tokio::sync::broadcast::Sender] for websocket event
    /// notification messages.
    ws_evt_sender: webserver::Notifier<notifier::Message>,
    /// [tokio::sync::broadcast::Sender] for websocket workflow-related
    /// notification messages.
    ws_workflow_sender: webserver::Notifier<notifier::Message>,
    /// [libp2p::Multiaddr] addresses to dial.
    node_addresses: Vec<libp2p::Multiaddr>,
    /// [libp2p::Multiaddr] externally reachable addresses to announce to the network.
    announce_addresses: Vec<libp2p::Multiaddr>,
    /// Maximum number of externally reachable addresses to announce to the network.
    external_address_limit: u32,
    /// Interval for polling the cache for expired entries.
    poll_cache_interval: Duration,
}

/// Event loop handler for libp2p network events and commands.
#[cfg(not(feature = "websocket-notify"))]
#[allow(missing_debug_implementations, dead_code)]
pub(crate) struct EventHandler<DB: Database> {
    /// Minimum number of peers required to receive a receipt.
    receipt_quorum: usize,
    /// Minimum number of peers required to receive workflow information.
    workflow_quorum: usize,
    /// Timeout for p2p workflow info record requests.
    p2p_workflow_info_timeout: Duration,
    /// Timeout for p2p workflow info record requests from a provider.
    p2p_provider_timeout: Duration,
    /// Accesible database instance.
    db: DB,
    /// [libp2p::swarm::Swarm] swarm instance.
    swarm: Swarm<ComposedBehaviour>,
    /// [moka::future::Cache] instance, centered around retry logic.
    cache: Arc<Cache<String, CacheValue>>,
    /// [channel::AsyncChannelReceiver] for receiving [Event]s from the [EventHandler].
    sender: Arc<channel::AsyncChannelSender<Event>>,
    /// [channel::AsyncChannelReceiver] for receiving [Event]s from the [EventHandler].
    receiver: channel::AsyncChannelReceiver<Event>,
    /// [QueryId] to [RequestResponseKey] and [P2PSender] mapping.
    query_senders: FnvHashMap<QueryId, (RequestResponseKey, Option<P2PSender>)>,
    /// [PeerId] to [ConnectedPoint] connections mapping.
    connections: Connections,
    /// [RequestId] to [RequestResponseKey] and [P2PSender] mapping.
    request_response_senders: FnvHashMap<RequestId, (RequestResponseKey, P2PSender)>,
    /// Rendezvous protocol configurations and state (cookies).
    rendezvous: Rendezvous,
    /// Whether or not to enable pubsub.
    pubsub_enabled: bool,
    /// [libp2p::Multiaddr] addresses to dial.
    node_addresses: Vec<libp2p::Multiaddr>,
    /// [libp2p::Multiaddr] externally reachable addresses to announce to the network.
    announce_addresses: Vec<libp2p::Multiaddr>,
    /// Maximum number of externally reachable addresses to announce to the network.
    external_address_limit: u32,
    /// Interval for polling the cache for expired entries.
    poll_cache_interval: Duration,
}

/// Rendezvous protocol configurations and state
struct Rendezvous {
    registration_ttl: Duration,
    discovery_interval: Duration,
    discovered_peers: FnvHashMap<PeerId, PeerDiscoveryInfo>,
    cookies: FnvHashMap<PeerId, Cookie>,
}

// Connected peers configuration and state
struct Connections {
    peers: FnvHashMap<PeerId, ConnectedPoint>,
    max_peers: u32,
}

impl<DB> EventHandler<DB>
where
    DB: Database,
{
    fn setup_channel(
        settings: &settings::Network,
    ) -> (
        channel::AsyncChannelSender<Event>,
        channel::AsyncChannelReceiver<Event>,
    ) {
        channel::AsyncChannel::with(settings.events_buffer_len)
    }

    /// Create an [EventHandler] with channel sender/receiver defaults.
    #[cfg(feature = "websocket-notify")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
    pub(crate) fn new(
        swarm: Swarm<ComposedBehaviour>,
        db: DB,
        settings: &settings::Network,
        ws_evt_sender: webserver::Notifier<notifier::Message>,
        ws_workflow_sender: webserver::Notifier<notifier::Message>,
    ) -> Self {
        let (sender, receiver) = Self::setup_channel(settings);
        let sender = Arc::new(sender);
        Self {
            receipt_quorum: settings.libp2p.dht.receipt_quorum,
            workflow_quorum: settings.libp2p.dht.workflow_quorum,
            p2p_workflow_info_timeout: settings.libp2p.dht.p2p_workflow_info_timeout,
            p2p_provider_timeout: settings.libp2p.dht.p2p_provider_timeout,
            db,
            swarm,
            cache: Arc::new(setup_cache(sender.clone())),
            sender,
            receiver,
            query_senders: FnvHashMap::default(),
            request_response_senders: FnvHashMap::default(),
            connections: Connections {
                peers: FnvHashMap::default(),
                max_peers: settings.libp2p.max_connected_peers,
            },
            rendezvous: Rendezvous {
                registration_ttl: settings.libp2p.rendezvous.registration_ttl,
                discovery_interval: settings.libp2p.rendezvous.discovery_interval,
                discovered_peers: FnvHashMap::default(),
                cookies: FnvHashMap::default(),
            },
            pubsub_enabled: settings.libp2p.pubsub.enable,
            ws_evt_sender,
            ws_workflow_sender,
            node_addresses: settings.libp2p.node_addresses.clone(),
            announce_addresses: settings.libp2p.announce_addresses.clone(),
            external_address_limit: settings.libp2p.max_announce_addresses,
            poll_cache_interval: settings.poll_cache_interval,
        }
    }

    /// Create an [EventHandler] with channel sender/receiver defaults.
    #[cfg(not(feature = "websocket-notify"))]
    pub(crate) fn new(
        swarm: Swarm<ComposedBehaviour>,
        db: DB,
        settings: &settings::Network,
    ) -> Self {
        let (sender, receiver) = Self::setup_channel(settings);
        let sender = Arc::new(sender);
        Self {
            receipt_quorum: settings.libp2p.dht.receipt_quorum,
            workflow_quorum: settings.libp2p.dht.workflow_quorum,
            p2p_workflow_info_timeout: settings.libp2p.dht.p2p_workflow_info_timeout,
            p2p_provider_timeout: settings.libp2p.dht.p2p_provider_timeout,
            db,
            swarm,
            cache: Arc::new(setup_cache(sender.clone())),
            sender,
            receiver,
            query_senders: FnvHashMap::default(),
            request_response_senders: FnvHashMap::default(),
            connections: Connections {
                peers: FnvHashMap::default(),
                max_peers: settings.libp2p.max_connected_peers,
            },
            rendezvous: Rendezvous {
                registration_ttl: settings.libp2p.rendezvous.registration_ttl,
                discovery_interval: settings.libp2p.rendezvous.discovery_interval,
                discovered_peers: FnvHashMap::default(),
                cookies: FnvHashMap::default(),
            },
            pubsub_enabled: settings.libp2p.pubsub.enable,
            node_addresses: settings.libp2p.node_addresses.clone(),
            announce_addresses: settings.libp2p.announce_addresses.clone(),
            external_address_limit: settings.libp2p.max_announce_addresses,
            poll_cache_interval: settings.poll_cache_interval,
        }
    }

    /// Sequence for shutting down [EventHandler].
    pub(crate) async fn shutdown(&mut self) {}

    /// Get a [Arc]'ed copy of the [EventHandler] channel sender.
    pub(crate) fn sender(&self) -> Arc<channel::AsyncChannelSender<Event>> {
        self.sender.clone()
    }

    /// [tokio::sync::broadcast::Sender] for sending workflow-related messages
    /// through the WebSocket server to subscribers.
    #[cfg(feature = "websocket-notify")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
    #[allow(dead_code)]
    pub(crate) fn ws_workflow_sender(&self) -> webserver::Notifier<notifier::Message> {
        self.ws_workflow_sender.clone()
    }

    /// [tokio::sync::broadcast::Sender] for sending event-related messages
    /// through the WebSocket server to subscribers.
    #[cfg(feature = "websocket-notify")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
    #[allow(dead_code)]
    pub(crate) fn ws_evt_sender(&self) -> webserver::Notifier<notifier::Message> {
        self.ws_evt_sender.clone()
    }

    /// Start [EventHandler] that matches on swarm and pubsub [events].
    ///
    /// [events]: libp2p::swarm::SwarmEvent
    #[cfg(not(feature = "ipfs"))]
    pub(crate) async fn start(mut self) -> Result<()> {
        let handle = Handle::current();
        handle.spawn(poll_cache(self.cache.clone(), self.poll_cache_interval));

        loop {
            select! {
                runtime_event = self.receiver.recv_async() => {
                    if let Ok(ev) = runtime_event {
                        let _ = ev.handle_event(&mut self).await;
                    }
                }
                swarm_event = self.swarm.select_next_some() => {
                     swarm_event.handle_event(&mut self).await;

                }
            }
        }
    }
    /// Start [EventHandler] that matches on swarm and pubsub [events].
    ///
    /// [events]: libp2p::swarm::SwarmEvent
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    pub(crate) async fn start(mut self, ipfs: IpfsCli) -> Result<()> {
        let handle = Handle::current();
        handle.spawn(poll_cache(self.cache.clone(), self.poll_cache_interval));

        loop {
            select! {
                runtime_event = self.receiver.recv_async() => {
                    if let Ok(ev) = runtime_event {
                        ev.handle_event(&mut self, ipfs.clone()).await;
                    }
                }
                swarm_event = self.swarm.select_next_some() => {
                    let ipfs_clone = ipfs.clone();
                        swarm_event.handle_event(&mut self, ipfs_clone).await;
                }
            }
        }
    }
}

/// Poll cache for expired entries
async fn poll_cache(cache: Arc<Cache<String, CacheValue>>, poll_interval: Duration) {
    let mut interval = tokio::time::interval(poll_interval);

    loop {
        interval.tick().await;
        cache.run_pending_tasks().await;
    }
}
