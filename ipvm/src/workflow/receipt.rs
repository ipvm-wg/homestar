//! Output of an invocation, referenced by its invocation pointer.

use crate::{db::schema::receipts, workflow::closure::Closure};
use anyhow::anyhow;
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{Binary, Text},
    sqlite::Sqlite,
    AsExpression, FromSqlRow, Insertable, Queryable,
};
use libipld::{
    cbor::DagCborCodec,
    cid::{
        multibase::Base,
        multihash::{Code, MultihashDigest},
        Cid,
    },
    prelude::Codec,
    serde::from_ipld,
    Ipld, Link,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, str::FromStr};

const CID_KEY: &str = "cid";
const CLOSURE_CID_KEY: &str = "closure_cid";
const NONCE_KEY: &str = "nonce";
const OUT_KEY: &str = "out";

/// Receipt for closure invocation, including it's own [Cid].
///
/// `@See` [LocalReceipt] for more info on the internal fields.
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, Serialize, Deserialize)]
pub struct Receipt {
    cid: LocalCid,
    closure_cid: LocalCid,
    nonce: String,
    out: LocalIpld,
}

impl fmt::Display for Receipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Receipt: [cid: {}, closure_cid: {}, nonce: {}, output: {:?}]",
            self.cid, self.closure_cid, self.nonce, self.out.0
        )
    }
}

impl Receipt {
    /// Generate a receipt.
    pub fn new(cid: Cid, local: &LocalReceipt) -> Self {
        Self {
            cid: LocalCid(cid),
            closure_cid: LocalCid(local.closure_cid),
            nonce: local.nonce.to_string(),
            out: LocalIpld(local.out.to_owned()),
        }
    }

    /// Get unique identifier of receipt.
    pub fn cid(&self) -> String {
        self.cid.to_string()
    }

    /// Get closure [Cid] in [Receipt] as a [String].
    pub fn closure_cid(&self) -> String {
        self.closure_cid.to_string()
    }

    /// Get closure executed result/value in [Receipt] as [Ipld].
    pub fn output(&self) -> &Ipld {
        &self.out.0
    }

    /// Get closure executed result/value in [Receipt] as encoded Cbor.
    pub fn output_encoded(&self) -> anyhow::Result<Vec<u8>> {
        DagCborCodec.encode(&self.out.0)
    }
}

impl TryFrom<Receipt> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt) -> Result<Self, Self::Error> {
        let receipt_ipld = Ipld::from(LocalReceipt::try_from(receipt)?);
        DagCborCodec.encode(&receipt_ipld)
    }
}

impl TryFrom<Vec<u8>> for Receipt {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let ipld: Ipld = DagCborCodec.decode(&bytes)?;
        Receipt::try_from(ipld)
    }
}

impl From<Receipt> for LocalReceipt {
    fn from(receipt: Receipt) -> Self {
        LocalReceipt {
            closure_cid: receipt.closure_cid.0,
            nonce: receipt.nonce,
            out: receipt.out.0,
        }
    }
}

impl From<Receipt> for Ipld {
    fn from(receipt: Receipt) -> Self {
        Ipld::Map(BTreeMap::from([
            (CID_KEY.into(), Ipld::Link(receipt.cid.0)),
            (CLOSURE_CID_KEY.into(), Ipld::Link(receipt.closure_cid.0)),
            (NONCE_KEY.into(), Ipld::String(receipt.nonce)),
            (OUT_KEY.into(), receipt.out.0),
        ]))
    }
}

impl TryFrom<Ipld> for Receipt {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let cid_ipld = map
            .get(CID_KEY)
            .ok_or_else(|| anyhow!("Missing {CID_KEY}"))?;
        let cid = from_ipld(cid_ipld.to_owned())?;

        let closure_cid_ipld = map
            .get(CLOSURE_CID_KEY)
            .ok_or_else(|| anyhow!("Missing {CLOSURE_CID_KEY}"))?;
        let closure_cid = from_ipld(closure_cid_ipld.to_owned())?;

        let nonce_ipld = map
            .get(NONCE_KEY)
            .ok_or_else(|| anyhow!("Missing {NONCE_KEY}"))?;
        let nonce = from_ipld(nonce_ipld.to_owned())?;
        let out = map
            .get(OUT_KEY)
            .ok_or_else(|| anyhow!("Missing {OUT_KEY}"))?
            .to_owned();

        Ok(Receipt {
            cid: LocalCid(cid),
            closure_cid: LocalCid(closure_cid),
            nonce,
            out: LocalIpld(out),
        })
    }
}

/// Local version of [Receipt] to generate [Cid].
///
/// A nonce is currently a [`xid`], 12 bytes / 20 chars,
/// configuration free, sortable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalReceipt {
    closure_cid: Cid,
    nonce: String,
    out: Ipld,
}

impl LocalReceipt {
    /// Generate a *local* receipt.
    pub fn new(link: Link<Closure>, result: Ipld) -> Self {
        Self {
            closure_cid: *link,
            nonce: xid::new().to_string(),
            out: result,
        }
    }
}

impl TryFrom<LocalReceipt> for Receipt {
    type Error = anyhow::Error;

