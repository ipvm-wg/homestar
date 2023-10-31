//! [EventHandler] implementation for handling network events and messages.

#[cfg(feature = "websocket-notify")]
use crate::network::webserver;
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::Database,
    network::swarm::{ComposedBehaviour, PeerDiscoveryInfo, RequestResponseKey},
    settings,
};
use anyhow::Result;
use async_trait::async_trait;
use fnv::FnvHashMap;
use libp2p::{
    core::ConnectedPoint, futures::StreamExt, kad::QueryId, rendezvous::Cookie,
    request_response::RequestId, swarm::Swarm, PeerId,
};
use moka::future::Cache;
use std::{sync::Arc, time::Duration};
use swarm_event::ResponseEvent;
use tokio::{runtime::Handle, select};

pub(crate) mod cache;
pub mod channel;
pub(crate) mod error;
pub(crate) mod event;
pub(crate) mod swarm_event;
pub(crate) use cache::{setup_cache, CacheValue};
pub(crate) use error::RequestResponseError;
pub(crate) use event::Event;

type P2PSender = channel::AsyncBoundedChannelSender<ResponseEvent>;

/// Handler trait for [EventHandler] events.
#[async_trait]
pub(crate) trait Handler<THandlerErr, DB>
where
    DB: Database,
{
    #[cfg(not(feature = "ipfs"))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>);
    #[cfg(feature = "ipfs")]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>, ipfs: IpfsCli);
}

/// Event loop handler for [libp2p] network events and commands.
#[cfg(feature = "websocket-notify")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
#[allow(missing_debug_implementations, dead_code)]
pub(crate) struct EventHandler<DB: Database> {
    receipt_quorum: usize,
    workflow_quorum: usize,
    p2p_provider_timeout: Duration,
    db: DB,
    swarm: Swarm<ComposedBehaviour>,
    cache: Arc<Cache<String, CacheValue>>,
    sender: Arc<channel::AsyncBoundedChannelSender<Event>>,
    receiver: channel::AsyncBoundedChannelReceiver<Event>,
    query_senders: FnvHashMap<QueryId, (RequestResponseKey, Option<P2PSender>)>,
    connections: Connections,
    request_response_senders: FnvHashMap<RequestId, (RequestResponseKey, P2PSender)>,
    rendezvous: Rendezvous,
    pubsub_enabled: bool,
    ws_msg_sender: webserver::Notifier,
    node_addresses: Vec<libp2p::Multiaddr>,
    announce_addresses: Vec<libp2p::Multiaddr>,
    external_address_limit: u32,
    poll_cache_interval: Duration,
}

