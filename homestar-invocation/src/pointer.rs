#![allow(missing_docs)]

//! Pointers and references to [Invocations], [Tasks], [Instructions], and/or
//! [Receipts], as well as handling for the [Await]'ed promises of pointers.
//!
//! [Invocations]: super::Invocation
//! [Tasks]: super::Task
//! [Instructions]: crate::task::Instruction
//! [Receipts]: super::Receipt

use crate::{ensure, Error, Unit};
use const_format::formatcp;
#[cfg(feature = "diesel")]
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use enum_assoc::Assoc;
use libipld::{cid::Cid, serde::from_ipld, Ipld, Link};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "diesel")]
use std::str::FromStr;
use std::{borrow::Cow, collections::btree_map::BTreeMap, fmt, module_path};

/// `await/ok` branch for instruction result.
pub const OK_BRANCH: &str = "await/ok";
/// `await/error` branch for instruction result.
pub const ERR_BRANCH: &str = "await/error";
/// `await/*` branch for instruction result.
pub const PTR_BRANCH: &str = "await/*";

/// Enumerated wrapper around resulting branches of a promise
/// that's being awaited on.
///
/// Variants and branch strings are interchangable:
///
/// # Example
///
/// ```
/// use homestar_invocation::pointer::AwaitResult;
///
/// let await_result = AwaitResult::Error;
/// assert_eq!(await_result.branch(), "await/error");
/// assert_eq!(AwaitResult::result("await/*").unwrap(), AwaitResult::Ptr);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Assoc, Deserialize, Serialize)]
#[func(pub const fn branch(&self) -> &'static str)]
#[func(pub fn result(s: &str) -> Option<Self>)]
pub enum AwaitResult {
    /// `Ok` branch.
    #[assoc(branch = OK_BRANCH)]
    #[assoc(result = OK_BRANCH)]
    Ok,
    /// `Error` branch.
    #[assoc(branch = ERR_BRANCH)]
    #[assoc(result = ERR_BRANCH)]
    Error,
    /// Direct resulting branch, without unwrapping of success or failure.
    #[assoc(branch = PTR_BRANCH)]
    #[assoc(result = PTR_BRANCH)]
    Ptr,
}

impl fmt::Display for AwaitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AwaitResult::Error => write!(f, "await/error"),
            AwaitResult::Ok => write!(f, "await/ok"),
            AwaitResult::Ptr => write!(f, "await/*"),
        }
    }
}

impl JsonSchema for AwaitResult {
    fn schema_name() -> String {
        "await_result".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(formatcp!("{}::AwaitResult", module_path!()))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject {
            instance_type: None,
            metadata: Some(Box::new(Metadata {
                title: Some("Await result".to_string()),
                description: Some("Branches of a promise that is awaited".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let await_ok = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([(
                    OK_BRANCH.to_string(),
                    gen.subschema_for::<Pointer>(),
                )]),
                ..Default::default()
            })),
            ..Default::default()
        };
        let await_err = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([(
                    ERR_BRANCH.to_string(),
                    gen.subschema_for::<Pointer>(),
                )]),
                ..Default::default()
            })),
            ..Default::default()
        };
        let await_ptr = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([(
                    PTR_BRANCH.to_string(),
                    gen.subschema_for::<Pointer>(),
                )]),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.subschemas().one_of = Some(vec![
            Schema::Object(await_ok),
            Schema::Object(await_err),
            Schema::Object(await_ptr),
        ]);
        schema.into()
    }
}

/// Describes the eventual output of the referenced [Instruction] as a
/// [Pointer], either resolving to a tagged [OK_BRANCH], [ERR_BRANCH], or direct
/// result of a [PTR_BRANCH].
///
/// [Instruction]: crate::task::Instruction
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Await {
    instruction: Pointer,
    result: AwaitResult,
}

impl Await {
    /// A new `Promise` [Await]'ed on, resulting in a [Pointer]
    /// and [AwaitResult].
    pub fn new(instruction: Pointer, result: AwaitResult) -> Self {
        Self {
            instruction,
            result,
        }
    }

    /// Return Cid to [Instruction] being [Await]'ed on.
    ///
    /// [Instruction]: crate::task::Instruction
    pub fn instruction_cid(&self) -> Cid {
        self.instruction.cid()
    }

    /// Return [AwaitResult] branch.
    pub fn result(&self) -> &AwaitResult {
        &self.result
    }
}

