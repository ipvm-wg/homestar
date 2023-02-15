//! Sets up a [libp2p] [Swarm], containing the state of the network and the way
//! it should behave.

use crate::{
    network::{
        client::{FileRequest, FileResponse},
        pubsub,
    },
    workflow::receipt::{Receipt, SharedReceipt},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use libp2p::{
    core::upgrade::{self, read_length_prefixed, write_length_prefixed, ProtocolName},
    floodsub::{self, Floodsub, FloodsubEvent},
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    gossipsub::{self, error::SubscriptionError, MessageId, TopicHash},
    identity::Keypair,
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent},
    mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm},
    tcp,
    yamux::YamuxConfig,
    Transport,
};
use std::{fmt, io, iter};

/// Build a new [Swarm] with a given transport and a tokio executor.
pub async fn new(keypair: Keypair) -> Result<Swarm<ComposedBehaviour>> {
    let peer_id = keypair.public().to_peer_id();

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&keypair)
                .expect("Signing libp2p-noise static DH keypair failed"),
        )
        .multiplex(YamuxConfig::default())
        .boxed();

    Ok(Swarm::with_tokio_executor(
        transport,
        ComposedBehaviour {
            floodsub: pubsub::new_floodsub(peer_id),
            gossipsub: pubsub::new_gossipsub(keypair)?,
            kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
            mdns: mdns::Behaviour::new(mdns::Config::default(), peer_id)?,
            request_response: request_response::Behaviour::new(
                FileExchangeCodec(),
                iter::once((FileExchangeProtocol(), ProtocolSupport::Full)),
                Default::default(),
            ),
        },
        peer_id,
    ))
}

/// Custom event types to listen for and respond to.
#[derive(Debug)]
pub enum ComposedEvent {
    /// [gossipsub::Event] event.
    Gossipsub(gossipsub::Event),
    /// [floodsub::FloodsubEvent] event.
    Floodsub(FloodsubEvent),
    /// [KademliaEvent] event.
    Kademlia(KademliaEvent),
    /// [mdns::Event] event.
    Mdns(mdns::Event),
    /// [request_response::Event] event.
    RequestResponse(request_response::Event<FileRequest, FileResponse>),
}

/// Message topic.
#[derive(Debug)]
pub struct Topic(String);

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Topic {
    /// Make a [Topic] from a [String].
    pub fn new(s: String) -> Self {
        Topic(s)
    }
}

/// Message types to deliver on a topic.
#[derive(Debug)]
pub enum TopicMessage {
    /// Receipt topic, wrapping [Receipt].
    Receipt(Receipt),
}

/// Custom behaviours for [Swarm].
#[allow(missing_debug_implementations)]
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedEvent")]
pub struct ComposedBehaviour {
    /// [gossipsub::Behaviour] behaviour.
    pub gossipsub: gossipsub::Behaviour,
    /// [floodsub::Floodsub] behaviour.
    pub floodsub: Floodsub,
    /// In-memory [kademlia: Kademlia] behaviour.
    pub kademlia: Kademlia<MemoryStore>,
    /// [mdns::tokio::Behaviour] behaviour.
    pub mdns: mdns::tokio::Behaviour,
    /// [request_response::Behaviour] behaviour.
    pub request_response: request_response::Behaviour<FileExchangeCodec>,
}

impl ComposedBehaviour {
    /// Subscribe to [Floodsub] topic.
    pub fn subscribe(&mut self, topic: &str) -> bool {
        let topic = floodsub::Topic::new(topic);
        self.floodsub.subscribe(topic)
    }

    /// Serialize [TopicMessage] and publish to [Floodsub] topic.
    pub fn publish(&mut self, topic: &str, msg: TopicMessage) -> Result<()> {
        let id_topic = floodsub::Topic::new(topic);
        // Make this an or msg to match on other topics.
        let TopicMessage::Receipt(receipt) = msg;
        let msg_bytes = bincode::serialize(&receipt)
            .map_err(|e| anyhow!("failed to serialize receipt: {e}"))?;

        self.floodsub.publish(id_topic, msg_bytes);
        Ok(())
    }

    /// Subscribe to [gossipsub] topic.
    pub fn gossip_subscribe(&mut self, topic: &str) -> Result<bool, SubscriptionError> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.gossipsub.subscribe(&topic)
    }

    /// Serialize [TopicMessage] and publish to [gossipsub] topic.
    pub fn gossip_publish(&mut self, topic: &str, msg: TopicMessage) -> Result<MessageId> {
        let id_topic = gossipsub::IdentTopic::new(topic);
        // Make this an or msg to match on other topics.
        let TopicMessage::Receipt(receipt) = msg;
        let msg_bytes = bincode::serialize(&SharedReceipt::try_from(receipt)?)
            .map_err(|e| anyhow!("failed to serialize receipt: {e}"))?;
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

impl From<FloodsubEvent> for ComposedEvent {
    fn from(event: FloodsubEvent) -> Self {
        ComposedEvent::Floodsub(event)
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

impl From<request_response::Event<FileRequest, FileResponse>> for ComposedEvent {
    fn from(event: request_response::Event<FileRequest, FileResponse>) -> Self {
        ComposedEvent::RequestResponse(event)
    }
}

/// Simple file-exchange protocol.
#[derive(Debug, Clone)]
pub struct FileExchangeProtocol();

/// File-exchange codec.
#[derive(Debug, Clone)]
pub struct FileExchangeCodec();

impl ProtocolName for FileExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/file-exchange/1".as_bytes()
    }
}

#[async_trait]
impl request_response::codec::Codec for FileExchangeCodec {
    type Protocol = FileExchangeProtocol;
    type Request = FileRequest;
    type Response = FileResponse;

    async fn read_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileRequest(String::from_utf8(vec).unwrap()))
    }

    async fn read_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 500_000_000).await?; // update transfer maximum

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileResponse(vec))
    }

    async fn write_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileRequest(data): FileRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileResponse(data): FileResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
