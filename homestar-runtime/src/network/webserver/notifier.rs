//! Notifier for broadcasting messages to websocket clients.

use anyhow::Result;
use homestar_core::{ipld::DagJson, workflow::Receipt};
use libipld::{ipld, Cid, Ipld};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Type-wrapper for websocket sender.
#[derive(Debug)]
pub(crate) struct Notifier(Arc<broadcast::Sender<Vec<u8>>>);

impl Clone for Notifier {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Notifier {
    /// Create a new [Notifier].
    pub(crate) fn new(sender: broadcast::Sender<Vec<u8>>) -> Self {
        Self(sender.into())
    }

    /// Get a reference to the inner [broadcast::Sender].
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &Arc<broadcast::Sender<Vec<u8>>> {
        &self.0
    }

    /// Get and take ownership of the inner [broadcast::Sender].
    #[allow(dead_code)]
    pub(crate) fn into_inner(self) -> Arc<broadcast::Sender<Vec<u8>>> {
        self.0
    }

    /// Send a message to all connected websocket clients.
    pub(crate) fn notify(&self, msg: Vec<u8>) -> Result<()> {
        let _ = self.0.send(msg)?;
        Ok(())
    }
}

/// A [Receipt] that is sent out *just* for websocket notifications.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NotifyReceipt(Ipld);

impl NotifyReceipt {
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
