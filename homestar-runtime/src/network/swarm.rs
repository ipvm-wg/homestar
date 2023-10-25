#![allow(missing_docs)]

//! Sets up a [libp2p] [Swarm], containing the state of the network and the way
//! it should behave.
//!
//! [libp2p]: libp2p
//! [Swarm]: libp2p::Swarm

use crate::{
    network::{error::PubSubError, pubsub},
    settings, Receipt, RECEIPT_TAG, WORKFLOW_TAG,
};
use anyhow::{Context, Result};
use enum_assoc::Assoc;
use faststr::FastStr;
use libp2p::{
    core::upgrade,
    gossipsub::{self, MessageId, TopicHash},
    identify,
    kad::{
        self,
        record::store::{MemoryStore, MemoryStoreConfig},
        Kademlia, KademliaConfig, KademliaEvent,
    },
    mdns,
    multiaddr::Protocol,
    noise, rendezvous,
    request_response::{self, ProtocolSupport},
    swarm::{behaviour::toggle::Toggle, NetworkBehaviour, Swarm, SwarmBuilder},
    tcp, yamux, StreamProtocol, Transport,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{info, warn};

pub(crate) const HOMESTAR_PROTOCOL_VER: &str = "homestar/0.0.1";

/// Build a new [Swarm] with a given transport and a tokio executor.
pub(crate) async fn new(settings: &settings::Node) -> Result<Swarm<ComposedBehaviour>> {
    let keypair = settings
        .network
        .keypair_config
        .keypair()
        .with_context(|| "Failed to generate/import keypair for libp2p".to_string())?;

    let peer_id = keypair.public().to_peer_id();
    info!(peer_id = peer_id.to_string(), "local peer ID generated");

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&keypair)?)
        .multiplex(yamux::Config::default())
        .timeout(settings.network.transport_connection_timeout)
        .boxed();

    let mut swarm = SwarmBuilder::with_tokio_executor(
        transport,
        ComposedBehaviour {
            gossipsub: Toggle::from(if settings.network.enable_pubsub {
                Some(pubsub::new(keypair.clone(), settings)?)
            } else {
                None
            }),
            kademlia: Kademlia::with_config(
                peer_id,
                MemoryStore::with_config(
                    peer_id,
                    MemoryStoreConfig {
                        // TODO: if below a better max, rely on cache-store or
                        // blockstore to fetch result if requested directly.
                        // 2gb at the moment
                        max_value_bytes: 10 * 1024 * 1024,
                        ..Default::default()
                    },
                ),
                {
                    let mut cfg = KademliaConfig::default();
                    cfg.set_max_packet_size(10 * 1024 * 1024);
                    cfg
                },
            ),
            request_response: request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/homestar-exchange/1.0.0"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
            mdns: Toggle::from(if settings.network.enable_mdns {
                Some(mdns::Behaviour::new(
                    mdns::Config {
                        ttl: settings.network.mdns_ttl,
                        query_interval: settings.network.mdns_query_interval,
                        enable_ipv6: settings.network.mdns_enable_ipv6,
                    },
                    peer_id,
                )?)
            } else {
                None
            }),
            rendezvous_client: Toggle::from(if settings.network.enable_rendezvous_client {
                Some(rendezvous::client::Behaviour::new(keypair.clone()))
            } else {
                None
            }),
            rendezvous_server: Toggle::from(if settings.network.enable_rendezvous_server {
                Some(rendezvous::server::Behaviour::new(
                    rendezvous::server::Config::with_min_ttl(
                        rendezvous::server::Config::default(),
                        1,
                    ),
                ))
            } else {
                None
            }),
            identify: identify::Behaviour::new(
                identify::Config::new(HOMESTAR_PROTOCOL_VER.to_string(), keypair.public())
                    .with_agent_version(format!("homestar-runtime/{}", env!("CARGO_PKG_VERSION"))),
            ),
        },
        peer_id,
    )
    .build();

    init(&mut swarm, &settings.network)?;

    Ok(swarm)
}