impl From<Await> for Ipld {
    fn from(await_promise: Await) -> Self {
        Ipld::Map(BTreeMap::from([(
            await_promise.result.branch().to_string(),
            await_promise.instruction.into(),
        )]))
    }
}

impl From<&Await> for Ipld {
    fn from(await_promise: &Await) -> Self {
        From::from(await_promise.to_owned())
    }
}

impl TryFrom<Ipld> for Await {
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        ensure!(
            map.len() == 1,
            Error::ConditionNotMet(
                "await promise must have only a single key in a map".to_string()
            )
        );

        let (key, value) = map.into_iter().next().unwrap();
        let instruction = Pointer::try_from(value)?;

        let result = match key.as_str() {
            OK_BRANCH => AwaitResult::Ok,
            ERR_BRANCH => AwaitResult::Error,
            _ => AwaitResult::Ptr,
        };

        Ok(Await {
            instruction,
            result,
        })
    }
}

impl TryFrom<&Ipld> for Await {
    type Error = Error<Unit>;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

/// References a specific [Invocation], [Task], [Instruction], and/or
/// [Receipt], always wrapping a Cid.
///
/// [Invocation]: super::Invocation
/// [Task]: super::Task
/// [Instruction]: crate::task::Instruction
/// [Receipt]: super::Receipt
#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
#[derive(
    Clone,
    Debug,
    AsExpression,
    FromSqlRow,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
    PartialOrd,
    Ord,
)]
#[diesel(sql_type = Text)]
#[repr(transparent)]
pub struct Pointer(Cid);

/// References a specific [Invocation], [Task], [Instruction], or
/// [Receipt], always wrapping a Cid.
///
/// [Invocation]: super::Invocation
/// [Task]: super::Task
/// [Instruction]: super::Instruction
/// [Receipt]: super::Receipt
#[cfg(not(feature = "diesel"))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Pointer(Cid);

impl fmt::Display for Pointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cid_as_string = self.0.to_string();
        write!(f, "{cid_as_string}")
    }
}

impl Pointer {
    /// Return the `inner` Cid for the [Pointer].
    pub fn cid(&self) -> Cid {
        self.0
    }

    /// Wrap an [Pointer] for a given Cid.
    pub fn new(cid: Cid) -> Self {
        Pointer(cid)
    }

    /// Convert an `Ipld::Link` to an [Pointer].
    pub fn new_from_link<T>(link: Link<T>) -> Self {
        Pointer(*link)
    }
}

impl From<Pointer> for Ipld {
    fn from(ptr: Pointer) -> Self {
        Ipld::Link(ptr.cid())
    }
}

impl TryFrom<Ipld> for Pointer {
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: Cid = from_ipld(ipld)?;
        Ok(Pointer(s))
    }
}

impl TryFrom<&Ipld> for Pointer {
    type Error = Error<Unit>;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl<'a> From<Pointer> for Cow<'a, Pointer> {
    fn from(ptr: Pointer) -> Self {
        Cow::Owned(ptr)
    }
}

impl<'a> From<&'a Pointer> for Cow<'a, Pointer> {
    fn from(ptr: &'a Pointer) -> Self {
        Cow::Borrowed(ptr)
    }
}

#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
impl ToSql<Text, Sqlite> for Pointer {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.cid().to_string());
        Ok(IsNull::No)
    }
}

#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
impl<DB> FromSql<Text, DB> for Pointer
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let s = String::from_sql(bytes)?;
        Ok(Pointer::new(Cid::from_str(&s)?))
    }
}

impl JsonSchema for Pointer {
    fn schema_name() -> String {
        "pointer".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(formatcp!("{}::Pointer", module_path!()))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([('/'.to_string(), <String>::json_schema(gen))]),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "CID reference to an invocation, task, instruction, or receipt".to_string(),
                ),
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
    use crate::test_utils::cid::generate_cid;
    use rand::thread_rng;

    #[test]
    fn ser_de_pointer() {
        let pointer = Pointer::new(generate_cid(&mut thread_rng()));
        let ser = serde_json::to_string(&pointer).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(pointer, de);
    }

    #[test]
    fn ser_de_await() {
        let awaited = Await::new(
            Pointer::new(generate_cid(&mut thread_rng())),
            AwaitResult::Ok,
        );
        let ser = serde_json::to_string(&awaited).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(awaited, de);
    }
}
