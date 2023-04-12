//! [EventLoop] implementation for handling network events and messages, as well
//! as commands for the running [libp2p] node.

use super::swarm::TopicMessage;
#[cfg(feature = "ipfs")]
use crate::IpfsCli;
use crate::{
    db::{Database, Db},
    network::swarm::{ComposedBehaviour, ComposedEvent},
    settings,
    workflow::WorkflowInfo,
    Receipt,
};
use anyhow::{anyhow, Result};
use concat_in_place::veccat;
use crossbeam::channel;
use homestar_core::{
    consts,
    workflow::{Pointer, Receipt as InvocationReceipt},
};
use libipld::Cid;
use libp2p::{
    futures::StreamExt,
    gossipsub,
    kad::{
        record::Key, AddProviderOk, BootstrapOk, GetProvidersOk, GetRecordOk, KademliaEvent,
        PeerRecord, PutRecordOk, QueryId, QueryResult, Quorum, Record,
    },
    mdns,
    multiaddr::Protocol,
    swarm::{Swarm, SwarmEvent},
};
use std::{collections::HashMap, fmt, num::NonZeroUsize, str};
use tokio::sync::mpsc;

/// [Receipt]-related topic for pub(gossip)sub.
///
/// [Receipt]: homestar_core::workflow::receipt
pub const RECEIPTS_TOPIC: &str = "receipts";

type WorkerSender = channel::Sender<(Cid, Receipt)>;

/// Event loop handler for [libp2p] network events and commands.
#[allow(missing_debug_implementations)]
pub struct EventLoop {
    receiver: mpsc::Receiver<Event>,
    receipt_quorum: usize,
    worker_senders: HashMap<QueryId, WorkerSender>,
    swarm: Swarm<ComposedBehaviour>,
}

impl EventLoop {
    /// Setup bounded, MPSC channel for runtime to send and receive internal
    /// events with workers.
    pub fn setup_channel(
        settings: &settings::Node,
    ) -> (mpsc::Sender<Event>, mpsc::Receiver<Event>) {
        mpsc::channel(settings.network.events_buffer_len)
    }

    /// Create an [EventLoop] with channel sender/receiver defaults.
    pub fn new(
        swarm: Swarm<ComposedBehaviour>,
        receiver: mpsc::Receiver<Event>,
        settings: &settings::Node,
    ) -> Self {
        Self {
            receiver,
            receipt_quorum: settings.network.receipt_quorum,
            worker_senders: HashMap::new(),
            swarm,
        }
    }

