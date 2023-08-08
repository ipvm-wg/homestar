#![allow(missing_docs)]

//! Sets up a [libp2p] [Swarm], containing the state of the network and the way
//! it should behave.

use crate::{network::pubsub, settings, Receipt, RECEIPT_TAG, WORKFLOW_TAG};
use anyhow::{anyhow, Context, Result};
use bincode::{Decode, Encode};
use enum_assoc::Assoc;
use libp2p::{
    core::upgrade,
    gossipsub::{self, MessageId, SubscriptionError, TopicHash},
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent},
    mdns,
    multiaddr::Protocol,
    noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm, SwarmBuilder},
    tcp, yamux, StreamProtocol, Transport,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{info, warn};

/// Build a new [Swarm] with a given transport and a tokio executor.
pub(crate) async fn new(settings: &settings::Node) -> Result<Swarm<ComposedBehaviour>> {
    let keypair = settings
        .network
        .keypair_config
        .keypair()
        .with_context(|| "Failed to generate/import keypair for libp2p".to_string())?;

    let peer_id = keypair.public().to_peer_id();
    info!(peer_id=?peer_id.to_string(), "local peer ID generated");

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&keypair)?)
        .multiplex(yamux::Config::default())
        .timeout(settings.network.transport_connection_timeout)
        .boxed();

    let mut swarm = SwarmBuilder::with_tokio_executor(
        transport,
        ComposedBehaviour {
            gossipsub: pubsub::new(keypair, settings)?,
            kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
            request_response: request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/homestar-exchange/1.0.0"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
            mdns: mdns::Behaviour::new(mdns::Config::default(), peer_id)?,
        },
        peer_id,
    )
    .build();

    startup(&mut swarm, &settings.network)?;

    Ok(swarm)
}

fn startup(swarm: &mut Swarm<ComposedBehaviour>, settings: &settings::Network) -> Result<()> {
    // Listen-on given address
    swarm.listen_on(settings.listen_address.to_string().parse()?)?;

    // Dial trusted nodes specified in settings. Failure here shouldn't halt node startup.
    for trusted_addr in &settings.trusted_node_addresses {
        swarm
            .dial(trusted_addr.clone())
            .map(|_| {
                info!(trusted_address=?trusted_addr, "Successfully dialed configured trusted node")
            })
            // log dial failure and continue
            .map_err(|e| warn!(err=?e, "Failed to dial trusted node"))
            .ok();

        // add node to kademlia routing table
        if let Some(peer_id) = trusted_addr.into_iter().find_map(|proto| match proto {
            Protocol::P2p(peer_id) => Some(peer_id),
            _ => None,
        }) {
            info!(trusted_address=?trusted_addr, "added configured trusted node to kademlia routing table");
            swarm
                .behaviour_mut()
                .kademlia
                .add_address(&peer_id, trusted_addr.clone());
        } else {
            warn!(trusted_address=?trusted_addr, "trusted node address did not include a peer ID. not added to kademlia routing table")
        }
    }

    // join `receipts` topic
    swarm
        .behaviour_mut()
        .gossip_subscribe(pubsub::RECEIPTS_TOPIC)?;

    Ok(())
}

/// Key data structure for [request_response::Event] messages.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub(crate) struct RequestResponseKey {
    pub(crate) cid: String,
    pub(crate) capsule_tag: CapsuleTag,
}

impl RequestResponseKey {
    /// Create a new [RequestResponseKey] with a given [Cid] string and capsule tag.
    ///
    /// [Cid]: libipld::Cid
    pub(crate) fn new(cid: String, capsule_tag: CapsuleTag) -> Self {
        Self { cid, capsule_tag }
    }
}

/// Tag for [RequestResponseKey] to indicate the type of capsule-wrapping.
#[derive(Debug, Clone, Assoc, Serialize, Deserialize, Encode, Decode)]
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
    pub(crate) gossipsub: gossipsub::Behaviour,
    /// In-memory [kademlia: Kademlia] behaviour.
    pub(crate) kademlia: Kademlia<MemoryStore>,
    /// [request_response::Behaviour] CBOR-flavored behaviour.
    pub(crate) request_response: request_response::cbor::Behaviour<RequestResponseKey, Vec<u8>>,
    /// [mdns::tokio::Behaviour] behaviour.
    pub(crate) mdns: mdns::tokio::Behaviour,
}

impl ComposedBehaviour {
    /// Subscribe to [gossipsub] topic.
    pub(crate) fn gossip_subscribe(&mut self, topic: &str) -> Result<bool, SubscriptionError> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.gossipsub.subscribe(&topic)
    }

    /// Serialize [TopicMessage] and publish to [gossipsub] topic.
    pub(crate) fn gossip_publish(&mut self, topic: &str, msg: TopicMessage) -> Result<MessageId> {
        let id_topic = gossipsub::IdentTopic::new(topic);
        // Make this a match once we have other topics.
        let TopicMessage::CapturedReceipt(receipt) = msg;
        let msg_bytes: Vec<u8> = receipt.try_into()?;
        if self
            .gossipsub
            .mesh_peers(&TopicHash::from_raw(topic))
            .peekable()
            .peek()
            .is_some()
        {
            let msg_id = self.gossipsub.publish(id_topic, msg_bytes)?;
            Ok(msg_id)
        } else {
            Err(anyhow!(
                "insufficient peers subscribed to topic {topic} for publishing"
            ))
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
