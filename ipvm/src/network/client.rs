//!

use crate::network::{
    eventloop::{Event, EventLoop},
    swarm::{ComposedBehaviour, TopicMessage},
};
use anyhow::Result;
use libp2p::{request_response::ResponseChannel, Multiaddr, PeerId, Swarm};
use std::collections::HashSet;
use tokio::sync::{mpsc, oneshot};

/// A client for interacting with the [libp2p] networking layer.
#[derive(Clone, Debug)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    /// Initialize a client with an event [mpsc::Receiver] and [EventLoop].
    pub async fn new(
        swarm: Swarm<ComposedBehaviour>,
    ) -> Result<(Self, mpsc::Receiver<Event>, EventLoop)> {
        let (command_sender, command_receiver) = mpsc::channel(1);
        let (event_sender, event_receiver) = mpsc::channel(1);

        Ok((
            Client {
                sender: command_sender,
            },
            event_receiver,
            EventLoop::new(swarm, command_receiver, event_sender),
        ))
    }

    /// Publish a [message] to a topic on a running pubsub protocol.
    ///
    /// [message]: TopicMessage
    pub async fn publish_message(&self, topic: &str, msg: TopicMessage) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::PublishMessage {
                msg,
                sender,
                topic: topic.to_string(),
            })
            .await?;
        receiver.await?
    }

    /// Listen for incoming connections on the given address.
    pub async fn start_listening(&mut self, addr: Multiaddr) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await?;
        receiver.await?
    }

    /// Dial the given peer at the given address.
    pub async fn dial(&mut self, peer_id: PeerId, peer_addr: Multiaddr) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await?;
        receiver.await?
    }

    /// Advertise the local node as the provider of the given file on the DHT.
    pub async fn start_providing(&mut self, file_name: String) -> Result<()> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartProviding { file_name, sender })
            .await?;
        receiver.await?
    }

    /// Find the providers for the given file on the DHT.
    pub async fn get_providers(&mut self, file_name: String) -> Result<HashSet<PeerId>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::GetProviders { file_name, sender })
            .await?;
        receiver.await?
    }

    /// Request the content of the given file from the given peer.
    pub async fn request_file(&mut self, peer: PeerId, file_name: String) -> Result<Vec<u8>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestFile {
                file_name,
                peer,
                sender,
            })
            .await?;
        receiver.await?
    }

    /// Respond with the provided file content to the given request.
    pub async fn respond_file(
        &mut self,
        file: Vec<u8>,
        channel: ResponseChannel<FileResponse>,
    ) -> Result<()> {
        self.sender
            .send(Command::RespondFile { file, channel })
            .await?;
        Ok(())
    }
}

/// Wrapper-type for file request name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRequest(pub(crate) String);

/// Wrapper-type for file response content/bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResponse(pub(crate) Vec<u8>);

#[derive(Debug)]
/// [Client] commands.
pub enum Command {
    /// Start listening on an address.
    StartListening {
        /// Address to listen on.
        addr: Multiaddr,
        /// Channel to send on.
        sender: oneshot::Sender<Result<()>>,
    },
    /// Dial a peer in the cluster.
    Dial {
        /// Peer identifier.
        peer_id: PeerId,
        /// Peer address.
        peer_addr: Multiaddr,
        /// Channel to send on.
        sender: oneshot::Sender<Result<()>>,
    },
    /// Start providing content over channel.
    StartProviding {
        /// File.
        file_name: String,
        /// Channel to send on.
        sender: oneshot::Sender<Result<()>>,
    },
    /// Lookup providers for given key (file).
    GetProviders {
        /// File.
        file_name: String,
        /// Channel to send on.
        sender: oneshot::Sender<Result<HashSet<PeerId>>>,
    },
    /// Request file from peer.
    RequestFile {
        /// File.
        file_name: String,
        /// Peer identifier.
        peer: PeerId,
        /// Channel to send on.
        sender: oneshot::Sender<Result<Vec<u8>>>,
    },
    /// Respond with file content.
    RespondFile {
        /// File content.
        file: Vec<u8>,
        /// Channel to send on.
        channel: ResponseChannel<FileResponse>,
    },
    /// Publish message on the topic name.
    PublishMessage {
        /// Pubsub (i.e. [libp2p::gossipsub]) topic.
        topic: String,
        /// [TopicMessage].
        msg: TopicMessage,
        /// Channel to send on.
        sender: oneshot::Sender<Result<()>>,
    },
}
