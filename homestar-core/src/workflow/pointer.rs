#![allow(missing_docs)]

//! Pointers and references to [Invocations], [Tasks], [Instructions], and/or
//! [Receipts], as well as handling for the [Await]'ed promises of pointers.
//!
//! [Invocations]: super::Invocation
//! [Tasks]: super::Task
//! [Instructions]: super::Instruction
//! [Receipts]: super::Receipt

use anyhow::ensure;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
use enum_assoc::Assoc;
use libipld::{
    cid::{multibase::Base, Cid},
    serde::from_ipld,
    Ipld, Link,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::btree_map::BTreeMap, fmt, str::FromStr};

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
/// use homestar_core::workflow::pointer::AwaitResult;
///
/// let await_result = AwaitResult::Error;
/// assert_eq!(await_result.branch(), "await/error");
/// assert_eq!(AwaitResult::result("await/*").unwrap(), AwaitResult::Ptr);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Assoc)]
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

/// Describes the eventual output of the referenced [Instruction] as a
/// [Pointer], either resolving to a tagged [OK_BRANCH], [ERR_BRANCH], or direct
/// result of a [PTR_BRANCH].
///
/// [Instruction]: super::Instruction
#[derive(Clone, Debug, PartialEq, Eq)]
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
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        ensure!(map.len() == 1, "unexpected keys inside awaited promise");

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
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

/// References a specific [Invocation], [Task], [Instruction], and/or
/// [Receipt], always wrapping a [Cid].
///
/// [Invocation]: super::Invocation
/// [Task]: super::Task
/// [Instruction]: super::Instruction
/// [Receipt]: super::Receipt
#[derive(Clone, Debug, AsExpression, FromSqlRow, PartialEq, Eq, Serialize, Deserialize)]
#[diesel(sql_type = Text)]
pub struct Pointer(Cid);

impl fmt::Display for Pointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cid_as_string = self
            .0
            .to_string_of_base(Base::Base32Lower)
            .map_err(|_| fmt::Error)?;

        write!(f, "{cid_as_string}")
    }
}

impl Pointer {
    /// Return the `inner` [Cid] for the [Pointer].
    pub fn cid(&self) -> Cid {
        self.0
    }

    /// Wrap an [Pointer] for a given [Cid].
    pub fn new(cid: Cid) -> Self {
        Pointer(cid)
    }

    /// Convert an [Ipld::Link] to an [Pointer].
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
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: Cid = from_ipld(ipld)?;
        Ok(Pointer(s))
    }
}

impl TryFrom<&Ipld> for Pointer {
    type Error = anyhow::Error;

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

impl ToSql<Text, Sqlite> for Pointer {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.cid().to_string_of_base(Base::Base32Lower)?);
        Ok(IsNull::No)
    }
}

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
