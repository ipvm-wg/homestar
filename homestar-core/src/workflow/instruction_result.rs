//!  The output `Result` of an [Instruction], tagged as a `success` (`Ok`) or
//!  `failure` (`Error`), or returned/inlined directly.
//!
//!  [Instruction]: super::Instruction

use anyhow::anyhow;
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Binary,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use libipld::{cbor::DagCborCodec, prelude::Codec, Ipld};
use serde::{Deserialize, Serialize};

const OK: &str = "ok";
const ERR: &str = "error";
const JUST: &str = "just";

/// Resultant output of an executed [Instruction].
///
/// [Instruction]: super::Instruction
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Binary)]
pub enum InstructionResult<T> {
    /// `Ok` branch.
    Ok(T),
    /// `Error` branch.
    Error(T),
    /// `Just` branch, meaning `just the value`. Used for
    /// not incorporating unwrapped ok/error into arg/param, where a
    /// result may show up directly.
    Just(T),
}

impl<T> InstructionResult<T> {
    /// Owned, inner result of a [Task] invocation.
    ///
    /// [Task]: super::Task
    pub fn into_inner(self) -> T {
        match self {
            InstructionResult::Ok(inner) => inner,
            InstructionResult::Error(inner) => inner,
            InstructionResult::Just(inner) => inner,
        }
    }

    /// Referenced, inner result of a [Task] invocation.
    ///
    /// [Task]: super::Task
    pub fn inner(&self) -> &T {
        match self {
            InstructionResult::Ok(inner) => inner,
            InstructionResult::Error(inner) => inner,
            InstructionResult::Just(inner) => inner,
        }
    }
}

impl<T> From<InstructionResult<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(result: InstructionResult<T>) -> Self {
        match result {
            InstructionResult::Ok(res) => Ipld::List(vec![OK.into(), res.into()]),
            InstructionResult::Error(res) => Ipld::List(vec![ERR.into(), res.into()]),
            InstructionResult::Just(res) => Ipld::List(vec![JUST.into(), res.into()]),
        }
    }
}

impl<T> TryFrom<Ipld> for InstructionResult<T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, anyhow::Error> {
        if let Ipld::List(v) = ipld {
            match &v[..] {
                [Ipld::String(result), res] if result == OK => {
                    Ok(InstructionResult::Ok(res.to_owned().try_into()?))
                }
                [Ipld::String(result), res] if result == ERR => {
                    Ok(InstructionResult::Error(res.to_owned().try_into()?))
                }
                [Ipld::String(result), res] if result == JUST => {
                    Ok(InstructionResult::Just(res.to_owned().try_into()?))
                }
                _ => Err(anyhow!("unexpected conversion type")),
            }
        } else {
            Err(anyhow!("not convertible to Ipld"))
        }
    }
}

impl<T> TryFrom<&Ipld> for InstructionResult<T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, anyhow::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

/// Diesel, [Sqlite] [ToSql] implementation.
impl ToSql<Binary, Sqlite> for InstructionResult<Ipld>
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
impl FromSql<Binary, Sqlite> for InstructionResult<Ipld> {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
        let raw_bytes = <*const [u8] as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        let raw_bytes: &[u8] = unsafe { &*raw_bytes };
        let decoded: Ipld = DagCborCodec.decode(raw_bytes)?;
        Ok(InstructionResult::try_from(decoded)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ipld_roundtrip() {
        let res1 = InstructionResult::Error(Ipld::String("bad stuff".to_string()));
        let res2 = InstructionResult::Ok(Ipld::String("ok stuff".to_string()));
        let res3 = InstructionResult::Just(Ipld::String("just the right stuff".to_string()));
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
}
