//! Issuer refers to a principal that issues a receipt.

use crate::{workflow, Unit};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use ucan::ipld::Principle as Principal;

/// [Principal] issuer of a receipt. If omitted issuer is
/// inferred from the [invocation] [task] audience.
///
/// [invocation]: super::Invocation
/// [task]: super::Task
/// [Principal]: Principal
#[derive(Clone, Debug, Deserialize, Serialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = Text)]
#[repr(transparent)]
pub struct Issuer(Principal);

impl Issuer {
    /// Create a new [Issuer], wrapping a [Principal].
    pub fn new(principal: Principal) -> Self {
        Issuer(principal)
    }
}

impl fmt::Display for Issuer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let did_as_string = self.0.to_string();
        write!(f, "{did_as_string}")
    }
}

impl From<Issuer> for Ipld {
    fn from(issuer: Issuer) -> Self {
        let principal = issuer.0.to_string();
        Ipld::String(principal)
    }
}

impl TryFrom<Ipld> for Issuer {
    type Error = workflow::Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s = from_ipld::<String>(ipld)?;
        Ok(Issuer(Principal::from_str(&s)?))
    }
}

impl ToSql<Text, Sqlite> for Issuer {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.0.to_string());
        Ok(IsNull::No)
    }
}

impl<DB> FromSql<Text, DB> for Issuer
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        Ok(Issuer(Principal::from_str(&s)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ser_de() {
        let issuer = Issuer::new(Principal::from_str("did:example:alice").unwrap());
        let ser = serde_json::to_string(&issuer).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(issuer, de);
    }
}