/// Event loop handler for [libp2p] network events and commands.
#[cfg(not(feature = "websocket-notify"))]
#[allow(missing_debug_implementations, dead_code)]
pub(crate) struct EventHandler<DB: Database> {
    receipt_quorum: usize,
    workflow_quorum: usize,
    p2p_provider_timeout: Duration,
    db: DB,
    swarm: Swarm<ComposedBehaviour>,
    cache: Cache<String, CacheValue>,
    sender: Arc<channel::AsyncBoundedChannelSender<Event>>,
    receiver: channel::AsyncBoundedChannelReceiver<Event>,
    query_senders: FnvHashMap<QueryId, (RequestResponseKey, Option<P2PSender>)>,
    connections: Connections,
    request_response_senders: FnvHashMap<RequestId, (RequestResponseKey, P2PSender)>,
    rendezvous: Rendezvous,
    pubsub_enabled: bool,
    node_addresses: Vec<libp2p::Multiaddr>,
    announce_addresses: Vec<libp2p::Multiaddr>,
    external_address_limit: u32,
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
        settings: &settings::Node,
    ) -> (
        channel::AsyncBoundedChannelSender<Event>,
        channel::AsyncBoundedChannelReceiver<Event>,
    ) {
        channel::AsyncBoundedChannel::with(settings.network.events_buffer_len)
    }

    /// Create an [EventHandler] with channel sender/receiver defaults.
    #[cfg(feature = "websocket-notify")]
    pub(crate) fn new(
        swarm: Swarm<ComposedBehaviour>,
        db: DB,
        settings: &settings::Node,
        ws_msg_sender: webserver::Notifier,
    ) -> Self {
        let (sender, receiver) = Self::setup_channel(settings);
        let sender = Arc::new(sender);
        Self {
            receipt_quorum: settings.network.receipt_quorum,
            workflow_quorum: settings.network.workflow_quorum,
            p2p_provider_timeout: settings.network.p2p_provider_timeout,
            db,
            swarm,
            cache: Arc::new(setup_cache(sender.clone())),
            sender,
            receiver,
            query_senders: FnvHashMap::default(),
            request_response_senders: FnvHashMap::default(),
            connections: Connections {
                peers: FnvHashMap::default(),
                max_peers: settings.network.max_connected_peers,
            },
            rendezvous: Rendezvous {
                registration_ttl: settings.network.rendezvous_registration_ttl,
                discovery_interval: settings.network.rendezvous_discovery_interval,
                discovered_peers: FnvHashMap::default(),
                cookies: FnvHashMap::default(),
            },
            pubsub_enabled: settings.network.enable_pubsub,
            ws_msg_sender,
            node_addresses: settings.network.node_addresses.clone(),
            announce_addresses: settings.network.announce_addresses.clone(),
            external_address_limit: settings.network.max_announce_addresses,
            poll_cache_interval: settings.network.poll_cache_interval,
        }
    }

    /// Create an [EventHandler] with channel sender/receiver defaults.
    #[cfg(not(feature = "websocket-notify"))]
    pub(crate) fn new(swarm: Swarm<ComposedBehaviour>, db: DB, settings: &settings::Node) -> Self {
        let (sender, receiver) = Self::setup_channel(settings);
        let sender = Arc::new(sender);
        Self {
            receipt_quorum: settings.network.receipt_quorum,
            workflow_quorum: settings.network.workflow_quorum,
            p2p_provider_timeout: settings.network.p2p_provider_timeout,
            db,
            swarm,
            cache: Arc::new(setup_cache(sender.clone())),
            sender,
            receiver,
            query_senders: FnvHashMap::default(),
            request_response_senders: FnvHashMap::default(),
            connections: Connections {
                peers: FnvHashMap::default(),
                max_peers: settings.network.max_connected_peers,
            },
            rendezvous: Rendezvous {
                registration_ttl: settings.network.rendezvous_registration_ttl,
                discovery_interval: settings.network.rendezvous_discovery_interval,
                discovered_peers: FnvHashMap::default(),
                cookies: FnvHashMap::default(),
            },
            pubsub_enabled: settings.network.enable_pubsub,
            node_addresses: settings.network.node_addresses.clone(),
            announce_addresses: settings.network.announce_addresses.clone(),
            external_address_limit: settings.network.max_announce_addresses,
            poll_cache_interval: settings.network.poll_cache_interval,
        }
    }

    /// Sequence for shutting down [EventHandler].
    pub(crate) async fn shutdown(&mut self) {}

    /// Get a [Arc]'ed copy of the [EventHandler] channel sender.
    pub(crate) fn sender(&self) -> Arc<channel::AsyncBoundedChannelSender<Event>> {
        self.sender.clone()
    }

    /// [tokio::sync::broadcast::Sender] for sending messages through the
    /// webSocket server to subscribers.
    #[cfg(feature = "websocket-notify")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
    #[allow(dead_code)]
    pub(crate) fn ws_sender(&self) -> webserver::Notifier {
        self.ws_msg_sender.clone()
    }

    /// Start [EventHandler] that matches on swarm and pubsub [events].
    ///
    /// [events]: libp2p::swarm::SwarmEvent
    #[cfg(not(feature = "ipfs"))]
    pub(crate) async fn start(mut self) -> Result<()> {
        let handle = Handle::current();
        handle.spawn(poll_cache(self.cache.clone()));

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
