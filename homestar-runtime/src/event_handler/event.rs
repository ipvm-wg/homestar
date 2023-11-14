//! Internal [Event] type and [Handler] implementation.

use super::EventHandler;
#[cfg(feature = "websocket-notify")]
use crate::event_handler::notification::{
    self, emit_receipt, EventNotificationTyp, SwarmNotification,
};
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::Database,
    event_handler::{channel::AsyncBoundedChannelSender, Handler, P2PSender, ResponseEvent},
    network::{
        pubsub,
        swarm::{CapsuleTag, RequestResponseKey, TopicMessage},
    },
    workflow, Db, Receipt,
};
use anyhow::Result;
use async_trait::async_trait;
#[cfg(feature = "websocket-notify")]
use homestar_core::workflow::Pointer;
use homestar_core::workflow::Receipt as InvocationReceipt;
use libipld::{Cid, Ipld};
use libp2p::{
    kad::{record::Key, Quorum, Record},
    rendezvous::Namespace,
    PeerId,
};
#[cfg(feature = "websocket-notify")]
use maplit::btreemap;
use std::{collections::HashSet, num::NonZeroUsize, sync::Arc};
#[cfg(all(feature = "ipfs", not(feature = "test-utils")))]
use tokio::runtime::Handle;
use tracing::{error, info, warn};

/// A [Receipt] captured (inner) event.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct Captured {
    /// The captured receipt.
    pub(crate) receipt: Cid,
    /// The captured workflow information.
    pub(crate) workflow: Arc<workflow::Info>,
    /// Additional metadata to event-on along with receipt.
    pub(crate) metadata: Option<Ipld>,
}

/// Replay struct for replaying [Receipt]s for notifications.
///
/// Note: This only receives [Pointer]s to [Receipt]s, and then uses
/// the database to retrieve the [Receipt]s, so as to avoid sending
/// large bytes over the wire/channel.
#[cfg(feature = "websocket-notify")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
#[derive(Debug, Clone)]
pub(crate) struct Replay {
    /// Set of [Pointer]s to [Receipt]s.
    pub(crate) pointers: Vec<Pointer>,
    /// Additional metadata to event-on along with receipt.
    pub(crate) metadata: Option<Ipld>,
}

/// A structured query for finding a [Record] in the DHT and
/// returning to a [P2PSender].
#[derive(Debug, Clone)]
pub(crate) struct QueryRecord {
    /// The record identifier, which is a [Cid].
    pub(crate) cid: Cid,
    /// The record capsule tag, which can be part of a key.
    pub(crate) capsule: CapsuleTag,
    /// The sender waiting for a response from the channel.
    pub(crate) sender: Option<P2PSender>,
}

/// A structured query for finding a [Record] in the DHT and
/// returning to a [P2PSender].
#[derive(Debug, Clone)]
pub(crate) struct PeerRequest {
    /// The peer to send a request to.
    pub(crate) peer: PeerId,
    /// The request key, which is a [Cid].
    pub(crate) request: RequestResponseKey,
    /// The channel to send the response to.
    pub(crate) sender: P2PSender,
}

/// Events to capture.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum Event {
    /// [Receipt] captured event.
    CapturedReceipt(Captured),
    /// [Receipt]s replayed for notifications.
    #[cfg(feature = "websocket-notify")]
    ReplayReceipts(Replay),
    /// General shutdown event.
    Shutdown(AsyncBoundedChannelSender<()>),
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
    /// Provide a record in the DHT, e.g. workflow information.
    ProvideRecord(Cid, Option<P2PSender>, CapsuleTag),
    /// Found Providers/[PeerId]s on the DHT.
    Providers(Result<(HashSet<PeerId>, RequestResponseKey, P2PSender)>),
    /// Register with a rendezvous node.
    RegisterPeer(PeerId),
    /// Discover peers from a rendezvous node.
    DiscoverPeers(PeerId),
    /// TODO
    GetListeners(AsyncBoundedChannelSender<Vec<libp2p::core::Multiaddr>>),
}

