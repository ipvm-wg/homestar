//! Output of an invocation, referenced by its invocation pointer.

use crate::db::schema::receipts;
use anyhow::anyhow;
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Binary,
    sqlite::Sqlite,
    AsExpression, FromSqlRow, Insertable, Queryable,
};
use homestar_core::workflow::{
    pointer::InvocationPointer,
    prf::UcanPrf,
    receipt::{Issuer, Receipt as LocalReceipt},
    InvocationResult,
};
use libipld::{cbor::DagCborCodec, cid::Cid, prelude::Codec, serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

const RAN_KEY: &str = "ran";
const OUT_KEY: &str = "out";
const ISSUER_KEY: &str = "iss";
const METADATA_KEY: &str = "meta";
const PROOF_KEY: &str = "prf";
const CID_KEY: &str = "cid";

/// Receipt for [Invocation], including it's own [Cid].
///
/// `@See` [LocalReceipt] for more info on the internal fields.
///
/// [Invocation]: homestar_core::workflow::Invocation
#[derive(Debug, Clone, PartialEq, Queryable, Insertable)]
pub struct Receipt {
    cid: InvocationPointer,
    ran: InvocationPointer,
    out: InvocationResult<Ipld>,
    meta: LocalIpld,
    iss: Option<Issuer>,
    prf: UcanPrf,
}

impl fmt::Display for Receipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Receipt: [cid: {}, ran: {}, output: {:?}, metadata: {:?}, issuer: {:?}]",
            self.cid, self.ran, self.out, self.meta.0, self.iss
        )
    }
}

impl Receipt {
    /// Generate a receipt.
    pub fn new(cid: Cid, local: &LocalReceipt<'_, Ipld>) -> Self {
        Self {
            ran: local.ran().to_owned(),
            out: local.out().to_owned(),
            meta: LocalIpld(local.meta().to_owned()),
            iss: local.issuer().to_owned(),
            prf: local.prf().to_owned(),
            cid: InvocationPointer::new(cid),
        }
    }

    /// Get unique identifier of receipt.
    pub fn cid(&self) -> String {
        self.cid.to_string()
    }

    /// Get [Cid] in [Receipt] as a [String].
    pub fn ran(&self) -> String {
        self.ran.to_string()
    }

    /// Get executed result/value in [Receipt] as [Ipld].
    pub fn output(&self) -> &InvocationResult<Ipld> {
        &self.out
    }

    /// Get executed result/value in [Receipt] as encoded Cbor.
    pub fn output_encoded(&self) -> anyhow::Result<Vec<u8>> {
        let ipld = Ipld::from(self.out.to_owned());
        DagCborCodec.encode(&ipld)
    }
}

impl TryFrom<Receipt> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt) -> Result<Self, Self::Error> {
        let receipt_ipld = Ipld::from(receipt);
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

impl From<Receipt> for LocalReceipt<'_, Ipld> {
    fn from(receipt: Receipt) -> Self {
        LocalReceipt::new(
            receipt.ran,
            receipt.out,
            receipt.meta.0,
            receipt.iss,
            receipt.prf,
        )
    }
}

impl From<Receipt> for Ipld {
    fn from(receipt: Receipt) -> Self {
        Ipld::Map(BTreeMap::from([
            (RAN_KEY.into(), receipt.ran.into()),
            (OUT_KEY.into(), receipt.out.into()),
            (METADATA_KEY.into(), receipt.meta.0),
            (
                ISSUER_KEY.into(),
                receipt
                    .iss
                    .map(|iss| iss.to_string().into())
                    .unwrap_or(Ipld::Null),
            ),
            (PROOF_KEY.into(), receipt.prf.into()),
            (CID_KEY.into(), receipt.cid.into()),
        ]))
    }
}

impl TryFrom<Ipld> for Receipt {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let ran = map
            .get(RAN_KEY)
            .ok_or_else(|| anyhow!("missing {RAN_KEY}"))?
            .try_into()?;

        let out = map
            .get(OUT_KEY)
            .ok_or_else(|| anyhow!("missing {OUT_KEY}"))?;

        let meta = map
            .get(METADATA_KEY)
            .ok_or_else(|| anyhow!("missing {METADATA_KEY}"))?;

        let iss = map
            .get(ISSUER_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok())
            .map(Issuer::new);

        let prf = map
            .get(PROOF_KEY)
            .ok_or_else(|| anyhow!("missing {PROOF_KEY}"))?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        Ok(Receipt {
            ran,
            out: InvocationResult::try_from(out)?,
            meta: LocalIpld(meta.to_owned()),
            iss,
            prf: UcanPrf::try_from(prf)?,
            cid: InvocationPointer::new(cid),
        })
    }
}

impl TryFrom<LocalReceipt<'_, Ipld>> for Receipt {
    type Error = anyhow::Error;

    fn try_from(receipt: LocalReceipt<'_, Ipld>) -> Result<Self, Self::Error> {
        TryFrom::try_from(&receipt)
    }
}

impl TryFrom<&LocalReceipt<'_, Ipld>> for Receipt {
    type Error = anyhow::Error;

    fn try_from(receipt: &LocalReceipt<'_, Ipld>) -> Result<Self, Self::Error> {
        let cid = Cid::try_from(receipt)?;
        Ok(Receipt::new(cid, receipt))
    }
}

/// Wrapper-type for [Ipld] in order integrate to/from for local storage/db.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Binary)]
pub struct LocalIpld(pub Ipld);

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
    use super::*;
    use crate::{db::schema, receipt::receipts, test_utils::db};
    use diesel::prelude::*;
    use libipld::{
        multihash::{Code, MultihashDigest},
        Link,
    };
    const RAW: u64 = 0x55;

    fn receipt<'a>() -> (LocalReceipt<'a, Ipld>, Receipt) {
        let h = Code::Blake3_256.digest(b"beep boop");
        let cid = Cid::new_v1(RAW, h);
        let link: Link<Cid> = Link::new(cid);
        let local = LocalReceipt::new(
            InvocationPointer::new_from_link(link),
            InvocationResult::Ok(Ipld::Bool(true)),
            Ipld::Null,
            None,
            UcanPrf::default(),
        );
        let receipt = Receipt::try_from(&local).unwrap();
        (local, receipt)
    }

    #[test]
    fn local_into_receipt() {
        let (local, receipt) = receipt();
        assert_eq!(local.ran().to_string(), receipt.ran());
        assert_eq!(local.out(), receipt.output());
        assert_eq!(local.meta(), &receipt.meta.0);
        assert_eq!(local.issuer(), &receipt.iss);
        assert_eq!(local.prf(), &receipt.prf);

        let output_bytes = DagCborCodec
            .encode::<Ipld>(&local.out().clone().into())
            .unwrap();
        assert_eq!(output_bytes, receipt.output_encoded().unwrap());
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

        let inserted_receipt = receipts::table.load::<Receipt>(&mut conn).unwrap();

        assert_eq!(vec![receipt], inserted_receipt);
    }

    #[test]
    fn receipt_bytes_roundtrip() {
        let (_, receipt) = receipt();
        let bytes: Vec<u8> = receipt.clone().try_into().unwrap();
        let from_bytes = Receipt::try_from(bytes).unwrap();

        assert_eq!(receipt, from_bytes);
    }
}