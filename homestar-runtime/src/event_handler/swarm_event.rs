//! Internal libp2p [SwarmEvent] handling and [Handler] implementation.

use super::EventHandler;
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::{Connection, Database},
    event_handler::{
        channel::BoundedChannel,
        event::{PeerRequest, QueryRecord},
        Event, Handler, RequestResponseError,
    },
    network::swarm::{CapsuleTag, ComposedEvent, RequestResponseKey, HOMESTAR_PROTOCOL_VER},
    receipt::{RECEIPT_TAG, VERSION_KEY},
    workflow,
    workflow::WORKFLOW_TAG,
    Db, Receipt,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use homestar_core::{
    consts,
    workflow::{Pointer, Receipt as InvocationReceipt},
};
use libipld::{Cid, Ipld};
use libp2p::{
    gossipsub, identify, kad,
    kad::{
        AddProviderOk, BootstrapOk, GetProvidersOk, GetRecordOk, KademliaEvent, PeerRecord,
        PutRecordOk, QueryResult,
    },
    mdns,
    multiaddr::Protocol,
    rendezvous::{self, Namespace},
    request_response,
    swarm::{dial_opts::DialOpts, SwarmEvent},
    PeerId, StreamProtocol,
};
use std::{collections::HashSet, fmt, time::Instant};
use tracing::{debug, error, info, warn};

const RENDEZVOUS_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/rendezvous/1.0.0");
const RENDEZVOUS_NAMESPACE: &str = "homestar";

/// Internal events within the [SwarmEvent] context related to finding results
/// on the DHT.
#[derive(Debug)]
pub(crate) enum ResponseEvent {
    /// Found [PeerRecord] on the DHT.
    Found(Result<FoundEvent>),
    /// Found Providers/[PeerId]s on the DHT.
    Providers(Result<HashSet<PeerId>>),
}

/// Internal events within the [SwarmEvent] context related to finding specific
/// data items on the DHT.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FoundEvent {
    /// Found [Receipt] on the DHT.
    Receipt(Receipt),
    /// Found [workflow::Info] on the DHT.
    Workflow(workflow::Info),
}

/// Trait for handling [PeerRecord]s found on the DHT.
trait FoundRecord {
    fn found_record(&self) -> Result<FoundEvent>;
}

impl FoundRecord for PeerRecord {
    fn found_record(&self) -> Result<FoundEvent> {
        let key_cid = Cid::try_from(self.record.key.as_ref())?;
        decode_capsule(key_cid, &self.record.value)
    }
}

#[async_trait]
impl<THandlerErr, DB> Handler<THandlerErr, DB> for SwarmEvent<ComposedEvent, THandlerErr>
where
    THandlerErr: fmt::Debug + Send,
    DB: Database + Sync,
{
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>, _ipfs: IpfsCli) {
        handle_swarm_event(self, event_handler).await
    }

    #[cfg(not(feature = "ipfs"))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>) {
        handle_swarm_event(self, event_handler).await
    }
}

