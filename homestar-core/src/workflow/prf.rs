//! The prf field contains links to any UCANs that provide the authority to
//! perform a given [Task].
//!
//! [Task]: super::Task

use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Binary,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use libipld::{cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Ipld, Link};
use ucan::ipld::UcanIpld;

/// Proof container, containing links to UCANs for a particular [Task].
///
/// [Task]: super::Task
#[derive(Clone, Debug, Default, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Binary)]
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
    type Error = anyhow::Error;

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
    type Error = anyhow::Error;

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

impl FromSql<Binary, Sqlite> for UcanPrf {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
        let raw_bytes = <*const [u8] as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let raw_bytes: &[u8] = unsafe { &*raw_bytes };
        let decoded = DagCborCodec.decode(raw_bytes)?;
        Ok(UcanPrf::new(decoded))
    }
}
