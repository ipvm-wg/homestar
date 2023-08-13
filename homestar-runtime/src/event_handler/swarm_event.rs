//! Internal libp2p [SwarmEvent] handling and [Handler] implementation.

use super::EventHandler;
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::{Connection, Database},
    event_handler::{event::QueryRecord, Event, Handler, RequestResponseError},
    network::swarm::{CapsuleTag, ComposedEvent, RequestResponseKey},
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
    gossipsub,
    kad::{
        AddProviderOk, BootstrapOk, GetProvidersOk, GetRecordOk, KademliaEvent, PeerRecord,
        PutRecordOk, QueryResult,
    },
    mdns,
    multiaddr::Protocol,
    request_response,
    swarm::SwarmEvent,
    PeerId,
};
use std::{collections::HashSet, fmt};
use tracing::{debug, info, warn};

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
        // TODO: add identify for adding compatable kademlia nodes.
        // TODO: use kademlia to discover new gossip nodes.
        SwarmEvent::Behaviour(ComposedEvent::Gossipsub(gossip_event)) => match *gossip_event {
            gossipsub::Event::Message {
                message,
                propagation_source,
                message_id,
            } => match Receipt::try_from(message.data) {
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
                debug!("{peer_id} subscribed to topic {topic} over gossipsub")
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
                    let Some((key, sender)) = event_handler.query_senders.remove(&id) else {
                        return;
                    };

                    if let Some(sender) = sender {
                        let ev_sender = event_handler.sender();
                        let _ = ev_sender
                            .send_async(Event::Providers(Ok((providers, key, sender))))
                            .await;
                    }

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
                    warn!(err=?err, "error retrieving outbound query providers");

                    let Some((_, sender)) = event_handler.query_senders.remove(&id) else {
                        return;
                    };

                    if let Some(sender) = sender {
                        let _ = sender
                            .send_async(ResponseEvent::Providers(Err(err.into())))
                            .await;
                    }
                }
                QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                    debug!(
                        "found record {:#?}, published by {:?}",
                        peer_record.record.key, peer_record.record.publisher
                    );
                    match peer_record.found_record() {
                        Ok(event) => {
                            let Some((_, sender)) = event_handler.query_senders.remove(&id) else {
                                return;
                            };

                            if let Some(sender) = sender {
                                let _ = sender.send_async(ResponseEvent::Found(Ok(event))).await;
                            }
                        }
                        Err(err) => {
                            warn!(err=?err, "error retrieving record");

                            let Some((_, sender)) = event_handler.query_senders.remove(&id) else {
                                return;
                            };

                            if let Some(sender) = sender {
                                let _ = sender.send_async(ResponseEvent::Found(Err(err))).await;
                            }
                        }
                    }
                }
                QueryResult::GetRecord(Ok(_)) => {}
                QueryResult::GetRecord(Err(err)) => {
                    warn!(err=?err, "error retrieving record");

                    // Upon an error, attempt to find the record on the DHT via
                    // a provider if it's a Workflow/Info one.
                    match event_handler.query_senders.remove(&id) {
                        Some((
                            RequestResponseKey {
                                cid: ref cid_str,
                                capsule_tag: CapsuleTag::Workflow,
                            },
                            sender,
                        )) => {
                            let ev_sender = event_handler.sender();
                            if let Ok(cid) = Cid::try_from(cid_str.as_str()) {
                                let _ = ev_sender
                                    .send_async(Event::GetProviders(QueryRecord::with(
                                        cid,
                                        CapsuleTag::Workflow,
                                        sender,
                                    )))
                                    .await;
                            }
                        }
                        Some((RequestResponseKey { capsule_tag, .. }, sender)) => {
                            let _ = event_handler.query_senders.remove(&id);
                            if let Some(sender) = sender {
                                let _ = sender
                                    .send_async(ResponseEvent::Found(Err(anyhow!(
                                        "not a valid provider record tag: {capsule_tag}"
                                    ))))
                                    .await;
                            } else {
                                warn!("not a valid provider record tag: {capsule_tag}",)
                            }
                        }
                        None => {
                            info!("No provider found for outbound query {id:?}")
                        }
                    }
                }
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    debug!("successfully put record {key:#?}");
                }
                QueryResult::PutRecord(Err(err)) => {
                    warn!("error putting record: {err}")
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
                    warn!("error providing key: {:#?}", err.key());
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
                            warn!(err=?err, cid=?cid, "error retrieving workflow info");

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
                if let Some((RequestResponseKey { cid: key_cid, .. }, sender)) =
                    event_handler.request_response_senders.remove(&request_id)
                {
                    if let Ok(cid) = Cid::try_from(key_cid.as_str()) {
                        match decode_capsule(cid, &response) {
                            Ok(event) => {
                                let _ = sender.send_async(ResponseEvent::Found(Ok(event))).await;
                            }
                            Err(err) => {
                                warn!(
                                    err=?err,
                                    cid = key_cid.as_str(),
                                    "error returning capsule for request_id: {request_id}"
                                );

                                let _ = sender.send_async(ResponseEvent::Found(Err(err))).await;
                            }
                        }
                    }
                }
            }
        },
        SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, multiaddr) in list {
                info!("mDNS discovered a new peer: {peer_id}");
                event_handler
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, multiaddr);
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
            // add peer to connected peers list
            event_handler.connected_peers.insert(peer_id, endpoint);
        }
        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
            info!("peer connection closed {peer_id}, cause: {cause:?}");
            event_handler.connected_peers.remove_entry(&peer_id);
        }
        _ => {}
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
            warn!(error=?err, "error deserializing record value");
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
