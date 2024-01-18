//!  The output `Result` of an [Instruction], tagged as a `success` (`Ok`) or
//!  `failure` (`Error`), or returned/inlined directly.
//!
//!  [Instruction]: crate::task::Instruction

use crate::{Error, Unit};
#[cfg(feature = "diesel")]
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Binary,
    sqlite::Sqlite,
};
use libipld::Ipld;
#[cfg(feature = "diesel")]
use libipld::{cbor::DagCborCodec, prelude::Codec};
use serde::{Deserialize, Serialize};

const OK: &str = "ok";
const ERR: &str = "error";
const JUST: &str = "just";

/// Resultant output of an executed [Instruction].
///
/// [Instruction]: super::Instruction
#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Binary)]
pub enum Result<T> {
    /// `Ok` branch.
    Ok(T),
    /// `Error` branch.
    Error(T),
    /// `Just` branch, meaning `just the value`. Used for
    /// not incorporating unwrapped ok/error into arg/param, where a
    /// result may show up directly.
    Just(T),
}

/// Output of an executed [Instruction].
///
/// [Instruction]: super::Instruction
#[cfg(not(feature = "diesel"))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Result<T> {
    /// `Ok` branch.
    Ok(T),
    /// `Error` branch.
    Error(T),
    /// `Just` branch, meaning `just the value`. Used for
    /// not incorporating unwrapped ok/error into arg/param, where a
    /// result may show up directly.
    Just(T),
}

impl<T> Result<T> {
    /// Owned, inner result of a [Task] invocation.
    ///
    /// [Task]: super::Task
    pub fn into_inner(self) -> T {
        match self {
            Result::Ok(inner) => inner,
            Result::Error(inner) => inner,
            Result::Just(inner) => inner,
        }
    }

    /// Referenced, inner result of a [Task] invocation.
    ///
    /// [Task]: super::Task
    pub fn inner(&self) -> &T {
        match self {
            Result::Ok(inner) => inner,
            Result::Error(inner) => inner,
            Result::Just(inner) => inner,
        }
    }
}

impl<T> From<Result<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(result: Result<T>) -> Self {
        match result {
            Result::Ok(res) => Ipld::List(vec![OK.into(), res.into()]),
            Result::Error(res) => Ipld::List(vec![ERR.into(), res.into()]),
            Result::Just(res) => Ipld::List(vec![JUST.into(), res.into()]),
        }
    }
}

impl<T> TryFrom<Ipld> for Result<T>
where
    T: From<Ipld>,
{
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> std::result::Result<Self, Error<Unit>> {
        if let Ipld::List(v) = ipld {
            match &v[..] {
                [Ipld::String(result), res] if result == OK => {
                    Ok(Result::Ok(res.to_owned().into()))
                }
                [Ipld::String(result), res] if result == ERR => {
                    Ok(Result::Error(res.to_owned().into()))
                }
                [Ipld::String(result), res] if result == JUST => {
                    Ok(Result::Just(res.to_owned().into()))
                }
                other_ipld => Err(Error::unexpected_ipld(other_ipld.to_owned().into())),
            }
        } else {
            Err(Error::not_an_ipld_list())
        }
    }
}

impl<T> TryFrom<&Ipld> for Result<T>
where
    T: From<Ipld>,
{
    type Error = Error<Unit>;

    fn try_from(ipld: &Ipld) -> std::result::Result<Self, Error<Unit>> {
        TryFrom::try_from(ipld.to_owned())
    }
}

/// Diesel, [Sqlite] [ToSql] implementation.
#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
impl ToSql<Binary, Sqlite> for Result<Ipld>
where
    [u8]: ToSql<Binary, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let ipld = Ipld::from(self.to_owned());
        out.set_value(DagCborCodec.encode(&ipld)?);
        Ok(IsNull::No)
    }
}

/// Diesel, [Sqlite] [FromSql] implementation.
#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
impl<DB> FromSql<Binary, DB> for Result<Ipld>
where
    DB: Backend,
    *const [u8]: FromSql<Binary, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let raw_bytes = <*const [u8] as FromSql<Binary, DB>>::from_sql(bytes)?;
        let raw_bytes: &[u8] = unsafe { &*raw_bytes };
        let decoded: Ipld = DagCborCodec.decode(raw_bytes)?;
        Ok(Result::try_from(decoded)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ipld_roundtrip() {
        let res1 = Result::Error(Ipld::String("bad stuff".to_string()));
        let res2 = Result::Ok(Ipld::String("ok stuff".to_string()));
        let res3 = Result::Just(Ipld::String("just the right stuff".to_string()));
        let ipld1 = Ipld::from(res1.clone());
        let ipld2 = Ipld::from(res2.clone());
        let ipld3 = Ipld::from(res3.clone());

        assert_eq!(ipld1, Ipld::List(vec!["error".into(), "bad stuff".into()]));
        assert_eq!(ipld2, Ipld::List(vec!["ok".into(), "ok stuff".into()]));
        assert_eq!(
            ipld3,
            Ipld::List(vec!["just".into(), "just the right stuff".into()])
        );

        assert_eq!(res1, ipld1.try_into().unwrap());
        assert_eq!(res2, ipld2.try_into().unwrap());
        assert_eq!(res3, ipld3.try_into().unwrap());
    }

    #[test]
    fn ser_de() {
        let res1 = Result::Error(Ipld::String("bad stuff".to_string()));
        let res2 = Result::Ok(Ipld::String("ok stuff".to_string()));
        let res3 = Result::Just(Ipld::String("just the right stuff".to_string()));

        let ser = serde_json::to_string(&res1).unwrap();
        let de = serde_json::from_str(&ser).unwrap();
        assert_eq!(res1, de);

        let ser = serde_json::to_string(&res2).unwrap();
        let de = serde_json::from_str(&ser).unwrap();
        assert_eq!(res2, de);

        let ser = serde_json::to_string(&res3).unwrap();
        let de = serde_json::from_str(&ser).unwrap();
        assert_eq!(res3, de);
    }
}
