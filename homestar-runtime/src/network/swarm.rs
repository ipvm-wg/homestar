#![allow(missing_docs)]

//! Sets up a [libp2p] [Swarm], containing the state of the network and the way
//! it should behave.

use crate::{network::pubsub, settings, Receipt, RECEIPT_TAG, WORKFLOW_TAG};
use anyhow::{anyhow, Context, Result};
use enum_assoc::Assoc;
use libp2p::{
    core::upgrade,
    gossipsub::{self, MessageId, SubscriptionError, TopicHash},
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent},
    mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm, SwarmBuilder},
    tcp, yamux, StreamProtocol, Transport,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Build a new [Swarm] with a given transport and a tokio executor.
pub(crate) async fn new(settings: &settings::Node) -> Result<Swarm<ComposedBehaviour>> {
    let keypair = settings
        .network
        .keypair_config
        .keypair()
        .with_context(|| "Failed to generate/import keypair for libp2p".to_string())?;

    let peer_id = keypair.public().to_peer_id();

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
            mdns: mdns::Behaviour::new(
                mdns::Config {
                    ttl: settings.network.mdns_ttl,
                    query_interval: settings.network.mdns_query_interval,
                    enable_ipv6: settings.network.mdns_enable_ipv6,
                },
                peer_id,
            )?,
        },
        peer_id,
    )
    .build();

    // Listen-on given address
    swarm.listen_on(settings.network.listen_address.to_string().parse()?)?;

    // subscribe to `receipts` topic
    swarm
        .behaviour_mut()
        .gossip_subscribe(pubsub::RECEIPTS_TOPIC)?;

    Ok(swarm)
}

/// Key data structure for [request_response::Event] messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
