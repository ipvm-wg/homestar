//! Output of an invocation, referenced by its invocation pointer.

use crate::{
    authority::{Issuer, UcanPrf},
    ipld::{DagCbor, DagCborRef, DagJson},
    task, Error, Pointer, Unit,
};
use libipld::{cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Ipld};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

pub mod metadata;

const RAN_KEY: &str = "ran";
const OUT_KEY: &str = "out";
const ISSUER_KEY: &str = "iss";
const METADATA_KEY: &str = "meta";
const PROOF_KEY: &str = "prf";

/// A Receipt is a cryptographically signed description of the [Invocation]
/// and its [resulting output] and requested effects.
///
/// TODO: Effects et al.
///
/// [resulting output]: task::Result
/// [Invocation]: super::Invocation
#[derive(Debug, Clone, PartialEq)]
pub struct Receipt<T> {
    ran: Pointer,
    out: task::Result<T>,
    meta: Ipld,
    issuer: Option<Issuer>,
    prf: UcanPrf,
}

impl<T> Receipt<T> {
    /// Create a new [Receipt].
    pub fn new(
        ran: Pointer,
        result: task::Result<T>,
        metadata: Ipld,
        issuer: Option<Issuer>,
        proof: UcanPrf,
    ) -> Self {
        Self {
            ran,
            out: result,
            meta: metadata,
            issuer,
            prf: proof,
        }
    }
}

impl<T> Receipt<T> {
    /// [Pointer] for [Invocation] ran.
    ///
    /// [Invocation]: super::Invocation
    pub fn ran(&self) -> &Pointer {
        &self.ran
    }

    /// [task::Result] output from invocation/execution.
    pub fn out(&self) -> &task::Result<T> {
        &self.out
    }

    /// Ipld metadata.
    pub fn meta(&self) -> &Ipld {
        &self.meta
    }

    /// Optional [Issuer] for [Receipt].
    pub fn issuer(&self) -> &Option<Issuer> {
        &self.issuer
    }

    /// [UcanPrf] delegation chain.
    pub fn prf(&self) -> &UcanPrf {
        &self.prf
    }
}

impl DagJson for Receipt<Ipld> {}

impl TryFrom<Receipt<Ipld>> for Vec<u8> {
    type Error = Error<Unit>;

    fn try_from(receipt: Receipt<Ipld>) -> Result<Self, Self::Error> {
        let receipt_ipld = Ipld::from(&receipt);
        let encoded = DagCborCodec.encode(&receipt_ipld)?;
        Ok(encoded)
    }
}

impl TryFrom<Vec<u8>> for Receipt<Ipld> {
    type Error = Error<Unit>;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let ipld: Ipld = DagCborCodec.decode(&bytes)?;
        ipld.try_into()
    }
}

impl DagCbor for Receipt<Ipld> {}
impl DagCborRef for Receipt<Ipld> {}

impl From<&Receipt<Ipld>> for Ipld {
    fn from(receipt: &Receipt<Ipld>) -> Self {
        Ipld::Map(BTreeMap::from([
            (RAN_KEY.into(), receipt.ran.to_owned().into()),
            (OUT_KEY.into(), receipt.out.to_owned().into()),
            (METADATA_KEY.into(), receipt.meta.to_owned()),
            (
                ISSUER_KEY.into(),
                receipt
                    .issuer
                    .as_ref()
                    .map(|issuer| issuer.to_string().into())
                    .unwrap_or(Ipld::Null),
            ),
            (PROOF_KEY.into(), receipt.prf.to_owned().into()),
        ]))
    }
}

impl From<Receipt<Ipld>> for Ipld {
    fn from(receipt: Receipt<Ipld>) -> Self {
        From::from(&receipt)
    }
}

impl TryFrom<Ipld> for Receipt<Ipld> {
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let ran = map
            .get(RAN_KEY)
            .ok_or_else(|| Error::<Unit>::MissingField(RAN_KEY.to_string()))?
            .try_into()?;

        let out = map
            .get(OUT_KEY)
            .ok_or_else(|| Error::<Unit>::MissingField(OUT_KEY.to_string()))?;

        let meta = map
            .get(METADATA_KEY)
            .ok_or_else(|| Error::<Unit>::MissingField(METADATA_KEY.to_string()))?;

        let issuer = map
            .get(ISSUER_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok())
            .map(Issuer::new);

        let prf = map
            .get(PROOF_KEY)
            .ok_or_else(|| Error::<Unit>::MissingField(PROOF_KEY.to_string()))?;

        Ok(Receipt {
            ran,
            out: task::Result::try_from(out)?,
            meta: meta.to_owned(),
            issuer,
            prf: UcanPrf::try_from(prf)?,
        })
    }
}

impl TryFrom<Receipt<Ipld>> for Pointer {
    type Error = Error<Unit>;

    fn try_from(receipt: Receipt<Ipld>) -> Result<Self, Self::Error> {
        Ok(Pointer::new(receipt.to_cid()?))
    }
}

impl<T> JsonSchema for Receipt<T> {
    fn schema_name() -> String {
        "receipt".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-invocation::receipt::Receipt")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let meta_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Receipt metadata".to_string()),
                description: Some(
                    "Receipt metadata including the operation that produced the receipt"
                        .to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([("op".to_owned(), <String>::json_schema(gen))]),
                required: BTreeSet::from(["op".to_string()]),
                ..Default::default()
            })),
            ..Default::default()
        };

        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Receipt".to_string()),
                description: Some("A computed receipt".to_string()),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([
                    ("ran".to_owned(), gen.subschema_for::<Pointer>()),
                    ("out".to_owned(), gen.subschema_for::<task::Result<()>>()),
                    ("meta".to_owned(), Schema::Object(meta_schema)),
                    ("iss".to_owned(), gen.subschema_for::<Option<Issuer>>()),
                    ("prf".to_owned(), gen.subschema_for::<UcanPrf>()),
                ]),
                required: BTreeSet::from([
                    "ran".to_string(),
                    "out".to_string(),
                    "meta".to_string(),
                    "prf".to_string(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.into()
    }
}
