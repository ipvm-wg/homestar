#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::Database,
    event_handler::Handler,
    network::swarm::ComposedEvent,
    receipt::{RECEIPT_TAG, VERSION_KEY},
    workflow,
    workflow::WORKFLOW_TAG,
    Db, EventHandler, Receipt,
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
    swarm::SwarmEvent,
};
use std::fmt;
use tracing::{debug, error, info};

/// Internal events within the [SwarmEvent] context related to finding results
/// on the DHT.
#[derive(Debug, Clone, PartialEq)]
pub enum FoundEvent {
    /// Found [Receipt] on the DHT.
    Receipt(Receipt),
    /// Found [workflow::Info] on the DHT.
    Workflow(workflow::Info),
}

/// Trait for handling [PeerRecord]s found on the DHT.
pub(crate) trait FoundRecord {
    fn found_record(&self) -> Result<FoundEvent>;
}

impl FoundRecord for PeerRecord {
    fn found_record(&self) -> Result<FoundEvent> {
        let key_cid = Cid::try_from(self.record.key.as_ref())?;
        match serde_ipld_dagcbor::de::from_reader(&*self.record.value) {
            Ok(Ipld::Map(mut map)) => match map.pop_first() {
                Some((code, Ipld::Map(mut rest))) if code == RECEIPT_TAG => {
                    if rest.remove(VERSION_KEY)
                        == Some(Ipld::String(consts::INVOCATION_VERSION.to_string()))
                    {
                        let invocation_receipt = InvocationReceipt::try_from(Ipld::Map(rest))?;
                        let receipt =
                            Receipt::try_with(Pointer::new(key_cid), &invocation_receipt)?;
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
}

#[async_trait]
impl<THandlerErr, DB> Handler<THandlerErr, DB> for SwarmEvent<ComposedEvent, THandlerErr>
where
    THandlerErr: fmt::Debug + Send,
    DB: Database,
{
    #[cfg(feature = "ipfs")]
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
        SwarmEvent::Behaviour(ComposedEvent::Gossipsub(gossipsub::Event::Message {
            message,
            propagation_source,
            message_id,
        })) => match Receipt::try_from(message.data) {
            Ok(receipt) => {
                info!(
                        "got message: {receipt} from {propagation_source} with message id: {message_id}"
                        );

                // Store gossiped receipt.
                let _ = event_handler
                    .db
                    .conn()
                    .as_mut()
                    .map(|conn| Db::store_receipt(receipt, conn));
            }
            Err(err) => info!(err=?err, "cannot handle incoming event message"),
        },
        SwarmEvent::Behaviour(ComposedEvent::Gossipsub(gossipsub::Event::Subscribed {
            peer_id,
            topic,
        })) => {
            debug!("{peer_id} subscribed to topic {topic} over gossipsub")
        }
        SwarmEvent::Behaviour(ComposedEvent::Gossipsub(_)) => {}
        SwarmEvent::Behaviour(ComposedEvent::Kademlia(
            KademliaEvent::OutboundQueryProgressed { id, result, .. },
        )) => match result {
            QueryResult::Bootstrap(Ok(BootstrapOk { peer, .. })) => {
                debug!("successfully bootstrapped peer: {peer}")
            }
            QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                key,
                providers,
                ..
            })) => {
                for peer in providers {
                    debug!("peer {peer} provides key: {key:#?}");
                }
            }
            QueryResult::GetProviders(Err(err)) => {
                error!("error retrieving outbound query providers: {err}")
            }
            QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                debug!(
                    "found record {:#?}, published by {:?}",
                    peer_record.record.key, peer_record.record.publisher
                );
                match peer_record.found_record() {
                    Ok(event) => {
                        info!("event: {event:#?}");
                        if let Some(sender) = event_handler.worker_swarm_senders.remove(&id) {
                            let _ = sender.send(event);
                        } else {
                            error!("error converting key {:#?} to cid", peer_record.record.key)
                        }
                    }
                    Err(err) => error!(err=?err, "error retrieving record"),
                }
            }
            QueryResult::GetRecord(Ok(_)) => {}
            QueryResult::GetRecord(Err(err)) => {
                error!("error retrieving record: {err}");
            }
            QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                debug!("successfully put record {key:#?}");
            }
            QueryResult::PutRecord(Err(err)) => {
                error!("error putting record: {err}")
            }
            QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                debug!("successfully put provider record {key:#?}");
            }
            QueryResult::StartProviding(Err(err)) => {
                error!("error putting provider record: {err}");
            }
            _ => {}
        },
        SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, _multiaddr) in list {
                info!("mDNS discovered a new peer: {peer_id}");

                event_handler
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .add_explicit_peer(&peer_id);
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Expired(list))) => {
            for (peer_id, _multiaddr) in list {
                info!("mDNS discover peer has expired: {peer_id}");

                event_handler
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .remove_explicit_peer(&peer_id);
            }
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            let local_peer_id = *event_handler.swarm.local_peer_id();
            info!(
                "local node is listening on {:?}",
                address.with(Protocol::P2p(local_peer_id.into()))
            );
        }
        SwarmEvent::IncomingConnection { .. } => {}
        _ => {}
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
        let bytes = Receipt::invocation_capsule(invocation_receipt).unwrap();
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
        let workflow_info =
            workflow::Info::default(workflow.clone().to_cid().unwrap(), workflow.len());
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