    fn try_from(receipt: LocalReceipt) -> Result<Self, Self::Error> {
        TryFrom::try_from(&receipt)
    }
}

impl TryFrom<&LocalReceipt> for Receipt {
    type Error = anyhow::Error;

    fn try_from(receipt: &LocalReceipt) -> Result<Self, Self::Error> {
        let cid = Cid::try_from(receipt)?;
        Ok(Receipt::new(cid, receipt))
    }
}

impl TryFrom<LocalReceipt> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(receipt: LocalReceipt) -> Result<Self, Self::Error> {
        let receipt_ipld = Ipld::from(&receipt);
        DagCborCodec.encode(&receipt_ipld)
    }
}

impl TryFrom<LocalReceipt> for Cid {
    type Error = anyhow::Error;

    fn try_from(receipt: LocalReceipt) -> Result<Self, Self::Error> {
        TryFrom::try_from(&receipt)
    }
}

impl TryFrom<&LocalReceipt> for Cid {
    type Error = anyhow::Error;

    fn try_from(receipt: &LocalReceipt) -> Result<Self, Self::Error> {
        let ipld = Ipld::from(receipt);
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(0x71, hash))
    }
}

impl From<LocalReceipt> for Ipld {
    fn from(receipt: LocalReceipt) -> Self {
        From::from(&receipt)
    }
}

impl From<&LocalReceipt> for Ipld {
    fn from(receipt: &LocalReceipt) -> Self {
        Ipld::Map(BTreeMap::from([
            (CLOSURE_CID_KEY.into(), receipt.closure_cid.into()),
            (NONCE_KEY.into(), receipt.nonce.to_string().into()),
            (OUT_KEY.into(), receipt.out.to_owned()),
        ]))
    }
}

/// Wrapper-type for [Cid] in order integrate to/from for local storage/db.
#[derive(Clone, Debug, AsExpression, FromSqlRow, PartialEq, Serialize, Deserialize)]
#[diesel(sql_type = Text)]
pub struct LocalCid(pub Cid);

impl fmt::Display for LocalCid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cid_as_string = self
            .0
            .to_string_of_base(Base::Base32Lower)
            .map_err(|_| fmt::Error)?;

        write!(f, "{cid_as_string}")
    }
}

/// /// Wrapper-type for [Ipld] in order integrate to/from for local storage/db.
#[derive(Clone, Debug, AsExpression, FromSqlRow, PartialEq, Serialize, Deserialize)]
#[diesel(sql_type = Binary)]
pub struct LocalIpld(pub Ipld);

impl ToSql<Text, Sqlite> for LocalCid {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.0.to_string_of_base(Base::Base32Lower)?);
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for LocalCid {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        // Will decode appropo base.
        Ok(LocalCid(Cid::from_str(&s)?))
    }
}

impl ToSql<Binary, Sqlite> for LocalIpld
where
    [u8]: ToSql<Binary, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(DagCborCodec.encode(&self.0)?);
        Ok(IsNull::No)
    }
}

impl FromSql<Binary, Sqlite> for LocalIpld {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
        let raw_bytes = <*const [u8] as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let raw_bytes: &[u8] = unsafe { &*raw_bytes };
        let decoded = DagCborCodec.decode(raw_bytes)?;
        Ok(LocalIpld(decoded))
    }
}

#[cfg(test)]
mod test {
    use crate::{db::schema, test_utils::db, workflow::receipt::receipts};
    use diesel::{QueryDsl, RunQueryDsl};

    use super::*;
    const RAW: u64 = 0x55;

    fn receipt() -> (LocalReceipt, Receipt) {
        let h = Code::Blake3_256.digest(b"beep boop");
        let cid = Cid::new_v1(RAW, h);
        let link = Link::new(cid);
        let local = LocalReceipt::new(link, Ipld::Bool(true));
        let receipt = Receipt::try_from(&local).unwrap();
        (local, receipt)
    }

    #[test]
    fn local_into_receipt() {
        let (local, receipt) = receipt();
        assert_eq!(local.closure_cid.to_string(), receipt.closure_cid());
        assert_eq!(local.nonce, receipt.nonce);
        assert_eq!(&local.out, receipt.output());

        let output_bytes = DagCborCodec.encode(&local.out).unwrap();
        assert_eq!(output_bytes, receipt.output_encoded().unwrap());

        let local_bytes: Vec<u8> = local.try_into().unwrap();
        let receipt_bytes: Vec<u8> = receipt.try_into().unwrap();
        assert_eq!(local_bytes, receipt_bytes);
    }

    #[test]
    fn receipt_sql_roundtrip() {
        let mut conn = db::setup().unwrap();

        let (_, receipt) = receipt();

        let rows_inserted = diesel::insert_into(schema::receipts::table)
            .values(&receipt)
            .execute(&mut conn)
            .unwrap();

        assert_eq!(1, rows_inserted);

        let inserted_receipt = receipts::table
            .select(receipts::cid)
            .load::<String>(&mut conn)
            .unwrap();

        assert_eq!(vec![receipt.cid()], inserted_receipt);
    }
}
