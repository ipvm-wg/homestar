//! Evented notifications emitted to clients.

use crate::{
    network::webserver::{
        notifier::{self, Header, Message, Notifier, SubscriptionTyp},
        SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
    },
    receipt::metadata::{WORKFLOW_KEY, WORKFLOW_NAME_KEY},
    Receipt,
};
use homestar_invocation::{ipld::DagJson, Receipt as InvocationReceipt};
use libipld::Ipld;
use tracing::{debug, warn};

pub(crate) mod network;
pub(crate) mod receipt;
pub(crate) use network::{
    ConnectionClosed, ConnectionEstablished, DiscoverServedRendezvous, DiscoveredMdns,
    DiscoveredRendezvous, GotReceiptDht, GotWorkflowInfoDht, IncomingConnectionError,
    NetworkNotification, NewListenAddr, OutgoingConnectionError, PeerRegisteredRendezvous,
    PublishedReceiptPubsub, PutReceiptDht, PutWorkflowInfoDht, ReceiptQuorumFailureDht,
    ReceiptQuorumSuccessDht, ReceivedReceiptPubsub, ReceivedWorkflowInfo, RegisteredRendezvous,
    SentWorkflowInfo, StatusChangedAutonat, WorkflowInfoQuorumFailureDht,
    WorkflowInfoQuorumSuccessDht, WorkflowInfoSource,
};
pub(crate) use receipt::ReceiptNotification;

/// Send receipt notification as bytes.
pub(crate) fn emit_receipt(
    notifier: Notifier<notifier::Message>,
    receipt: &Receipt,
    metadata: Option<Ipld>,
) {
    let invocation_receipt = InvocationReceipt::from(receipt);
    let receipt_cid = receipt.cid();
    let notification = ReceiptNotification::with(invocation_receipt, receipt_cid, metadata.clone());

    if let Ok(json) = notification.to_json() {
        debug!(
            subject = "notification.receipt",
            category = "notification",
            cid = receipt_cid.to_string(),
            instruction_cid = receipt.instruction().cid().to_string(),
            "emitting receipt to WebSocket"
        );
        if let Some(ipld) = metadata {
            match (ipld.get(WORKFLOW_KEY), ipld.get(WORKFLOW_NAME_KEY)) {
                (Ok(Ipld::Link(cid)), Ok(Ipld::String(name))) => {
                    let header =
                        Header::new(SubscriptionTyp::Cid(*cid), Some((name.to_string()).into()));
                    let _ = notifier.notify(Message::new(header, json));
                }
                (Ok(Ipld::Link(cid)), Err(_err)) => {
                    let header = Header::new(SubscriptionTyp::Cid(*cid), None);
                    let _ = notifier.notify(Message::new(header, json));
                }
                _ => (),
            }
        }
    } else {
        warn!(
            subject = "notification.err",
            category = "notification",
            cid = receipt_cid.to_string(),
            "unable to serialize receipt notification as bytes"
        );
    }
}

/// Send network event notification as bytes.
pub(crate) fn emit_network_event(
    notifier: Notifier<notifier::Message>,
    notification: NetworkNotification,
) {
    let header = Header::new(
        SubscriptionTyp::EventSub(SUBSCRIBE_NETWORK_EVENTS_ENDPOINT.to_string()),
        None,
    );

    if let Ok(json) = notification.to_json() {
        if let Err(err) = notifier.notify(Message::new(header, json)) {
            debug!(
                subject = "notification.err",
                category = "notification",
                err=?err,
                "unable to send notification {:?}",
                notification,
            )
        };
    } else {
        debug!(
            subject = "notification.err",
            category = "notification",
            "unable to serialize event notification as bytes: {:?}",
            notification
        );
    }
}
