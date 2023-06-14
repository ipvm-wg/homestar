//! Issuer referring to a principal (principal of least authority) that issues
//! a receipt.

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
use ucan::ipld::Principle;

/// [Principal] issuer of a receipt. If omitted issuer is
/// inferred from the [invocation] [task] audience.
///
/// [invocation]: super::Invocation
/// [task]: super::Task
/// [Principal]: Principle
#[derive(Clone, Debug, Deserialize, Serialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = Text)]
pub struct Issuer(Principle);

impl Issuer {
    /// Create a new [Issuer], wrapping a [Principle].
    pub fn new(principle: Principle) -> Self {
        Issuer(principle)
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
        let principle = issuer.0.to_string();
        Ipld::String(principle)
    }
}

impl TryFrom<Ipld> for Issuer {
    type Error = workflow::Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s = from_ipld::<String>(ipld)?;
        Ok(Issuer(Principle::from_str(&s)?))
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
        Ok(Issuer(Principle::from_str(&s)?))
    }
}
