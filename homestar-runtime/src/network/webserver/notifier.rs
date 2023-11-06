//! Notifier for broadcasting messages to websocket clients.

use anyhow::Result;
use faststr::FastStr;
use homestar_core::{ipld::DagJson, workflow::Receipt};
use libipld::{ipld, Cid, Ipld};
use std::{fmt, sync::Arc};
use tokio::sync::broadcast;

/// Type-wrapper for websocket sender.
#[derive(Debug)]
pub(crate) struct Notifier<T>(Arc<broadcast::Sender<T>>);

impl<T> Clone for Notifier<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Notifier<T>
where
    T: Send + Sync + fmt::Debug + 'static,
{
    /// Create a new [Notifier].
    pub(crate) fn new(sender: broadcast::Sender<T>) -> Self {
        Self(sender.into())
    }

    /// Get a reference to the inner [broadcast::Sender].
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &Arc<broadcast::Sender<T>> {
        &self.0
    }

    /// Get and take ownership of the inner [broadcast::Sender].
    #[allow(dead_code)]
    pub(crate) fn into_inner(self) -> Arc<broadcast::Sender<T>> {
        self.0
    }

    /// Send a message to all connected websocket clients.
    pub(crate) fn notify(&self, msg: T) -> Result<()> {
        let _ = self.0.send(msg)?;
        Ok(())
    }
}

/// Subscription type: either directed via a [Cid] or an event subscription string.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum SubscriptionTyp {
    EventSub(String),
    Cid(Cid),
}

/// A header for a message to be sent to a websocket client.
#[derive(Debug, Clone)]
pub(crate) struct Header {
    pub(crate) subscription: SubscriptionTyp,
    pub(crate) ident: Option<FastStr>,
}

impl Header {
    /// Create a new [Header].
    pub(crate) fn new(sub: SubscriptionTyp, ident: Option<FastStr>) -> Self {
        Self {
            subscription: sub,
            ident,
        }
    }
}

/// A message to be sent to a websocket client, with a header and payload.
#[derive(Debug, Clone)]
pub(crate) struct Message {
    pub(crate) header: Header,
    pub(crate) payload: Vec<u8>,
}

impl Message {
    /// Create a new [Message].
    pub(crate) fn new(header: Header, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    /// TODO
    #[allow(dead_code)]
    pub(crate) fn header(&self) -> &Header {
        &self.header
    }

    /// TODO
    #[allow(dead_code)]
    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// A [Receipt] that is sent out *just* for websocket notifications.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NotifyReceipt(Ipld);

impl NotifyReceipt {
    /// TODO
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &Ipld {
        &self.0
    }

    /// TODO
    #[allow(dead_code)]
    pub(crate) fn into_inner(self) -> Ipld {
        self.0.to_owned()
    }

    pub(crate) fn with(receipt: Receipt<Ipld>, cid: Cid, metadata: Option<Ipld>) -> Self {
        let receipt: Ipld = receipt.into();
        let data = ipld!({
            "receipt": receipt,
            "metadata": metadata.as_ref().map(|m| m.to_owned()).map_or(Ipld::Null, |m| m),
            "receipt_cid": cid,
        });
        NotifyReceipt(data)
    }
}

impl DagJson for NotifyReceipt where Ipld: From<NotifyReceipt> {}

impl From<NotifyReceipt> for Ipld {
    fn from(receipt: NotifyReceipt) -> Self {
        receipt.0
    }
}

impl From<Ipld> for NotifyReceipt {
    fn from(ipld: Ipld) -> Self {
        NotifyReceipt(ipld)
    }
}
