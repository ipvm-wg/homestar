//! Output of an invocation, referenced by its invocation pointer.

use crate::{db::schema::receipts, workflow::closure::Closure};
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
    cid::{multibase::Base, Cid},
    prelude::Codec,
    Ipld, Link,
};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

/// Receipt for closure invocation.
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, Serialize, Deserialize)]
pub struct Receipt {
    id: String,
    closure_cid: LocalCid,
    val: LocalIpld,
}

impl Receipt {
    /// Generate a receipt.
    pub fn new(link: Link<Closure>, result: Ipld) -> Self {
        Receipt {
            id: Uuid::new_v4().to_string(),
            closure_cid: LocalCid(*link),
            val: LocalIpld(result),
        }
    }

    /// Get unique identifier of receipt.
    pub fn id(&self) -> String {
        self.id.to_string()
    }

    /// Get closure [Cid] in [Receipt] as a [String].
    pub fn closure_cid(&self) -> String {
        self.closure_cid.to_string()
    }

    /// Get closure executed result/value in [Receipt] as [Ipld].
    pub fn value(&self) -> Ipld {
        self.val.0.to_owned()
    }

    /// Get closure executed result/value in [Receipt] as encoded Cbor.
    pub fn value_encoded(&self) -> anyhow::Result<Vec<u8>> {
        DagCborCodec.encode(&self.val.0)
    }
}

impl fmt::Display for Receipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Receipt: [id: {}, closure_cid: {}, val: {:?}]",
            self.id, self.closure_cid, self.val.0
        )
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

impl<'a> ToSql<Text, Sqlite> for LocalCid
where
    &'a str: ToSql<Text, Sqlite>,
{
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

/// Receipt-wrapper sharing over PubSub.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedReceipt {
    id: String,
    closure_cid: String,
    val: Vec<u8>,
}

impl TryFrom<SharedReceipt> for Receipt {
    type Error = anyhow::Error;

    fn try_from(shared_receipt: SharedReceipt) -> Result<Self, Self::Error> {
        Ok(Self {
            id: shared_receipt.id,
            closure_cid: LocalCid(Cid::try_from(shared_receipt.closure_cid)?),
            val: LocalIpld(DagCborCodec.decode(&shared_receipt.val)?),
        })
    }
}

impl TryFrom<Receipt> for SharedReceipt {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt) -> Result<Self, Self::Error> {
        let id = receipt.id();
        let cid = receipt.closure_cid();
        let value = receipt.value_encoded()?;

        Ok(Self {
            id,
            closure_cid: cid,
            val: value,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::workflow::closure::{Action, Input};
    use url::Url;

    #[test]
    fn receipt_shared_roundtrip() {
        let closure = Closure {
            resource: Url::parse(
                "ipfs://bafkreiemaanh3kxqchhcdx3yckeb3xvmboztptlgtmnu5jp63bvymxtlva",
            )
            .unwrap(),
            action: Action::try_from("wasm/run").unwrap(),
            inputs: Input::IpldData(Ipld::List(vec![Ipld::Integer(5)])),
        };

        let link: Link<Closure> = Closure::try_into(closure).unwrap();
        let receipt = Receipt::new(link, Ipld::List(vec![Ipld::Integer(42)]));

        let shared = SharedReceipt::try_from(receipt.clone()).unwrap();
        let receipt_shared = Receipt::try_from(shared.clone()).unwrap();

        assert_eq!(receipt, receipt_shared);

        let msg_bytes = bincode::serialize(&shared).unwrap();
        let decoded: SharedReceipt = bincode::deserialize(&msg_bytes).unwrap();
        assert_eq!(shared, decoded);
    }
}
