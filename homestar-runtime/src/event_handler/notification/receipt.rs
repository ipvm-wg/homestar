//! Notification receipts.

use homestar_invocation::{
    ipld::{schema, DagJson},
    Receipt,
};
use libipld::{ipld, Cid, Ipld};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

/// A [Receipt] that is sent out for websocket notifications.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiptNotification(Ipld);

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

impl JsonSchema for ReceiptNotification {
    fn schema_name() -> String {
        "receipt_notification".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-runtime::event_handler::notification::ReceiptNotification")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let metadata_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Metadata".to_string()),
                description: Some("Workflow metadata to contextualize the receipt".to_string()),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([
                    ("name".to_owned(), <String>::json_schema(gen)),
                    ("replayed".to_owned(), <bool>::json_schema(gen)),
                    (
                        "workflow".to_owned(),
                        gen.subschema_for::<schema::IpldLinkStub>(),
                    ),
                ]),
                required: BTreeSet::from([
                    "name".to_string(),
                    "receipt".to_string(),
                    "receipt_cid".to_string(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        };

        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Receipt notification".to_string()),
                description: Some(
                    "A receipt notification associated with a running workflow".to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([
                    ("metadata".to_owned(), Schema::Object(metadata_schema)),
                    ("receipt".to_owned(), gen.subschema_for::<Receipt<()>>()),
                    (
                        "receipt_cid".to_owned(),
                        gen.subschema_for::<schema::IpldLinkStub>(),
                    ),
                ]),
                required: BTreeSet::from(["receipt".to_string(), "receipt_cid".to_string()]),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.into()
    }
}
