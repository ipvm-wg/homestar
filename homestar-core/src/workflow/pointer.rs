#![allow(missing_docs)]

//! Pointers and references to [Invocations] and [Tasks].
//!
//! [Invocations]: super::Invocation
//! [Tasks]: super::Task

use anyhow::ensure;
use diesel::{
    backend::RawValue,
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

/// `await/ok` branch for a task invocation.
pub const OK_BRANCH: &str = "await/ok";
/// `await/error` branch for a task invocation.
pub const ERR_BRANCH: &str = "await/error";
/// `await/*` branch for a task invocation.
pub const PTR_BRANCH: &str = "await/*";

/// Type alias around [InvocationPointer] for [Task] pointers.
///
/// Essentially, reusing [InvocationPointer] as a [Cid] wrapper.
///
/// [Task]: super::Task
pub type InvokedTaskPointer = InvocationPointer;

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
    ///
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

/// Describes the eventual output of the referenced [Task invocation], either
/// resolving to [OK_BRANCH], [ERR_BRANCH], or [PTR_BRANCH].
///
/// [Task invocation]: InvokedTaskPointer
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Await {
    invoked_task: InvokedTaskPointer,
    result: AwaitResult,
}

impl Await {
    /// A new `Promise` [Await]'ed on, resulting in a [InvokedTaskPointer]
    /// and [AwaitResult].
    pub fn new(invoked: InvokedTaskPointer, result: AwaitResult) -> Self {
        Await {
            invoked_task: invoked,
            result,
        }
    }

    /// Return [Cid] for [InvokedTaskPointer].
    pub fn task_cid(&self) -> Cid {
        self.invoked_task.cid()
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
            await_promise.invoked_task.into(),
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
        let invoked_task = InvokedTaskPointer::try_from(value)?;

        let result = match key.as_str() {
            OK_BRANCH => AwaitResult::Ok,
            ERR_BRANCH => AwaitResult::Error,
            _ => AwaitResult::Ptr,
        };

        Ok(Await {
            invoked_task,
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

/// References a specific [Invocation], always by Cid.
///
/// [Invocation]: super::Invocation
#[derive(Clone, Debug, AsExpression, FromSqlRow, PartialEq, Eq, Serialize, Deserialize)]
#[diesel(sql_type = Text)]
pub struct InvocationPointer(Cid);

impl fmt::Display for InvocationPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cid_as_string = self
            .0
            .to_string_of_base(Base::Base32Lower)
            .map_err(|_| fmt::Error)?;

        write!(f, "{cid_as_string}")
    }
}

impl InvocationPointer {
    /// Return the `inner` [Cid] for the [InvocationPointer].
    pub fn cid(&self) -> Cid {
        self.0
    }

    /// Wrap an [InvocationPointer] for a given [Cid].
    pub fn new(cid: Cid) -> Self {
        InvocationPointer(cid)
    }

    /// Convert an [Ipld::Link] to an [InvocationPointer].
    pub fn new_from_link<T>(link: Link<T>) -> Self {
        InvocationPointer(*link)
    }
}

impl From<InvocationPointer> for Ipld {
    fn from(ptr: InvocationPointer) -> Self {
        Ipld::Link(ptr.cid())
    }
}

impl TryFrom<Ipld> for InvocationPointer {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s: Cid = from_ipld(ipld)?;
        Ok(InvocationPointer(s))
    }
}

impl TryFrom<&Ipld> for InvocationPointer {
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl<'a> From<InvocationPointer> for Cow<'a, InvocationPointer> {
    fn from(ptr: InvocationPointer) -> Self {
        Cow::Owned(ptr)
    }
}

impl<'a> From<&'a InvocationPointer> for Cow<'a, InvocationPointer> {
    fn from(ptr: &'a InvocationPointer) -> Self {
        Cow::Borrowed(ptr)
    }
}

impl ToSql<Text, Sqlite> for InvocationPointer {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.cid().to_string_of_base(Base::Base32Lower)?);
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for InvocationPointer {
    fn from_sql(bytes: RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(InvocationPointer::new(Cid::from_str(&s)?))
    }
}
