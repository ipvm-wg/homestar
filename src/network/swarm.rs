use crate::network::{
    client::{FileRequest, FileResponse},
    pubsub,
};
use anyhow::Result;
use async_trait::async_trait;
use libp2p::{
    core::upgrade::{self, read_length_prefixed, write_length_prefixed, ProtocolName},
    //floodsub::{Floodsub, FloodsubEvent, Topic},
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    gossipsub::{
        error::{PublishError, SubscriptionError},
        Gossipsub, GossipsubEvent, IdentTopic as Topic, MessageId,
    },
    identity::Keypair,
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent},
    mdns,
    noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm},
    tcp,
    yamux::YamuxConfig,
    Transport,
};
use std::{io, iter};

pub async fn new(keypair: Keypair) -> Result<Swarm<ComposedBehaviour>> {
    let peer_id = keypair.public().to_peer_id();

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&keypair)
                .expect("Signing libp2p-noise static DH keypair failed."),
        )
        .multiplex(YamuxConfig::default())
        .boxed();

    Ok(Swarm::with_tokio_executor(
        transport,
        ComposedBehaviour {
            pubsub: pubsub::new(keypair)?,
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

#[derive(Debug)]
pub enum ComposedEvent {
    PubSub(GossipsubEvent),
    Kademlia(KademliaEvent),
    Mdns(mdns::Event),
    RequestResponse(request_response::Event<FileRequest, FileResponse>),
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedEvent")]
pub struct ComposedBehaviour {
    pub pubsub: Gossipsub,
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
    pub request_response: request_response::Behaviour<FileExchangeCodec>,
}

impl ComposedBehaviour {
    pub fn subscribe(&mut self, topic: &str) -> Result<bool, SubscriptionError> {
        let topic = Topic::new(topic);
        self.pubsub.subscribe(&topic)
    }

    pub fn publish(&mut self, topic: &str, msg: String) -> Result<MessageId, PublishError> {
        let topic = Topic::new(topic);
        let msg_bytes = msg.as_bytes();
        self.pubsub.publish(topic, msg_bytes)
    }
}

impl From<GossipsubEvent> for ComposedEvent {
    fn from(event: GossipsubEvent) -> Self {
        ComposedEvent::PubSub(event)
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

#[derive(Debug, Clone)]
pub struct FileExchangeProtocol();

#[derive(Clone)]
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
