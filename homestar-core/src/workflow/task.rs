//! A [Task] is the smallest unit of work that can be requested from a UCAN.

use super::{
    pointer::{InvocationPointer, InvokedTaskPointer},
    Ability, Input, Nonce,
};
use anyhow::anyhow;
use libipld::{
    cbor::DagCborCodec,
    cid::{
        multibase::Base,
        multihash::{Code, MultihashDigest},
        Cid,
    },
    prelude::Codec,
    serde::from_ipld,
    Ipld,
};
use std::{borrow::Cow, collections::BTreeMap, fmt};
use url::Url;

const DAG_CBOR: u64 = 0x71;
const ON_KEY: &str = "on";
const CALL_KEY: &str = "call";
const INPUT_KEY: &str = "input";
const NNC_KEY: &str = "nnc";

/// Enumerator for `either` an expanded [Task] structure or
/// an [InvokedTaskPointer] ([Cid] wrapper).
#[derive(Debug, Clone, PartialEq)]
pub enum RunTask<'a, T> {
    /// [Task] as an expanded structure.
    Expanded(Task<'a, T>),
    /// [Task] as a pointer.
    Ptr(InvokedTaskPointer),
}

impl<'a, T> From<Task<'a, T>> for RunTask<'a, T> {
    fn from(task: Task<'a, T>) -> Self {
        RunTask::Expanded(task)
    }
}

impl<'a, T> TryFrom<RunTask<'a, T>> for Task<'a, T>
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: RunTask<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunTask::Expanded(task) => Ok(task),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<T> From<InvokedTaskPointer> for RunTask<'_, T> {
    fn from(ptr: InvokedTaskPointer) -> Self {
        RunTask::Ptr(ptr)
    }
}

impl<'a, T> TryFrom<RunTask<'a, T>> for InvokedTaskPointer
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: RunTask<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunTask::Ptr(ptr) => Ok(ptr),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<'a, 'b, T> TryFrom<&'b RunTask<'a, T>> for &'b InvokedTaskPointer
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: &'b RunTask<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunTask::Ptr(ptr) => Ok(ptr),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<'a, 'b, T> TryFrom<&'b RunTask<'a, T>> for InvokedTaskPointer
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: &'b RunTask<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunTask::Ptr(ptr) => Ok(ptr.to_owned()),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<T> From<RunTask<'_, T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(run: RunTask<'_, T>) -> Self {
        match run {
            RunTask::Expanded(task) => task.into(),
            RunTask::Ptr(taskptr) => taskptr.into(),
        }
    }
}

impl<T> TryFrom<Ipld> for RunTask<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from<'a>(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(_) => Ok(RunTask::Expanded(Task::try_from(ipld)?)),
            Ipld::Link(_) => Ok(RunTask::Ptr(InvokedTaskPointer::try_from(ipld)?)),
            _ => Err(anyhow!("unexpected conversion type")),
        }
    }
}

/// A Task is the smallest unit of work that can be requested from a UCAN.
/// It describes one (resource, ability, input) triple. The [Input] field is
/// free-form, and depend on the specific resource and ability being interacted
/// with. Inputs can be expressed as [Ipld] or as a [deferred promise].
///
///
/// # Example
///
/// ```
/// use homestar_core::{Unit, workflow::{Ability, Input, Task}};
/// use libipld::Ipld;
/// use url::Url;
///
/// let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
/// let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();
///
/// let task = Task::unique(
///     resource,
///     Ability::from("wasm/run"),
///     Input::<Unit>::Ipld(Ipld::List(vec![Ipld::Bool(true)]))
/// );
/// ```
///
/// We can also set-up a [Task] with a Deferred input to await on:
/// ```
/// use homestar_core::{
///     workflow::{Ability, Input, Nonce, Task,
///         pointer::{Await, AwaitResult, InvocationPointer, InvokedTaskPointer},
///     },
///     Unit,
/// };
/// use libipld::{cid::{multihash::{Code, MultihashDigest}, Cid}, Ipld, Link};
/// use url::Url;

/// let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
/// let resource = Url::parse(format!("ipfs://{wasm}").as_str()).expect("IPFS URL");
/// let h = Code::Blake3_256.digest(b"beep boop");
/// let cid = Cid::new_v1(0x55, h);
/// let link: Link<Cid> = Link::new(cid);
/// let invoked_task = InvocationPointer::new_from_link(link);
///
/// let task = Task::new(
///     resource,
///     Ability::from("wasm/run"),
///     Input::<Unit>::Deferred(Await::new(invoked_task, AwaitResult::Ok)),
///     Some(Nonce::generate())
/// );
///
/// // And covert it to a pointer:
/// let ptr = InvokedTaskPointer::try_from(task).unwrap();
/// ```
/// [deferred promise]: super::pointer::Await
#[derive(Clone, Debug, PartialEq)]
pub struct Task<'a, T> {
    on: Url,
    call: Cow<'a, Ability>,
    input: Input<T>,
    nnc: Option<Nonce>,
}