const RENDEZVOUS_NAMESPACE: &str = "homestar";

#[allow(unreachable_patterns)]
impl Event {
    async fn handle_info<DB>(self, event_handler: &mut EventHandler<DB>) -> Result<()>
    where
        DB: Database,
    {
        match self {
            Event::CapturedReceipt(captured) => {
                let _ = captured.store_and_notify(event_handler);
            }
            Event::Shutdown(tx) => {
                info!("event_handler server shutting down");
                event_handler.shutdown().await;
                let _ = tx.send(());
            }
            Event::GetListeners(tx) => {
                let _ = tx.send(event_handler.swarm.listeners().cloned().collect());
            }
            Event::FindRecord(record) => record.find(event_handler).await,
            Event::RemoveRecord(record) => record.remove(event_handler).await,
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
            Event::GetProviders(record) => record.get_providers(event_handler).await,
            Event::ProvideRecord(cid, sender, capsule_tag) => {
                let query_id = event_handler
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(Key::new(&cid.to_bytes()))
                    .map_err(anyhow::Error::new)?;

                let key = RequestResponseKey::new(cid.to_string().into(), capsule_tag);

                event_handler.query_senders.insert(query_id, (key, sender));
            }
            Event::Providers(Ok((providers, key, sender))) => {
                for peer in providers {
                    let ev_sender = event_handler.sender();
                    let _ = ev_sender
                        .send_async(Event::OutboundRequest(PeerRequest::with(
                            peer,
                            key.clone(),
                            sender.clone(),
                        )))
                        .await;
                }
            }
            Event::Providers(Err(err)) => {
                error!("failed to find providers: {}", err);
            }
            Event::RegisterPeer(peer_id) => {
                if let Some(rendezvous_client) = event_handler
                    .swarm
                    .behaviour_mut()
                    .rendezvous_client
                    .as_mut()
                {
                    // register self with remote
                    if let Err(err) = rendezvous_client.register(
                        Namespace::from_static(RENDEZVOUS_NAMESPACE),
                        peer_id,
                        Some(event_handler.rendezvous.registration_ttl.as_secs()),
                    ) {
                        warn!(
                            peer_id = peer_id.to_string(),
                            err = format!("{err}"),
                            "failed to register with rendezvous peer"
                        )
                    }
                }
            }
            Event::DiscoverPeers(peer_id) => {
                if let Some(rendezvous_client) = event_handler
                    .swarm
                    .behaviour_mut()
                    .rendezvous_client
                    .as_mut()
                {
                    let cookie = event_handler.rendezvous.cookies.get(&peer_id).cloned();

                    rendezvous_client.discover(
                        Some(Namespace::from_static(RENDEZVOUS_NAMESPACE)),
                        cookie,
                        None,
                        peer_id,
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Captured {
    /// `Captured` structure, containing a [Receipt] and [workflow::Info].
    pub(crate) fn with(
        receipt_cid: Cid,
        workflow: Arc<workflow::Info>,
        metadata: Option<Ipld>,
    ) -> Self {
        Self {
            receipt: receipt_cid,
            workflow,
            metadata,
        }
    }

    #[allow(dead_code)]
    fn store_and_notify<DB>(
        mut self,
        event_handler: &mut EventHandler<DB>,
    ) -> Result<(Cid, InvocationReceipt<Ipld>)>
    where
        DB: Database,
    {
        let receipt = Db::find_receipt_by_cid(self.receipt, &mut event_handler.db.conn()?)?;
        let invocation_receipt = InvocationReceipt::from(&receipt);
        let instruction_bytes = receipt.instruction_cid_as_bytes();
        let receipt_cid = receipt.cid();

        #[cfg(feature = "websocket-notify")]
        {
            emit_receipt(
                event_handler.ws_workflow_sender(),
                &receipt,
                self.metadata.to_owned(),
            )
        }

        if event_handler.pubsub_enabled {
            match event_handler.swarm.behaviour_mut().gossip_publish(
                pubsub::RECEIPTS_TOPIC,
                TopicMessage::CapturedReceipt(receipt.clone()),
            ) {
                Ok(msg_id) => {
                    info!(
                        cid = receipt_cid.to_string(),
                        message_id = msg_id.to_string(),
                        "message published on {} topic for receipt with cid: {receipt_cid}",
                        pubsub::RECEIPTS_TOPIC
                    );

                    #[cfg(feature = "websocket-notify")]
                    notification::emit_event(
                        event_handler.ws_evt_sender(),
                        EventNotificationTyp::SwarmNotification(
                            SwarmNotification::PublishedReceiptPubsub,
                        ),
                        btreemap! {
                            "cid" => receipt.cid().to_string(),
                            "ran" => receipt.ran().to_string()
                        },
                    );
                }
                Err(err) => {
                    warn!(
                        err=?err,
                        cid = receipt_cid.to_string(),
                        "message not published on {} topic for receipt",
                        pubsub::RECEIPTS_TOPIC
                    )
                }
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

        if let Ok(receipt_bytes) = Receipt::invocation_capsule(&invocation_receipt) {
            let _id = event_handler
                .swarm
                .behaviour_mut()
                .kademlia
                .put_record(
                    Record::new(instruction_bytes, receipt_bytes.to_vec()),
                    receipt_quorum,
                )
                .map_err(|err| warn!(err=?err, "receipt not PUT on dht"));

            Arc::make_mut(&mut self.workflow).increment_progress(receipt_cid);
            let workflow_cid_bytes = self.workflow.cid_as_bytes();
            if let Ok(workflow_bytes) = self.workflow.capsule() {
                let _id = event_handler
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(
                        Record::new(workflow_cid_bytes, workflow_bytes),
                        workflow_quorum,
                    )
                    .map_err(|err| warn!(err=?err, "workflow information not PUT on dht"));
            } else {
                error!(
                    "cannot convert workflow information {} to bytes",
                    self.workflow.cid()
                );
            }
        } else {
            error!("cannot convert receipt {receipt_cid} to bytes");
        }

        Ok((self.receipt, invocation_receipt))
    }
}

#[cfg(feature = "websocket-notify")]
impl Replay {
    /// `Replay` structure, containing a set of [Pointers] and [Ipld] metadata.
    ///
    /// [Pointers]: Pointer
    pub(crate) fn with(pointers: Vec<Pointer>, metadata: Option<Ipld>) -> Self {
        Self { pointers, metadata }
    }

    fn notify<DB>(self, event_handler: &mut EventHandler<DB>) -> Result<()>
    where
        DB: Database,
    {
        let mut receipts =
            Db::find_instruction_pointers(&self.pointers, &mut event_handler.db.conn()?)?;
        receipts.sort_by_key(|receipt| {
            self.pointers
                .iter()
                .position(|p| p == receipt.instruction())
        });

        #[cfg(debug_assertions)]
        debug_assert_eq!(
            receipts
                .iter()
                .map(|receipt| receipt.instruction())
                .collect::<Vec<_>>(),
            self.pointers.iter().collect::<Vec<_>>()
        );

        #[cfg(feature = "websocket-notify")]
        receipts.iter().for_each(|receipt| {
            emit_receipt(
                event_handler.ws_workflow_sender(),
                receipt,
                self.metadata.to_owned(),
            );
        });

        // gossiping replayed receipts
        if event_handler.pubsub_enabled {
            receipts.into_iter().for_each(|receipt| {
                let receipt_cid = receipt.cid().to_string();
                let _ = event_handler
                    .swarm
                    .behaviour_mut()
                    .gossip_publish(
                        pubsub::RECEIPTS_TOPIC,
                        TopicMessage::CapturedReceipt(receipt.clone()),
                    )
                    .map(|msg_id| {
                         info!(cid=receipt_cid,
                             message_id = msg_id.to_string(),
                             "message published on {} topic for receipt with cid: {receipt_cid}",
                              pubsub::RECEIPTS_TOPIC);

                         #[cfg(feature = "websocket-notify")]
                         notification::emit_event(
                             event_handler.ws_evt_sender(),
                             EventNotificationTyp::SwarmNotification(
                                 SwarmNotification::PublishedReceiptPubsub,
                             ),
                             btreemap! {
                                 "cid" => receipt.cid().to_string(),
                                 "ran" => receipt.ran().to_string()
                             },
                         );
                    })
                    .map_err(
                        |err|
                        warn!(err=?err, cid=receipt_cid,
                              "message not published on {} topic for receipt", pubsub::RECEIPTS_TOPIC),
                    );
            });
        }
        Ok(())
    }
}

impl QueryRecord {
    /// Create a new [QueryRecord] with a [Cid] and [P2PSender].
    pub(crate) fn with(cid: Cid, capsule: CapsuleTag, sender: Option<P2PSender>) -> Self {
        Self {
            cid,
            capsule,
            sender,
        }
    }

    async fn find<DB>(self, event_handler: &mut EventHandler<DB>)
    where
        DB: Database,
    {
        if event_handler.connections.peers.is_empty() {
            info!("no connections to send request to");

            if let Some(sender) = self.sender {
                let _ = sender.send_async(ResponseEvent::NoPeersAvailable).await;
            }

            return;
        }

        let id = event_handler
            .swarm
            .behaviour_mut()
            .kademlia
            .get_record(Key::new(&self.cid.to_bytes()));

        let key = RequestResponseKey::new(self.cid.to_string().into(), self.capsule);
        event_handler.query_senders.insert(id, (key, self.sender));
    }

    async fn remove<DB>(self, event_handler: &mut EventHandler<DB>)
    where
        DB: Database,
    {
        if event_handler.connections.peers.is_empty() {
            info!("no connections to send request to");

            if let Some(sender) = self.sender {
                let _ = sender.send_async(ResponseEvent::NoPeersAvailable).await;
            }

            return;
        }

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

    async fn get_providers<DB>(self, event_handler: &mut EventHandler<DB>)
    where
        DB: Database,
    {
        if event_handler.connections.peers.is_empty() {
            info!("no connections to send request to");

            if let Some(sender) = self.sender {
                let _ = sender.send_async(ResponseEvent::NoPeersAvailable).await;
            }

            return;
        }

        let id = event_handler
            .swarm
            .behaviour_mut()
            .kademlia
            .get_providers(Key::new(&self.cid.to_bytes()));

        let key = RequestResponseKey::new(self.cid.to_string().into(), self.capsule);
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
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    #[allow(unused_variables)]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>, ipfs: IpfsCli) {
        match self {
            Event::CapturedReceipt(captured) => {
                if let Ok((cid, receipt)) = captured.store_and_notify(event_handler) {
                    #[cfg(not(feature = "test-utils"))]
                    {
                        // Spawn client call in the background, without awaiting.
                        let handle = Handle::current();
                        let ipfs = ipfs.clone();
                        handle.spawn(async move {
                        if let Ok(bytes) = receipt.try_into() {
                            match ipfs.put_receipt_bytes(bytes).await {
                                Ok(put_cid) => {
                                    info!(cid = put_cid, "IPLD DAG node stored");
                                    #[cfg(debug_assertions)]
                                    debug_assert_eq!(put_cid, cid.to_string());
                                }
                                Err(err) => {
                                error!(error=?err, cid=cid.to_string(), "failed to store IPLD DAG node");
                                }
                            }
                        } else {
                            warn!(cid=cid.to_string(), "failed to convert receipt to bytes");
                        }
                    });
                    }
                    #[cfg(feature = "test-utils")]
                    info!(cid = cid.to_string(), "cid stored on the network");
                } else {
                    error!("failed to store receipt");
                }
            }
            #[cfg(feature = "websocket-notify")]
            Event::ReplayReceipts(replay) => {
                if let Err(err) = replay.notify(event_handler) {
                    error!(error=?err, "error notifying receipts")
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
