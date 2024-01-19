//! Notification receipts.

use homestar_invocation::{ipld::DagJson, Receipt};
use libipld::{ipld, Cid, Ipld};

/// A [Receipt] that is sent out for websocket notifications.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReceiptNotification(Ipld);

impl ReceiptNotification {
    /// Obtain a reference to the inner Ipld value.
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &Ipld {
        &self.0
    }

    /// Obtain ownership of the inner Ipld value.
    #[allow(dead_code)]
    pub(crate) fn into_inner(self) -> Ipld {
        self.0.to_owned()
    }

    /// Create a new [ReceiptNotification].
    pub(crate) fn with(receipt: Receipt<Ipld>, cid: Cid, metadata: Option<Ipld>) -> Self {
        let receipt: Ipld = receipt.into();
        let data = ipld!({
            "receipt": receipt,
            "metadata": metadata.as_ref().map(|m| m.to_owned()).map_or(Ipld::Null, |m| m),
            "receipt_cid": cid,
        });
        ReceiptNotification(data)
    }
}

impl DagJson for ReceiptNotification where Ipld: From<ReceiptNotification> {}

impl From<ReceiptNotification> for Ipld {
    fn from(receipt: ReceiptNotification) -> Self {
        receipt.0
    }
}

impl From<Ipld> for ReceiptNotification {
    fn from(ipld: Ipld) -> Self {
        ReceiptNotification(ipld)
    }
}
