//! Output of an invocation, referenced by its invocation pointer.

use super::{pointer::InvocationPointer, prf::UcanPrf, InvocationResult};
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use libipld::{
    cbor::DagCborCodec,
    cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    },
    prelude::Codec,
    Ipld,
};
use std::{borrow::Cow, collections::BTreeMap, fmt, str::FromStr};
use ucan::ipld::Principle;

const RAN_KEY: &str = "ran";
const OUT_KEY: &str = "out";
const ISSUER_KEY: &str = "iss";
const METADATA_KEY: &str = "meta";
const PROOF_KEY: &str = "prf";

/// [Principal] that issued this receipt. If omitted issuer is
/// inferred from the [invocation] [task] audience.
///
/// [invocation]: super::Invocation
/// [task]: suepr::Task
#[derive(Clone, Debug, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = Text)]
pub struct Issuer(Principle);

impl fmt::Display for Issuer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let did_as_string = self.0.to_string();
        write!(f, "{did_as_string}")
    }
}

impl Issuer {
    /// Create a new [Issuer], wrapping a [Principle].
    pub fn new(principle: Principle) -> Self {
        Issuer(principle)
    }
}

/// A Receipt is an attestation of the [Result] and requested [Effects] by a
/// [Task Invocation].
///
/// A Receipt MUST be signed by the Executor or it's delegate. If signed by the
/// delegate, the proof of delegation from the [Executor] to the delegate
/// MUST be provided in prf.
///
/// TODO: Effects et al.
///
/// [Result]: InvocationResult
/// [Effects]: https://github.com/ucan-wg/invocation#7-effect
/// [Task Invocation]: super::Invocation
/// [Executor]: Issuer
#[derive(Debug, Clone, PartialEq)]
pub struct Receipt<'a, T> {
    ran: Cow<'a, InvocationPointer>,
    out: InvocationResult<T>,
    meta: Ipld,
    iss: Option<Issuer>,
    prf: UcanPrf,
}

impl<'a, T> Receipt<'a, T> {
    ///
    pub fn new(
        ran: InvocationPointer,
        result: InvocationResult<T>,
        metadata: Ipld,
        issuer: Option<Issuer>,
        proof: UcanPrf,
    ) -> Self {
        Self {
            ran: Cow::from(ran),
            out: result,
            meta: metadata,
            iss: issuer,
            prf: proof,
        }
    }
}

impl<T> Receipt<'_, T> {
    /// [InvocationPointer] for [Task] ran.
    ///
    /// [Task]: super::Task
    pub fn ran(&self) -> &InvocationPointer {
        &self.ran
    }

    /// [InvocationResult] output from [Task] invocation/execution.
    ///
    /// [Task]: super::Task
    pub fn out(&self) -> &InvocationResult<T> {
        &self.out
    }

    /// [Ipld] metadata.
    pub fn meta(&self) -> &Ipld {
        &self.meta
    }

    /// Optional [Issuer] for [Receipt].
    pub fn issuer(&self) -> &Option<Issuer> {
        &self.iss
    }

    /// [UcanPrf] delegation chain.
    pub fn prf(&self) -> &UcanPrf {
        &self.prf
    }
}

impl TryFrom<Receipt<'_, Ipld>> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt<'_, Ipld>) -> Result<Self, Self::Error> {
        let receipt_ipld = Ipld::from(&receipt);
        DagCborCodec.encode(&receipt_ipld)
    }
}

impl TryFrom<Receipt<'_, Ipld>> for Cid {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt<'_, Ipld>) -> Result<Self, Self::Error> {
        TryFrom::try_from(&receipt)
    }
}

impl TryFrom<&Receipt<'_, Ipld>> for Cid {
    type Error = anyhow::Error;

    fn try_from(receipt: &Receipt<'_, Ipld>) -> Result<Self, Self::Error> {
        let ipld = Ipld::from(receipt);
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(0x71, hash))
    }
}

impl From<Receipt<'_, Ipld>> for Ipld {
    fn from(receipt: Receipt<'_, Ipld>) -> Self {
        From::from(&receipt)
    }
}

impl From<&Receipt<'_, Ipld>> for Ipld {
    fn from(receipt: &Receipt<'_, Ipld>) -> Self {
        Ipld::Map(BTreeMap::from([
            (RAN_KEY.into(), receipt.ran.as_ref().to_owned().into()),
            (OUT_KEY.into(), receipt.out.to_owned().into()),
            (METADATA_KEY.into(), receipt.meta.to_owned()),
            (
                ISSUER_KEY.into(),
                receipt
                    .iss
                    .as_ref()
                    .map(|iss| iss.to_string().into())
                    .unwrap_or(Ipld::Null),
            ),
            (PROOF_KEY.into(), receipt.prf.to_owned().into()),
        ]))
    }
}

impl ToSql<Text, Sqlite> for Issuer {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.0.to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for Issuer {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(Issuer(Principle::from_str(&s)?))
    }
}