/// Initialize a [Swarm] with given [settings::Network].
///
/// Steps includes:
/// - Listen on given address.
/// - Dial nodes specified in configuration and add them to kademlia.
/// - Subscribe to `receipts` topic for [gossipsub].
///
/// [gossipsub]: libp2p::gossipsub
pub(crate) fn init(
    swarm: &mut Swarm<ComposedBehaviour>,
    settings: &settings::Network,
) -> Result<()> {
    // Listen-on given address
    swarm.listen_on(settings.listen_address.to_string().parse()?)?;

    // Set Kademlia server mode
    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    // add external addresses from settings
    if !settings.announce_addresses.is_empty() {
        for addr in settings.announce_addresses.iter() {
            swarm.add_external_address(addr.clone());
        }
    } else {
        warn!(
            err = "no addresses to announce to peers defined in settings",
            "node may be unreachable to external peers"
        )
    }

    // Dial nodes specified in settings. Failure here shouldn't halt node startup.
    for (index, addr) in settings.node_addresses.iter().enumerate() {
        if index < settings.max_connected_peers as usize {
            let _ = swarm
                .dial(addr.clone())
                // log dial failure and continue
                .map_err(|e| warn!(err=?e, "failed to dial configured node"));

            // add node to kademlia routing table
            if let Some(Protocol::P2p(peer_id)) =
                addr.iter().find(|proto| matches!(proto, Protocol::P2p(_)))
            {
                info!(addr=?addr, "added configured node to kademlia routing table");
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
            } else {
                warn!(addr=?addr, err="configured node address did not include a peer ID", "node not added to kademlia routing table")
            }
        } else {
            warn!(addr=?addr, "address not dialed because node addresses count exceeds max connected peers configuration")
        }
    }

    if settings.enable_pubsub {
        // join `receipts` topic
        swarm
            .behaviour_mut()
            .gossip_subscribe(pubsub::RECEIPTS_TOPIC)?;
    }

    Ok(())
}

/// Key data structure for [request_response::Event] messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RequestResponseKey {
    pub(crate) cid: FastStr,
    pub(crate) capsule_tag: CapsuleTag,
}

impl RequestResponseKey {
    /// Create a new [RequestResponseKey] with a given [Cid] string and capsule tag.
    ///
    /// [Cid]: libipld::Cid
    pub(crate) fn new(cid: FastStr, capsule_tag: CapsuleTag) -> Self {
        Self { cid, capsule_tag }
    }
}

/// Tag for [RequestResponseKey] to indicate the type of capsule wrapping.
#[derive(Debug, Clone, Assoc, Serialize, Deserialize)]
#[func(pub(crate) fn tag(&self) -> &'static str)]
#[func(pub(crate) fn capsule_type(s: &str) -> Option<Self>)]
pub(crate) enum CapsuleTag {
    /// Receipt capsule-tag-wrapper: [RECEIPT_TAG].
    #[assoc(tag = RECEIPT_TAG)]
    #[assoc(capsule_type = RECEIPT_TAG)]
    Receipt,
    /// Workflow/workflow-info capsule-tag-wrapper: [WORKFLOW_TAG].
    #[assoc(tag = WORKFLOW_TAG)]
    #[assoc(capsule_type = WORKFLOW_TAG)]
    Workflow,
}

impl fmt::Display for CapsuleTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag())
    }
}

/// Custom event types to listen for and respond to.
#[derive(Debug)]
pub(crate) enum ComposedEvent {
    /// [gossipsub::Event] event.
    Gossipsub(Box<gossipsub::Event>),
    /// [KademliaEvent] event.
    Kademlia(KademliaEvent),
    /// [request_response::Event] event.
    RequestResponse(request_response::Event<RequestResponseKey, Vec<u8>>),
    /// [mdns::Event] event.
    Mdns(mdns::Event),
    /// [rendezvous::client::Event] event.
    RendezvousClient(rendezvous::client::Event),
    /// [rendezvous::server::Event] event.
    RendezvousServer(rendezvous::server::Event),
    /// [identify::Event] event.
    Identify(identify::Event),
}