impl<T> Task<'_, T> {
    /// Create a new [Task].
    pub fn new(on: Url, ability: Ability, input: Input<T>, nnc: Option<Nonce>) -> Self {
        Task {
            on,
            call: Cow::from(ability),
            input,
            nnc,
        }
    }

    /// Create a unique [Task], with a default [Nonce] generator.
    pub fn unique(on: Url, ability: Ability, input: Input<T>) -> Self {
        Task {
            on,
            call: Cow::from(ability),
            input,
            nnc: Some(Nonce::generate()),
        }
    }

    /// Return [Task] resource, i.e. [Url].
    pub fn resource(&self) -> &Url {
        &self.on
    }

    /// Return [Ability] associated with `call`.
    pub fn call(&self) -> &Ability {
        &self.call
    }

    /// Return [Task] [Input].
    pub fn input(&self) -> &Input<T> {
        &self.input
    }
}

impl<T> TryFrom<Task<'_, T>> for InvocationPointer
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(task: Task<'_, T>) -> Result<Self, Self::Error> {
        Ok(InvocationPointer::new(Cid::try_from(task)?))
    }
}

impl<T> TryFrom<Task<'_, T>> for Cid
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(task: Task<'_, T>) -> Result<Self, Self::Error> {
        let ipld: Ipld = task.into();
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(DAG_CBOR, hash))
    }
}

impl<T> From<Task<'_, T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(task: Task<'_, T>) -> Self {
        Ipld::Map(BTreeMap::from([
            (ON_KEY.into(), task.on.to_string().into()),
            (CALL_KEY.into(), task.call.to_string().into()),
            (INPUT_KEY.into(), task.input.into()),
            (
                NNC_KEY.into(),
                task.nnc.map(|nnc| nnc.into()).unwrap_or(Ipld::Null),
            ),
        ]))
    }
}

impl<T> TryFrom<&Ipld> for Task<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl<T> TryFrom<Ipld> for Task<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let on = match map.get(ON_KEY) {
            Some(Ipld::Link(cid)) => cid
                .to_string_of_base(Base::Base32Lower)
                .map_err(|e| anyhow!("failed to encode CID into multibase string: {e}"))
                .and_then(|txt| {
                    Url::parse(format!("{}{}", "ipfs://", txt).as_str())
                        .map_err(|e| anyhow!("failed to parse URL: {e}"))
                }),
            Some(Ipld::String(txt)) => {
                Url::parse(txt.as_str()).map_err(|e| anyhow!("failed to parse URL: {e}"))
            }
            _ => Err(anyhow!("no resource/with set.")),
        }?;

        Ok(Task {
            on,
            call: from_ipld(
                map.get(CALL_KEY)
                    .ok_or_else(|| anyhow!("no `call` field set"))?
                    .to_owned(),
            )?,
            input: Input::try_from(
                map.get(INPUT_KEY)
                    .ok_or_else(|| anyhow!("no `input` field set"))?
                    .to_owned(),
            )?,
            nnc: map.get(NNC_KEY).and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Nonce::try_from(ipld).ok(),
            }),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Unit;

    fn task<'a, T>() -> (Task<'a, T>, Vec<u8>) {
        let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
        let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();
        let nonce = Nonce::generate();

        (
            Task::new(
                resource,
                Ability::from("wasm/run"),
                Input::Ipld(Ipld::List(vec![Ipld::Bool(true)])),
                Some(nonce.clone()),
            ),
            nonce.as_nonce96().unwrap().to_vec(),
        )
    }

    #[test]
    fn ipld_roundtrip() {
        let (task, bytes) = task::<Unit>();
        let ipld = Ipld::from(task.clone());

        assert_eq!(
            ipld,
            Ipld::Map(BTreeMap::from([
                (
                    ON_KEY.into(),
                    Ipld::String(
                        "ipfs://bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".into()
                    )
                ),
                (CALL_KEY.into(), Ipld::String("wasm/run".to_string())),
                (INPUT_KEY.into(), Ipld::List(vec![Ipld::Bool(true)])),
                (
                    NNC_KEY.into(),
                    Ipld::List(vec![Ipld::Integer(0), Ipld::Bytes(bytes)])
                )
            ]))
        );
        assert_eq!(task, ipld.try_into().unwrap())
    }
}