    /// Loop and select over swarm and pubsub [events] and client [commands].
    ///
    /// [events]: SwarmEvent
    #[cfg(not(feature = "ipfs"))]
    pub async fn run(mut self, db: Db) -> Result<()> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.select_next_some() => self.handle_event(swarm_event, db.clone()).await,
                runtime_event = self.receiver.recv() => if let Some(ev) = runtime_event { self.handle_runtime_event(ev).await },
            }
        }
    }

    /// Loop and select over swarm and pubsub [events].
    ///
    /// [events]: SwarmEvent
    #[cfg(feature = "ipfs")]
    pub async fn run(mut self, db: Db, ipfs: IpfsCli) -> Result<()> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.select_next_some() => self.handle_event(swarm_event, db.clone()).await,
                runtime_event = self.receiver.recv() => if let Some(ev) = runtime_event { self.handle_runtime_event(ev, ipfs.clone()).await },
            }
        }
    }

    #[cfg(not(feature = "ipfs"))]
    async fn handle_runtime_event(&mut self, event: Event) {
        match event {
            Event::CapturedReceipt(receipt, workflow_info) => {
                match self.on_capture(receipt, workflow_info) {
                    Ok((cid, _bytes)) => {
                        tracing::debug!(
                            cid = cid,
                            "record replicated with quorum {}",
                            self.receipt_quorum
                        )
                    }

                    Err(err) => {
                        tracing::error!(error=?err, "error putting record on DHT with quorum {}", self.receipt_quorum)
                    }
                }
            }
            Event::FindReceipt(cid, sender) => self.on_find_receipt(cid, sender),
        }
    }

    #[cfg(feature = "ipfs")]
    async fn handle_runtime_event(&mut self, event: Event, ipfs: IpfsCli) {
        match event {
            Event::CapturedReceipt(receipt, workflow_info) => {
                match self.on_capture(receipt, workflow_info) {
                    Ok((cid, bytes)) => {
                        tracing::debug!(
                            cid = cid,
                            "record replicated with quorum {}",
                            self.receipt_quorum
                        );

                        // Spawn client call in background, without awaiting.
                        tokio::spawn(async move {
                            match ipfs.put_receipt_bytes(bytes.to_vec()).await {
                                Ok(put_cid) => {
                                    tracing::info!(cid = put_cid, "IPLD DAG node stored");

                                    #[cfg(debug_assertions)]
                                    debug_assert_eq!(put_cid, cid);
                                }
                                Err(err) => {
                                    tracing::info!(error=?err, cid=cid, "Failed to store IPLD DAG node")
                                }
                            }
                        });
                    }
                    Err(err) => {
                        tracing::error!(error=?err, "error putting record on DHT with quorum {}", self.receipt_quorum)
                    }
                }
            }
            Event::FindReceipt(cid, sender) => self.on_find_receipt(cid, sender),
        }
    }

    fn on_capture(
        &mut self,
        receipt: Receipt,
        mut workflow_info: WorkflowInfo,
    ) -> Result<(String, Vec<u8>)> {
        let receipt_cid = receipt.cid();
        let invocation_receipt = InvocationReceipt::from(&receipt);
        let instruction_bytes = receipt.instruction_cid_as_bytes();
        match self.swarm.behaviour_mut()
                    .gossip_publish(RECEIPTS_TOPIC, TopicMessage::CapturedReceipt(receipt)) {
                        Ok(msg_id) =>
                            tracing::info!("message {msg_id} published on {RECEIPTS_TOPIC} for receipt with cid: {receipt_cid}"),
                        Err(err) => tracing::error!(error=?err, "message not published on {RECEIPTS_TOPIC} for receipt with cid: {receipt_cid}")
                    }

        let quorum = if self.receipt_quorum > 0 {
            unsafe { Quorum::N(NonZeroUsize::new_unchecked(self.receipt_quorum)) }
        } else {
            Quorum::One
        };

        if let Ok(receipt_bytes) = Vec::try_from(invocation_receipt) {
            let ref_bytes = &receipt_bytes;
            let value = veccat!(consts::INVOCATION_VERSION.as_bytes() ref_bytes);
            let _id = self
                .swarm
                .behaviour_mut()
                .kademlia
                .put_record(Record::new(instruction_bytes, value.to_vec()), quorum)
                .map_err(anyhow::Error::msg)?;

            // increment progress with capture
            workflow_info.increment_progress();
            let _id = self
                .swarm
                .behaviour_mut()
                .kademlia
                .put_record(
                    Record::new(workflow_info.cid.to_bytes(), Vec::try_from(workflow_info)?),
                    quorum,
                )
                .map_err(anyhow::Error::msg)?;

            Ok((receipt_cid, receipt_bytes))
        } else {
            Err(anyhow!("cannot convert receipt {receipt_cid} to bytes"))
        }
    }

    fn on_find_receipt(&mut self, instruction_cid: Cid, sender: WorkerSender) {
        let id = self
            .swarm
            .behaviour_mut()
            .kademlia
            .get_record(Key::new(&instruction_cid.to_bytes()));
        self.worker_senders.insert(id, sender);
    }

    fn on_found_record(key_cid: Cid, value: Vec<u8>) -> Result<Receipt> {
        if value.starts_with(consts::INVOCATION_VERSION.as_bytes()) {
            let receipt_bytes = &value[consts::INVOCATION_VERSION.as_bytes().len()..];
            let invocation_receipt = InvocationReceipt::try_from(receipt_bytes.to_vec())?;
            Receipt::try_with(Pointer::new(key_cid), &invocation_receipt)
        } else {
            Err(anyhow!(
                "receipt version mismatch, current version: {}",
                consts::INVOCATION_VERSION
            ))
        }
    }

    async fn handle_event<THandlerErr: fmt::Debug>(
        &mut self,
        event: SwarmEvent<ComposedEvent, THandlerErr>,
        db: Db,
    ) {
        match event {
            SwarmEvent::Behaviour(ComposedEvent::Gossipsub(gossipsub::Event::Message {
                message,
                propagation_source,
                message_id,
            })) => match Receipt::try_from(message.data) {
                Ok(receipt) => {
                    tracing::info!(
                        "got message: {receipt} from {propagation_source} with message id: {message_id}"
                        );

                    // Store gossiped receipt.
                    let _ = db
                        .conn()
                        .as_mut()
                        .map(|conn| Db::store_receipt(receipt, conn));
                }
                Err(err) => tracing::info!(err=?err, "cannot handle incoming event message"),
            },
            SwarmEvent::Behaviour(ComposedEvent::Gossipsub(gossipsub::Event::Subscribed {
                peer_id,
                topic,
            })) => {
                tracing::debug!("{peer_id} subscribed to topic {topic} over gossipsub")
            }
            SwarmEvent::Behaviour(ComposedEvent::Gossipsub(_)) => {}
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(
                KademliaEvent::OutboundQueryProgressed { id, result, .. },
            )) => match result {
                QueryResult::Bootstrap(Ok(BootstrapOk { peer, .. })) => {
                    tracing::debug!("successfully bootstrapped peer: {peer}")
                }
                QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                    key,
                    providers,
                    ..
                })) => {
                    for peer in providers {
                        tracing::debug!("peer {peer} provides key: {key:#?}");
                    }
                }
                QueryResult::GetProviders(Err(err)) => {
                    tracing::error!("error retrieving outbound query providers: {err}")
                }
                QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(PeerRecord {
                    record:
                        Record {
                            key,
                            value,
                            publisher,
                            ..
                        },
                    ..
                }))) => {
                    tracing::debug!("found record {key:#?}, published by {publisher:?}");
                    if let Ok(cid) = Cid::try_from(key.as_ref()) {
                        match Self::on_found_record(cid, value) {
                            Ok(receipt) => {
                                tracing::info!("found receipt: {receipt}");
                                if let Some(sender) = self.worker_senders.remove(&id) {
                                    let _ = sender.send((cid, receipt));
                                } else {
                                    tracing::error!("error converting key {key:#?} to cid")
                                }
                            }
                            Err(err) => tracing::error!(err=?err, "error retrieving receipt"),
                        }
                    }
                }
                QueryResult::GetRecord(Ok(_)) => {}
                QueryResult::GetRecord(Err(err)) => {
                    tracing::error!("error retrieving record: {err}");
                }
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    tracing::debug!("successfully put record {key:#?}");
                }
                QueryResult::PutRecord(Err(err)) => {
                    tracing::error!("error putting record: {err}")
                }
                QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                    tracing::debug!("successfully put provider record {key:#?}");
                }
                QueryResult::StartProviding(Err(err)) => {
                    tracing::error!("error putting provider record: {err}");
                }
                _ => {}
            },
            SwarmEvent::Behaviour(ComposedEvent::Kademlia(_)) => {}
            SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, _multiaddr) in list {
                    tracing::info!("mDNS discovered a new peer: {peer_id}");

                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {
                    tracing::info!("mDNS discover peer has expired: {peer_id}");

                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let local_peer_id = *self.swarm.local_peer_id();
                tracing::info!(
                    "local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id.into()))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            _ => {}
        }
    }
}

/// Internal events to capture.
#[derive(Debug, Clone)]
pub enum Event {
    /// [Receipt] stored and captured event.
    CapturedReceipt(Receipt, WorkflowInfo),
    /// Find a [Receipt] stored in the DHT.
    ///
    /// [Receipt]: InvocationReceipt
    FindReceipt(Cid, WorkerSender),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn found_record() {
        let (invocation_receipt, receipt) = test_utils::receipt::receipts();
        let instruction_bytes = receipt.instruction_cid_as_bytes();
        let bytes = Vec::try_from(invocation_receipt).unwrap();
        let ref_bytes = &bytes;
        let value = veccat!(consts::INVOCATION_VERSION.as_bytes() ref_bytes);
        let record = Record::new(instruction_bytes, value.to_vec());
        let record_value = record.value;
        let found_receipt =
            EventLoop::on_found_record(Cid::try_from(receipt.instruction()).unwrap(), record_value)
                .unwrap();

        assert_eq!(found_receipt, receipt);
    }
}