async fn handle_swarm_event<THandlerErr: fmt::Debug + Send, DB: Database>(
    event: SwarmEvent<ComposedEvent, THandlerErr>,
    event_handler: &mut EventHandler<DB>,
) {
    match event {
        SwarmEvent::Behaviour(ComposedEvent::Identify(identify_event)) => {
            match identify_event {
                identify::Event::Error { peer_id, error } => {
                    warn!(err=?error, peer_id=peer_id.to_string(), "error while attempting to identify the remote")
                }
                identify::Event::Sent { peer_id } => {
                    debug!(peer_id = peer_id.to_string(), "sent identify info to peer")
                }
                identify::Event::Received { peer_id, info } => {
                    debug!(peer_id=peer_id.to_string(), info=?info, "identify info received from peer");

                    let num_addresses = event_handler.swarm.external_addresses().count();

                    // don't add an address we already have
                    if !event_handler
                        .swarm
                        .external_addresses()
                        .any(|addr| addr == &info.observed_addr)
                        && num_addresses < event_handler.external_address_limit
                    {
                        info.observed_addr
                            .iter()
                            // if _any_ part of the multiaddr includes a private IP, dont add it to our external address list
                            .filter_map(|proto| match proto {
                                Protocol::Ip4(ip) => Some(ip),
                                _ => None,
                            })
                            .all(|proto| !proto.is_private())
                            // identify observed a potentially valid external address that we weren't aware of.
                            // add it to the addresses we announce to other peers
                            // TODO: have a set of _maybe_ external addresses that we validate with other peers first before adding it
                            .then(|| event_handler.swarm.add_external_address(info.observed_addr));
                    }

                    let behavior = event_handler.swarm.behaviour_mut();

                    // don't bother talking with nodes that aren't running our protocol
                    if info.protocol_version == HOMESTAR_PROTOCOL_VER {
                        debug!(protocol_version=info.protocol_version, "peer was not using our homestar protocol version: {HOMESTAR_PROTOCOL_VER}");
                        return;
                    }

                    // kademlia
                    if info.protocols.contains(&kad::PROTOCOL_NAME) {
                        // add listen addresses to kademlia routing table
                        for addr in info.listen_addrs {
                            behavior.kademlia.add_address(&peer_id, addr);
                        }
                    }

                    // rendezvous
                    // we are good to register self & discover with any node we contact. more peers = more better!
                    if info.protocols.contains(&RENDEZVOUS_PROTOCOL_NAME) {
                        // register self with remote
                        if let Err(err) = behavior.rendezvous_client.register(
                            Namespace::from_static(RENDEZVOUS_NAMESPACE),
                            peer_id,
                            None,
                        ) {
                            warn!(
                                err = format!("{err}"),
                                peer_id = peer_id.to_string(),
                                "failed to register with rendezvous peer"
                            )
                        }
                        // discover other nodes
                        behavior.rendezvous_client.discover(
                            Some(Namespace::from_static(RENDEZVOUS_NAMESPACE)),
                            None,
                            None,
                            peer_id,
                        );
                    }
                }
                identify::Event::Pushed { peer_id } => debug!(
                    peer_id = peer_id.to_string(),
                    "pushed identify info too peer"
                ),
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::RendezvousClient(rendezvous_client_event)) => {
            match rendezvous_client_event {
                rendezvous::client::Event::Discovered {
                    rendezvous_node,
                    registrations,
                    cookie,
                } => {
                    // save cookie for later (when we are hungry for snacks again. yummy.)
                    if cookie.namespace() == Some(&Namespace::from_static(RENDEZVOUS_NAMESPACE)) {
                        event_handler
                            .rendezvous_cookies
                            .insert(rendezvous_node, cookie);

                        // dial discovered peers
                        for registration in registrations {
                            // TODO: do anything with ttl here?
                            let opts = DialOpts::peer_id(registration.record.peer_id())
                                .addresses(registration.record.addresses().to_vec())
                                .condition(libp2p::swarm::dial_opts::PeerCondition::Disconnected)
                                .build();
                            // TODO: we might be dialing too many peers here. Add settings to configure when we stop dialing new peers
                            if let Err(err) = event_handler.swarm.dial(opts) {
                                warn!(err=?err, peer_id=registration.record.peer_id().to_string(), "failed to dial peer discovered through rendezvous")
                            }
                        }
                    } else {
                        // don't add peers that aren't from our namespace
                        warn!(peer_id=rendezvous_node.to_string(), namespace=?cookie.namespace(), "rendezvous peer gave records from an unexpected namespace");
                    }
                }
                rendezvous::client::Event::DiscoverFailed {
                    rendezvous_node,
                    error,
                    ..
                } => {
                    error!(err=?error, peer_id=rendezvous_node.to_string(), "failed to discover peers from rendezvous peer")
                }
                rendezvous::client::Event::Registered {
                    rendezvous_node,
                    ttl,
                    ..
                } => debug!(
                    peer_id = rendezvous_node.to_string(),
                    ttl = ttl,
                    "registered self with rendezvous peer"
                ),
                rendezvous::client::Event::RegisterFailed {
                    rendezvous_node,
                    error,
                    ..
                } => {
                    error!(err=?error, peer_id=rendezvous_node.to_string(), "failed to register self with rendezvous peer")
                }
                rendezvous::client::Event::Expired { peer } => {
                    // re-discover records from peer
                    let cookie = event_handler.rendezvous_cookies.get(&peer).cloned();
                    event_handler
                        .swarm
                        .behaviour_mut()
                        .rendezvous_client
                        .discover(
                            Some(Namespace::from_static(RENDEZVOUS_NAMESPACE)),
                            cookie,
                            None,
                            peer,
                        );
                }
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::RendezvousServer(rendezvous_server_event)) => {
            match rendezvous_server_event {
                rendezvous::server::Event::DiscoverServed { enquirer, .. } => debug!(
                    peer_id = enquirer.to_string(),
                    "served rendezvous discover request to peer"
                ),
                rendezvous::server::Event::DiscoverNotServed { enquirer, error } => {
                    warn!(err=?error, peer_id=enquirer.to_string(), "did not serve rendezvous discover request")
                }
                rendezvous::server::Event::PeerNotRegistered {
                    peer,
                    namespace,
                    error,
                } => {
                    warn!(err=?error, namespace=?namespace, peer_id=peer.to_string(), "did not register peer with rendezvous")
                }
                _ => (),
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::Gossipsub(gossip_event)) => match *gossip_event {
            gossipsub::Event::Message {
                message,
                propagation_source,
                message_id,
            } => match Receipt::try_from(message.data) {
                // TODO: dont fail blindly if we get a non receipt message
                Ok(receipt) => {
                    info!("got message: {receipt} from {propagation_source} with message id: {message_id}");

                    // Store gossiped receipt.
                    let _ = event_handler
                        .db
                        .conn()
                        .as_mut()
                        .map(|conn| Db::store_receipt(receipt, conn));
                }
                Err(err) => info!(err=?err, "cannot handle incoming event message"),
            },
            gossipsub::Event::Subscribed { peer_id, topic } => {
                debug!(
                    peer_id = peer_id.to_string(),
                    topic = topic.to_string(),
                    "subscribed to topic over gossipsub"
                )
            }
            _ => {}
        },
        SwarmEvent::Behaviour(ComposedEvent::Kademlia(
            KademliaEvent::OutboundQueryProgressed { id, result, .. },
        )) => {
            match result {
                QueryResult::Bootstrap(Ok(BootstrapOk { peer, .. })) => {
                    debug!("successfully bootstrapped peer: {peer}")
                }
                QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                    key: _,
                    providers,
                    ..
                })) => {
                    let _ = event_handler.query_senders.remove(&id).map(|(_, sender)| {
                        sender.try_send(ResponseEvent::Providers(Ok(providers)))
                    });

                    // Finish the query. We are only interested in the first
                    // result from a provider.
                    let _ = event_handler
                        .swarm
                        .behaviour_mut()
                        .kademlia
                        .query_mut(&id)
                        .map(|mut query| query.finish());
                }
                QueryResult::GetProviders(Err(err)) => {
                    error!(err=?err, "error retrieving outbound query providers");

                    let _ = event_handler.query_senders.remove(&id).map(|(_, sender)| {
                        sender.try_send(ResponseEvent::Providers(Err(err.into())))
                    });
                }
                QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                    debug!(
                        "found record {:#?}, published by {:?}",
                        peer_record.record.key, peer_record.record.publisher
                    );
                    match peer_record.found_record() {
                        Ok(event) => {
                            debug!("event: {event:#?}");
                            let _ = event_handler.query_senders.remove(&id).map(|(_, sender)| {
                                sender.try_send(ResponseEvent::Found(Ok(event)))
                            });
                        }
                        Err(err) => {
                            error!(err=?err, "error retrieving record");
                            let _ = event_handler
                                .query_senders
                                .remove(&id)
                                .map(|(_, sender)| sender.try_send(ResponseEvent::Found(Err(err))));
                        }
                    }
                }
                QueryResult::GetRecord(Ok(_)) => {}
                QueryResult::GetRecord(Err(err)) => {
                    error!(err=?err, "error retrieving record");

                    // Upon an error, attempt to find the record on the DHT via
                    // a provider if it's a Workflow/Info one.

                    if let Some((
                        RequestResponseKey {
                            cid: cid_str,
                            capsule_tag: CapsuleTag::Workflow,
                        },
                        sender,
                    )) = event_handler.query_senders.remove(&id)
                    {
                        let (tx, rx) = BoundedChannel::oneshot();
                        if let Ok(cid) = Cid::try_from(cid_str.as_str()) {
                            if let Err(err) = event_handler.sender().try_send(Event::GetProviders(
                                QueryRecord::with(cid, CapsuleTag::Workflow, tx),
                            )) {
                                error!(err = ?err, "error opening channel to get providers");
                                let _ = sender.try_send(ResponseEvent::Found(Err(err.into())));
                                return;
                            }

                            match rx
                                .recv_deadline(Instant::now() + event_handler.p2p_provider_timeout)
                            {
                                Ok(ResponseEvent::Providers(Ok(providers))) => {
                                    for peer in providers {
                                        let request = RequestResponseKey::new(
                                            cid_str.to_string(),
                                            CapsuleTag::Workflow,
                                        );
                                        let (tx, _rx) = BoundedChannel::oneshot();
                                        if let Err(err) =
                                            event_handler.sender().try_send(Event::OutboundRequest(
                                                PeerRequest::with(peer, request, tx),
                                            ))
                                        {
                                            error!(err = ?err, "error sending outbound request");
                                            let _ = sender
                                                .try_send(ResponseEvent::Found(Err(err.into())));
                                        }
                                    }
                                }
                                _ => {
                                    let _ = sender.try_send(ResponseEvent::Found(Err(err.into())));
                                }
                            }
                        }
                    } else if let Some((RequestResponseKey { capsule_tag, .. }, sender)) =
                        event_handler.query_senders.remove(&id)
                    {
                        let _ = sender.try_send(ResponseEvent::Found(Err(anyhow!(
                            "not a valid provider record tag: {capsule_tag}"
                        ))));
                    }
                }
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    debug!("successfully put record {key:#?}");
                }
                QueryResult::PutRecord(Err(err)) => {
                    error!("error putting record: {err}")
                }
                QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                    // Currently, we don't send anything to the <worker> channel,
                    // once they key is provided.
                    let _ = event_handler.query_senders.remove(&id);
                    debug!("successfully providing {key:#?}");
                }
                QueryResult::StartProviding(Err(err)) => {
                    // Currently, we don't send anything to the <worker> channel,
                    // once they key is provided.
                    let _ = event_handler.query_senders.remove(&id);
                    error!("error providing key: {:#?}", err.key());
                }
                _ => {}
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
            request_response::Event::Message {
                message,
                peer: _peer,
            },
        )) => match message {
            request_response::Message::Request {
                request, channel, ..
            } => match (
                Cid::try_from(request.cid.as_str()),
                request.capsule_tag.tag(),
            ) {
                (Ok(cid), WORKFLOW_TAG) => {
                    match workflow::Info::gather(
                        cid,
                        event_handler.p2p_provider_timeout,
                        event_handler.sender.clone(),
                        event_handler.db.conn().ok(),
                        None::<fn(Cid, Option<Connection>) -> Result<workflow::Info>>,
                    )
                    .await
                    {
                        Ok(workflow_info) => {
                            if let Ok(bytes) = workflow_info.capsule() {
                                let _ = event_handler
                                    .swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(channel, bytes);
                            } else {
                                let _ = event_handler
                                    .swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(
                                        channel,
                                        RequestResponseError::InvalidCapsule(request)
                                            .encode()
                                            .unwrap_or_default(),
                                    );
                            }
                        }
                        Err(err) => {
                            error!(err=?err, cid=?cid, "error retrieving workflow info");

                            let _ = event_handler
                                .swarm
                                .behaviour_mut()
                                .request_response
                                .send_response(
                                    channel,
                                    RequestResponseError::Timeout(request)
                                        .encode()
                                        .unwrap_or_default(),
                                );
                        }
                    }
                }
                _ => {
                    let _ = event_handler
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(
                            channel,
                            RequestResponseError::Unsupported(request)
                                .encode()
                                .unwrap_or_default(),
                        );
                }
            },
            request_response::Message::Response {
                request_id,
                response,
            } => {
                event_handler
                    .request_response_senders
                    .remove(&request_id)
                    .map(|(RequestResponseKey { cid: key_cid, .. }, sender)| {
                        Cid::try_from(key_cid.as_str()).map(|cid| {
                            decode_capsule(cid, &response)
                                .map(|event| sender.try_send(ResponseEvent::Found(Ok(event))))
                                .map_err(|err| {
                                    error!(err=?err, cid = key_cid,
                                           "error returning capsule for request_id: {request_id}");
                                    sender.try_send(ResponseEvent::Found(Err(err)))
                                })
                        })
                    });
            }
        },
        SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, multiaddr) in list {
                info!(
                    peer_id = peer_id.to_string(),
                    addr = multiaddr.to_string(),
                    "mDNS discovered a new peer"
                );
                let _ = event_handler.swarm.dial(
                    DialOpts::peer_id(peer_id)
                        .addresses(vec![multiaddr])
                        .build(),
                );
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Expired(list))) => {
            for (peer_id, multiaddr) in list {
                info!("mDNS discover peer has expired: {peer_id}");
                if event_handler.swarm.behaviour_mut().mdns.has_node(&peer_id) {
                    event_handler
                        .swarm
                        .behaviour_mut()
                        .kademlia
                        .remove_address(&peer_id, &multiaddr);
                }
            }
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            let local_peer = *event_handler.swarm.local_peer_id();
            info!(
                "local node is listening on {}",
                address.with(Protocol::P2p(local_peer))
            );
        }
        SwarmEvent::IncomingConnection { .. } => {}
        SwarmEvent::ConnectionEstablished {
            peer_id, endpoint, ..
        } => {
            debug!(endpoint=?endpoint, peer_id=peer_id.to_string(), "peer connection established");
            // add peer to connected peers list
            event_handler.connected_peers.insert(peer_id, endpoint);
        }
        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
            info!("peer connection closed {peer_id}, cause: {cause:?}");
            event_handler.connected_peers.remove_entry(&peer_id);
        }
        SwarmEvent::OutgoingConnectionError {
            connection_id,
            peer_id,
            error,
        } => {
            error!(
                err=?error,
                peer_id=peer_id.map(|p| p.to_string()).unwrap_or_default(),
                connection_id=?connection_id,
                "outgoing connection error"
            )
        }
        SwarmEvent::IncomingConnectionError {
            connection_id,
            local_addr,
            send_back_addr,
            error,
        } => {
            error!(
                err=?error,
                connection_id=?connection_id,
                local_address=local_addr.to_string(),
                remote_address=send_back_addr.to_string(),
                "incoming connection error"
            )
        }
        SwarmEvent::ListenerError { listener_id, error } => {
            error!(err=?error, listener_id=?listener_id, "listener error")
        }
        SwarmEvent::Dialing { .. } => todo!(),
        e => debug!(e=?e, "uncaught event"),
    }
}

