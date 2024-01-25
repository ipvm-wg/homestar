//! Issuer refers to the issuer of the invocation.

use crate::{Error, Unit};
#[cfg(feature = "diesel")]
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use libipld::{serde::from_ipld, Ipld};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, str::FromStr};
use ucan::ipld::Principle as Principal;

/// [Principal] issuer of the [Invocation]. If omitted issuer is
/// inferred from the [invocation] [task] audience.
///
/// [invocation]: crate::Invocation
/// [task]: crate::Task
/// [Principal]: Principal
#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
#[derive(Clone, Debug, Deserialize, Serialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = Text)]
#[repr(transparent)]
pub struct Issuer(Principal);

/// [Principal] issuer of the invocation. If omitted issuer is
/// inferred from the [invocation] [task] audience.
///
/// [invocation]: crate::Invocation
/// [task]: crate::Task
/// [Principal]: Principal
#[cfg(not(feature = "diesel"))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s = from_ipld::<String>(ipld)?;
        Ok(Issuer(Principal::from_str(&s)?))
    }
}

#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
impl ToSql<Text, Sqlite> for Issuer {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.0.to_string());
        Ok(IsNull::No)
    }
}

#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
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

impl JsonSchema for Issuer {
    fn schema_name() -> String {
        "iss".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-invocation::authority::issuer::Issuer")
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Issuer".to_string()),
                description: Some("Principal that issued the receipt".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };
        schema.into()
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
