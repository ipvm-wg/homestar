use crate::network::{
    client::{Command, FileRequest, FileResponse},
    swarm::{ComposedBehaviour, ComposedEvent},
};
use anyhow::{anyhow, Result};
use either::Either;
use libp2p::{
    core::PeerId,
    futures::StreamExt,
    gossipsub::{error::GossipsubHandlerError, GossipsubEvent},
    kad::{GetProvidersOk, KademliaEvent, QueryId, QueryResult},
    mdns,
    multiaddr::Protocol,
    request_response::{self, RequestId, ResponseChannel},
    swarm::{ConnectionHandlerUpgrErr, Swarm, SwarmEvent},
};
use std::{
    collections::{hash_map, HashMap, HashSet},
    io,
};
use tokio::sync::{mpsc, oneshot};
use void::Void;

type HandlerErr = Either<
    Either<Either<GossipsubHandlerError, io::Error>, Void>,
    ConnectionHandlerUpgrErr<io::Error>,
>;

pub const RECEIPTS_TOPIC: &str = "receipts";

pub struct EventLoop {
    swarm: Swarm<ComposedBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<()>>>,
    pending_start_providing: HashMap<QueryId, oneshot::Sender<Result<()>>>,
    pending_get_providers: HashMap<QueryId, oneshot::Sender<Result<HashSet<PeerId>>>>,
    pending_request_file: HashMap<RequestId, oneshot::Sender<Result<Vec<u8>>>>,
}

impl EventLoop {
    pub fn new(
        swarm: Swarm<ComposedBehaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
            pending_request_file: Default::default(),
        }
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await  ,
                command = self.command_receiver.recv() =>if let Some(c) = command {self.handle_command(c).await}
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<ComposedEvent, HandlerErr>) {
        match event {
            SwarmEvent::Behaviour(ComposedEvent::PubSub(GossipsubEvent::Message {
                message,
                ..
            })) => {
                eprintln!("Got message: '{}'", String::from_utf8_lossy(&message.data),)
            }
            SwarmEvent::Behaviour(ComposedEvent::PubSub(_)) => {}
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result: QueryResult::StartProviding(_),
                    ..
                },
            )) => {
                let sender = self
                    .pending_start_providing
                    .remove(&id)
                    .expect("Completed query to be previously pending.");
                let _ = sender.send(Ok(()));
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed {
                    id,
                    result:
                        QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                            providers, ..
                        })),
                    ..
                },
            )) => {
                let _ = self
                    .pending_get_providers
                    .remove(&id)
                    .expect("Completed query to be previously pending.")
                    .send(Ok(providers));
            }
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(_)) => {}
            SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    eprintln!("mDNS discovered a new peer: {peer_id}");
                    // self.swarm
                    //     .behaviour_mut()
                    //     .pubsub
                    //     .add_node_to_partial_view(peer_id);
                    self.swarm
                        .behaviour_mut()
                        .pubsub
                        .add_explicit_peer(&peer_id);

                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, multiaddr);
                }
            }
            SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, multiaddr) in list {
                    eprintln!("mDNS discover peer has expired: {peer_id}");
                    // self.swarm
                    //     .behaviour_mut()
                    //     .pubsub
                    //     .remove_node_from_partial_view(&peer_id);
                    self.swarm
                        .behaviour_mut()
                        .pubsub
                        .remove_explicit_peer(&peer_id);

                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .remove_address(&peer_id, &multiaddr);
                }
            }
            SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    self.event_sender
                        .send(Event::InboundRequest {
                            request: request.0,
                            channel,
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .pending_request_file
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(Ok(response.0));
                }
            },
            SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    request_id, error, ..
                },
            )) => {
                let _ = self
                    .pending_request_file
                    .remove(&request_id)
                    .expect("Request to still be pending.")
                    .send(Err(anyhow!(error)));
            }
            SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
                request_response::Event::ResponseSent { .. },
            )) => {}
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                eprintln!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id.into()))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(anyhow!(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing(peer_id) => eprintln!("Dialing {peer_id}"),
            e => panic!("{e:?}"),
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::SendMessage { message, sender } => {
                let _ = self.swarm.behaviour_mut().publish(RECEIPTS_TOPIC, message);
                let millis = std::time::Duration::from_millis(1000);
                let _ = sender.send(Ok(()));
                std::thread::sleep(millis);
            }
            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(anyhow!(e))),
                };
            }
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, peer_addr.clone());
                    match self
                        .swarm
                        .dial(peer_addr.with(Protocol::P2p(peer_id.into())))
                    {
                        Ok(()) => {
                            e.insert(sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(anyhow!(e)));
                        }
                    }
                } else {
                    todo!("Already dialing peer.");
                }
            }
            Command::StartProviding { file_name, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(file_name.into_bytes().into())
                    .expect("No store error.");
                self.pending_start_providing.insert(query_id, sender);
            }
            Command::GetProviders { file_name, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(file_name.into_bytes().into());
                self.pending_get_providers.insert(query_id, sender);
            }
            Command::RequestFile {
                file_name,
                peer,
                sender,
            } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, FileRequest(file_name));
                self.pending_request_file.insert(request_id, sender);
            }
            Command::RespondFile { file, channel } => {
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, FileResponse(file))
                    .expect("Connection to peer to be still open.");
            }
        }
    }
}

#[derive(Debug)]
pub enum Event {
    InboundRequest {
        request: String,
        channel: ResponseChannel<FileResponse>,
    },
}
