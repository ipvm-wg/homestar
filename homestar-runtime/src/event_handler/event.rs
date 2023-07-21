//! Internal [Event] type and [Handler] implementation.

use super::EventHandler;
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::{Connection, Database, Db},
    event_handler::{Handler, P2PSender},
    network::{
        pubsub,
        swarm::{CapsuleTag, RequestResponseKey, TopicMessage},
    },
    workflow, Receipt,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use homestar_core::workflow::Receipt as InvocationReceipt;
use libipld::Cid;
use libp2p::{
    kad::{record::Key, Quorum, Record},
    PeerId,
};
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::oneshot;
use tracing::{error, info};

/// A [Receipt] captured (inner) event.
#[derive(Debug, Clone)]
pub struct Captured {
    /// The captured receipt.
    pub(crate) receipt: Receipt,
    /// The captured workflow information.
    pub(crate) workflow: Arc<workflow::Info>,
    /// The sender waiting for a response from the channel.
    pub(crate) sender: P2PSender,
}

/// A structured query for finding a [Record] in the DHT and
/// returning to a [P2PSender].
#[derive(Debug, Clone)]
pub struct QueryRecord {
    /// The record identifier, which is a [Cid].
    pub(crate) cid: Cid,
    /// The record capsule tag, which can be part of a key.
    pub(crate) capsule: CapsuleTag,
    /// The sender waiting for a response from the channel.
    pub(crate) sender: P2PSender,
}

/// A structured query for finding a [Record] in the DHT and
/// returning to a [P2PSender].
#[derive(Debug, Clone)]
pub struct PeerRequest {
    /// The peer to send a request to.
    pub(crate) peer: PeerId,
    /// The request key, which is a [Cid].
    pub(crate) request: RequestResponseKey,
    /// The channel to send the response to.
    pub(crate) sender: P2PSender,
}

/// Internal events to capture.
#[derive(Debug)]
pub enum Event {
    /// [Receipt] captured event.
    CapturedReceipt(Captured),
    /// General shutdown event.
    Shutdown(oneshot::Sender<()>),
    /// Find a [Record] in the DHT, e.g. a [Receipt].
    ///
    /// [Record]: libp2p::kad::Record
    /// [Receipt]: homestar_core::workflow::Receipt
    FindRecord(QueryRecord),
    /// Remove a given record from the DHT, e.g. a [Receipt].
    RemoveRecord(QueryRecord),
    /// Outbound request event to pull data from peers.
    OutboundRequest(PeerRequest),
    /// Get providers for a record in the DHT, e.g. workflow information.
    GetProviders(QueryRecord),
}

impl Event {
    async fn handle_info<DB>(self, event_handler: &mut EventHandler<DB>) -> Result<()>
    where
        DB: Database,
    {
        match self {
            Event::CapturedReceipt(captured) => {
                let mut conn = event_handler.db.conn()?;
                let (cid, _bytes) = captured.store(event_handler, &mut conn)?;
                info!(
                    cid = cid.to_string(),
                    "record replicated with quorum {}", event_handler.receipt_quorum
                );
            }
            Event::Shutdown(tx) => {
                info!("event_handler server shutting down");
                event_handler.shutdown().await;
                let _ = tx.send(());
            }
            Event::FindRecord(record) => record.find(event_handler),
            Event::RemoveRecord(record) => record.remove(event_handler),
            Event::OutboundRequest(PeerRequest {
                peer,
                request,
                sender,
            }) => {
                let request_id = event_handler
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, request.clone());

                event_handler
                    .request_response_senders
                    .insert(request_id, (request, sender));
            }
            Event::GetProviders(record) => record.get_providers(event_handler),
        }
        Ok(())
    }
}

impl Captured {
    /// `Captured` structure, containing a [Receipt] and [workflow::Info].
    pub(crate) fn with(receipt: Receipt, workflow: Arc<workflow::Info>, sender: P2PSender) -> Self {
        Self {
            receipt,
            workflow,
            sender,
        }
    }