/// Message types to deliver on a topic.
#[derive(Debug)]
pub(crate) enum TopicMessage {
    /// Receipt topic, wrapping [Receipt].
    CapturedReceipt(Receipt),
}

/// Custom behaviours for [Swarm].
#[allow(missing_debug_implementations)]
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedEvent")]
pub(crate) struct ComposedBehaviour {
    /// [gossipsub::Behaviour] behaviour.
    pub(crate) gossipsub: Toggle<gossipsub::Behaviour>,
    /// In-memory [kademlia: Kademlia] behaviour.
    pub(crate) kademlia: Kademlia<MemoryStore>,
    /// [request_response::Behaviour] CBOR-flavored behaviour.
    pub(crate) request_response: request_response::cbor::Behaviour<RequestResponseKey, Vec<u8>>,
    /// [mdns::tokio::Behaviour] behaviour.
    pub(crate) mdns: Toggle<mdns::tokio::Behaviour>,
    /// [rendezvous::client::Behaviour] behaviour.
    pub(crate) rendezvous_client: Toggle<rendezvous::client::Behaviour>,
    /// [rendezvous::server::Behaviour] behaviour.
    pub(crate) rendezvous_server: Toggle<rendezvous::server::Behaviour>,
    /// [identify::Behaviour] behaviour.
    pub(crate) identify: identify::Behaviour,
}

impl ComposedBehaviour {
    /// Subscribe to [gossipsub] topic.
    pub(crate) fn gossip_subscribe(&mut self, topic: &str) -> Result<bool, PubSubError> {
        if let Some(gossipsub) = self.gossipsub.as_mut() {
            let topic = gossipsub::IdentTopic::new(topic);
            let subscribed = gossipsub.subscribe(&topic)?;

            Ok(subscribed)
        } else {
            Err(PubSubError::NotEnabled)
        }
    }

    /// Serialize [TopicMessage] and publish to [gossipsub] topic.
    pub(crate) fn gossip_publish(
        &mut self,
        topic: &str,
        msg: TopicMessage,
    ) -> Result<MessageId, PubSubError> {
        if let Some(gossipsub) = self.gossipsub.as_mut() {
            let id_topic = gossipsub::IdentTopic::new(topic);
            // Make this a match once we have other topics.
            let TopicMessage::CapturedReceipt(receipt) = msg;
            let msg_bytes: Vec<u8> = receipt.try_into()?;
            if gossipsub
                .mesh_peers(&TopicHash::from_raw(topic))
                .peekable()
                .peek()
                .is_some()
            {
                let msg_id = gossipsub.publish(id_topic, msg_bytes)?;
                Ok(msg_id)
            } else {
                Err(PubSubError::InsufficientPeers(topic.to_owned()))
            }
        } else {
            Err(PubSubError::NotEnabled)
        }
    }
}

impl From<gossipsub::Event> for ComposedEvent {
    fn from(event: gossipsub::Event) -> Self {
        ComposedEvent::Gossipsub(Box::new(event))
    }
}

impl From<KademliaEvent> for ComposedEvent {
    fn from(event: KademliaEvent) -> Self {
        ComposedEvent::Kademlia(event)
    }
}

impl From<request_response::Event<RequestResponseKey, Vec<u8>>> for ComposedEvent {
    fn from(event: request_response::Event<RequestResponseKey, Vec<u8>>) -> Self {
        ComposedEvent::RequestResponse(event)
    }
}

impl From<mdns::Event> for ComposedEvent {
    fn from(event: mdns::Event) -> Self {
        ComposedEvent::Mdns(event)
    }
}

impl From<rendezvous::client::Event> for ComposedEvent {
    fn from(event: rendezvous::client::Event) -> Self {
        ComposedEvent::RendezvousClient(event)
    }
}

impl From<rendezvous::server::Event> for ComposedEvent {
    fn from(event: rendezvous::server::Event) -> Self {
        ComposedEvent::RendezvousServer(event)
    }
}

impl From<identify::Event> for ComposedEvent {
    fn from(event: identify::Event) -> Self {
        ComposedEvent::Identify(event)
    }
}
