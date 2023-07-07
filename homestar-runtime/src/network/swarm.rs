//! Sets up a [libp2p] [Swarm], containing the state of the network and the way
//! it should behave.

use crate::{network::pubsub, settings, Receipt};
use anyhow::{anyhow, Context, Result};
use libp2p::{
    core::upgrade,
    gossipsub::{self, MessageId, SubscriptionError, TopicHash},
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent},
    mdns, noise,
    swarm::{NetworkBehaviour, Swarm, SwarmBuilder},
    tcp, yamux, Transport,
};

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
            mdns: mdns::Behaviour::new(mdns::Config::default(), peer_id)?,
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

/// Custom event types to listen for and respond to.
#[derive(Debug)]
pub(crate) enum ComposedEvent {
    /// [gossipsub::Event] event.
    Gossipsub(gossipsub::Event),
    /// [KademliaEvent] event.
    Kademlia(KademliaEvent),
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
#[behaviour(out_event = "ComposedEvent")]
pub(crate) struct ComposedBehaviour {
    /// [gossipsub::Behaviour] behaviour.
    pub(crate) gossipsub: gossipsub::Behaviour,
    /// In-memory [kademlia: Kademlia] behaviour.
    pub(crate) kademlia: Kademlia<MemoryStore>,
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
        ComposedEvent::Gossipsub(event)
    }
}

impl From<KademliaEvent> for ComposedEvent {
    fn from(event: KademliaEvent) -> Self {
        ComposedEvent::Kademlia(event)
    }
}

impl From<mdns::Event> for ComposedEvent {
    fn from(event: mdns::Event) -> Self {
        ComposedEvent::Mdns(event)
    }
}