fn decode_capsule(key_cid: Cid, value: &Vec<u8>) -> Result<FoundEvent> {
    // If it decodes to an error, return the error.
    if let Ok((decoded_error, _)) = RequestResponseError::decode(value) {
        return Err(anyhow!("value returns an error: {decoded_error}"));
    };

    match serde_ipld_dagcbor::de::from_reader(&**value) {
        Ok(Ipld::Map(mut map)) => match map.pop_first() {
            Some((code, Ipld::Map(mut rest))) if code == RECEIPT_TAG => {
                if rest.remove(VERSION_KEY)
                    == Some(Ipld::String(consts::INVOCATION_VERSION.to_string()))
                {
                    let invocation_receipt = InvocationReceipt::try_from(Ipld::Map(rest))?;
                    let receipt = Receipt::try_with(Pointer::new(key_cid), &invocation_receipt)?;
                    Ok(FoundEvent::Receipt(receipt))
                } else {
                    Err(anyhow!(
                        "record version mismatch, current version: {}",
                        consts::INVOCATION_VERSION
                    ))
                }
            }
            Some((code, Ipld::Map(rest))) if code == WORKFLOW_TAG => {
                let workflow_info = workflow::Info::try_from(Ipld::Map(rest))?;
                Ok(FoundEvent::Workflow(workflow_info))
            }
            Some((code, _)) => Err(anyhow!("decode mismatch: {code} is not known")),
            None => Err(anyhow!("invalid record value")),
        },
        Ok(ipld) => Err(anyhow!(
            "decode mismatch: expected an Ipld map, got {ipld:#?}",
        )),
        Err(err) => {
            error!(error=?err, "error deserializing record value");
            Err(anyhow!("error deserializing record value"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils, workflow};
    use homestar_core::{
        ipld::DagCbor,
        test_utils::workflow as workflow_test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
        Workflow,
    };
    use homestar_wasm::io::Arg;
    use libp2p::{kad::Record, PeerId};

    #[test]
    fn found_receipt_record() {
        let (invocation_receipt, receipt) = test_utils::receipt::receipts();
        let instruction_bytes = receipt.instruction_cid_as_bytes();
        let bytes = Receipt::invocation_capsule(&invocation_receipt).unwrap();
        let record = Record::new(instruction_bytes, bytes);
        let peer_record = PeerRecord {
            record,
            peer: Some(PeerId::random()),
        };
        if let FoundEvent::Receipt(found_receipt) = peer_record.found_record().unwrap() {
            assert_eq!(found_receipt, receipt);
        } else {
            panic!("Incorrect event type")
        }
    }

    #[test]
    fn found_workflow_record() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            workflow_test_utils::related_wasm_instructions::<Arg>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1.clone()),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.into(),
            UcanPrf::default(),
        );

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let stored_info = workflow::Stored::default(
            Pointer::new(workflow.clone().to_cid().unwrap()),
            workflow.len() as i32,
        );
        let workflow_info = workflow::Info::default(stored_info);
        let workflow_cid_bytes = workflow_info.cid_as_bytes();
        let bytes = workflow_info.capsule().unwrap();
        let record = Record::new(workflow_cid_bytes, bytes);
        let peer_record = PeerRecord {
            record,
            peer: Some(PeerId::random()),
        };
        if let FoundEvent::Workflow(found_workflow) = peer_record.found_record().unwrap() {
            assert_eq!(found_workflow, workflow_info);
        } else {
            panic!("Incorrect event type")
        }
    }
}
