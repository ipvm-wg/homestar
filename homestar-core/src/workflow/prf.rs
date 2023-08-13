//! The prf field contains links to any UCANs that provide the authority to
//! perform a given [Task].
//!
//! [Task]: super::Task

use crate::{ipld::Link, workflow, Unit};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Binary,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use libipld::{cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use ucan::ipld::UcanIpld;

/// Proof container, containing links to UCANs for a particular [Task] or [Receipt].
///
/// [Task]: super::Task
/// [Receipt]: super::Receipt
#[derive(Clone, Debug, Default, PartialEq, AsExpression, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = Binary)]
#[repr(transparent)]
pub struct UcanPrf(Vec<Link<UcanIpld>>);

impl UcanPrf {
    /// Return owned list of [UcanIpld] links.
    pub fn into_inner(self) -> Vec<Link<UcanIpld>> {
        self.0
    }

    /// Return a reference to a list of [UcanIpld] links.
    pub fn inner(&self) -> &Vec<Link<UcanIpld>> {
        &self.0
    }

    /// Generate a new [UcanPrf] constructor.
    pub fn new(prf: Vec<Link<UcanIpld>>) -> Self {
        UcanPrf(prf)
    }
}

impl From<UcanPrf> for Ipld {
    fn from(prf: UcanPrf) -> Self {
        Ipld::List(prf.0.iter().map(|link| Ipld::Link(*link.cid())).collect())
    }
}

impl TryFrom<Ipld> for UcanPrf {
    type Error = workflow::Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::List(inner) = ipld {
            let prf = inner.iter().try_fold(vec![], |mut acc, ipld| {
                let cid = from_ipld(ipld.to_owned())?;
                acc.push(Link::new(cid));
                Ok::<_, Self::Error>(acc)
            })?;
            Ok(UcanPrf(prf))
        } else {
            Ok(UcanPrf::default())
        }
    }
}

impl TryFrom<&Ipld> for UcanPrf {
    type Error = workflow::Error<Unit>;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl ToSql<Binary, Sqlite> for UcanPrf {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(DagCborCodec.encode(self.inner())?);
        Ok(IsNull::No)
    }
}

impl<DB> FromSql<Binary, DB> for UcanPrf
where
    DB: Backend,
    *const [u8]: FromSql<Binary, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let raw_bytes = <*const [u8] as FromSql<Binary, DB>>::from_sql(bytes)?;
        let raw_bytes: &[u8] = unsafe { &*raw_bytes };
        let decoded = DagCborCodec.decode(raw_bytes)?;
        Ok(UcanPrf::new(decoded))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::cid::generate_cid;
    use rand::thread_rng;

    #[test]
    fn ser_de() {
        let cid1 = generate_cid(&mut thread_rng());
        let cid2 = generate_cid(&mut thread_rng());

        let prf = UcanPrf::new(vec![Link::new(cid1), Link::new(cid2)]);
        let ser = serde_json::to_string(&prf).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(prf, de);
    }
}