    fn store<DB>(
        mut self,
        event_handler: &mut EventHandler<DB>,
        conn: &mut Connection,
    ) -> Result<(Cid, Vec<u8>)>
    where
        DB: Database,
    {
        let receipt_cid = self.receipt.cid();
        let invocation_receipt = InvocationReceipt::from(&self.receipt);
        let instruction_bytes = self.receipt.instruction_cid_as_bytes();
        match event_handler.swarm.behaviour_mut().gossip_publish(
            pubsub::RECEIPTS_TOPIC,
            TopicMessage::CapturedReceipt(self.receipt),
        ) {
            Ok(msg_id) => info!(
                "message {msg_id} published on {} for receipt with cid: {receipt_cid}",
                pubsub::RECEIPTS_TOPIC
            ),
            Err(err) => {
                error!(
                    error=?err, "message not published on {} for receipt with cid: {receipt_cid}",
                    pubsub::RECEIPTS_TOPIC
                )
            }
        }

        let receipt_quorum = if event_handler.receipt_quorum > 0 {
            unsafe { Quorum::N(NonZeroUsize::new_unchecked(event_handler.receipt_quorum)) }
        } else {
            Quorum::One
        };

        let workflow_quorum = if event_handler.workflow_quorum > 0 {
            unsafe { Quorum::N(NonZeroUsize::new_unchecked(event_handler.receipt_quorum)) }
        } else {
            Quorum::One
        };

        if let Ok(receipt_bytes) = Receipt::invocation_capsule(invocation_receipt) {
            let _id = event_handler
                .swarm
                .behaviour_mut()
                .kademlia
                .put_record(
                    Record::new(instruction_bytes, receipt_bytes.to_vec()),
                    receipt_quorum,
                )
                .map_err(anyhow::Error::new)?;

            // Store workflow_receipt join information.
            let _ = Db::store_workflow_receipt(self.workflow.cid, receipt_cid, conn);
            Arc::make_mut(&mut self.workflow).increment_progress(receipt_cid);

            let workflow_cid_bytes = self.workflow.cid_as_bytes();
            let workflow_bytes = self.workflow.capsule()?;

            let query_id = event_handler
                .swarm
                .behaviour_mut()
                .kademlia
                .start_providing(Key::new(&workflow_cid_bytes))
                .map_err(anyhow::Error::new)?;

            let key = RequestResponseKey::new(self.workflow.cid.to_string(), CapsuleTag::Workflow);

            event_handler
                .query_senders
                .insert(query_id, (key, self.sender));

            let _id = event_handler
                .swarm
                .behaviour_mut()
                .kademlia
                .put_record(
                    Record::new(workflow_cid_bytes, workflow_bytes),
                    workflow_quorum,
                )
                .map_err(anyhow::Error::new)?;

            // TODO: Handle Workflow Complete / Num of Tasks finished.

            Ok((receipt_cid, receipt_bytes.to_vec()))
        } else {
            Err(anyhow!("cannot convert receipt {receipt_cid} to bytes"))
        }
    }
}

impl QueryRecord {
    /// Create a new [QueryRecord] with a [Cid] and [P2PSender].
    pub(crate) fn with(cid: Cid, capsule: CapsuleTag, sender: P2PSender) -> Self {
        Self {
            cid,
            capsule,
            sender,
        }
    }

    fn find<DB>(self, event_handler: &mut EventHandler<DB>)
    where
        DB: Database,
    {
        let id = event_handler
            .swarm
            .behaviour_mut()
            .kademlia
            .get_record(Key::new(&self.cid.to_bytes()));

        let key = RequestResponseKey::new(self.cid.to_string(), self.capsule);
        event_handler.query_senders.insert(id, (key, self.sender));
    }

    fn remove<DB>(self, event_handler: &mut EventHandler<DB>)
    where
        DB: Database,
    {
        event_handler
            .swarm
            .behaviour_mut()
            .kademlia
            .remove_record(&Key::new(&self.cid.to_bytes()));

        event_handler
            .swarm
            .behaviour_mut()
            .kademlia
            .stop_providing(&Key::new(&self.cid.to_bytes()));
    }

    fn get_providers<DB>(self, event_handler: &mut EventHandler<DB>)
    where
        DB: Database,
    {
        let id = event_handler
            .swarm
            .behaviour_mut()
            .kademlia
            .get_providers(Key::new(&self.cid.to_bytes()));

        let key = RequestResponseKey::new(self.cid.to_string(), self.capsule);
        event_handler.query_senders.insert(id, (key, self.sender));
    }
}

impl PeerRequest {
    /// Create a new [PeerRequest] with a [PeerId], [RequestResponseKey] and [P2PSender].
    pub(crate) fn with(peer: PeerId, request: RequestResponseKey, sender: P2PSender) -> Self {
        Self {
            peer,
            request,
            sender,
        }
    }
}

#[async_trait]
impl<DB> Handler<(), DB> for Event
where
    DB: Database,
{
    #[cfg(not(feature = "ipfs"))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>) {
        if let Err(err) = self.handle_info(event_handler).await {
            error!(error=?err, "error storing event")
        }
    }

    #[cfg(feature = "ipfs")]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>, ipfs: IpfsCli) {
        match self {
            Event::CapturedReceipt(captured) => {
                if let Err(err) = event_handler.db.conn().map(|mut conn| {
                    captured.store(event_handler, &mut conn).map(|(cid, bytes)| {
                        info!(
                            cid = cid.to_string(),
                            "record replicated with quorum {}", event_handler.receipt_quorum
                        );

                        // Spawn client call in background, without awaiting.
                        tokio::spawn(async move {
                            match ipfs.put_receipt_bytes(bytes.to_vec()).await {
                                Ok(put_cid) => {
                                    info!(cid = put_cid, "IPLD DAG node stored");

                                    #[cfg(debug_assertions)]
                                    debug_assert_eq!(put_cid, cid.to_string());
                                }
                                Err(err) => {
                                    info!(error=?err, cid=cid.to_string(), "Failed to store IPLD DAG node")
                                }
                            }
                        });
                    })
                }) {
                    error!(error=?err, "error storing event")
                }
            }
            event => {
                if let Err(err) = event.handle_info(event_handler).await {
                    error!(error=?err, "error storing event")
                }
            }
        }
    }
}
