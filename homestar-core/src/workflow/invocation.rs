//! [Invocation] container for running a [Task] or Task(s).
//!
//! [Task]: super::Task
use crate::{
    consts::VERSION,
    workflow::{
        pointer::{InvocationPointer, InvokedTaskPointer},
        prf::UcanPrf,
        task::RunTask,
    },
};
use anyhow::anyhow;
use libipld::{
    cbor::DagCborCodec,
    cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    },
    prelude::Codec,
    serde::from_ipld,
    Ipld,
};
use semver::Version;
use std::collections::BTreeMap;

const VERSION_KEY: &str = "v";
const RUN_KEY: &str = "run";
const CAUSE_KEY: &str = "cause";
const METADATA_KEY: &str = "meta";
const PROOF_KEY: &str = "prf";

/// An Invocation is an instruction to the [Executor] to perform enclosed
/// [Task].
///
/// Invocations are not executable until they have been provided provable
/// authority (in form of UCANs in the [prf] field) and an Authorization.
///
/// [Executor]: https://github.com/ucan-wg/invocation#212-executor
/// [Task]: super::Task
/// [prf]: super::prf
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation<'a, T> {
    v: Version,
    run: RunTask<'a, T>,
    cause: Option<InvocationPointer>,
    meta: Ipld,
    prf: UcanPrf,
}

impl<'a, T> Invocation<'a, T>
where
    Ipld: From<T>,
    T: Clone,
{
    /// Generate a new [Invocation] to run, with metadata, and `prf`.
    pub fn new(run: RunTask<'a, T>, meta: Ipld, prf: UcanPrf) -> anyhow::Result<Self> {
        let invok = Invocation {
            v: Version::parse(VERSION)?,
            run,
            cause: None,
            meta,
            prf,
        };

        Ok(invok)
    }

    /// Generate a new [Invocation] to run, with metadata, given a [cause], and
    /// `prf`.
    ///
    /// [cause]: https://github.com/ucan-wg/invocation#523-cause
    pub fn new_with_cause(
        run: RunTask<'a, T>,
        meta: Ipld,
        prf: UcanPrf,
        cause: Option<InvocationPointer>,
    ) -> anyhow::Result<Self> {
        let invok = Invocation {
            v: Version::parse(VERSION)?,
            run,
            cause,
            meta,
            prf,
        };

        Ok(invok)
    }

    /// Return a reference pointer to given [Task] to run.
    ///
    /// [Task]: super::Task
    pub fn run(&self) -> &RunTask<'_, T> {
        &self.run
    }

    /// Return the [Cid] of the [Task] to run.
    ///
    /// [Task]: super::Task
    pub fn task_cid(&self) -> anyhow::Result<Cid> {
        match &self.run {
            RunTask::Expanded(task) => Ok(InvokedTaskPointer::try_from(task.to_owned())?.cid()),
            RunTask::Ptr(taskptr) => Ok(taskptr.cid()),
        }
    }
}

impl<T> TryFrom<Invocation<'_, T>> for Ipld
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(invocation: Invocation<'_, T>) -> Result<Self, Self::Error> {
        let map = Ipld::Map(BTreeMap::from([
            (VERSION_KEY.into(), invocation.v.to_string().into()),
            (RUN_KEY.into(), invocation.run.try_into()?),
            (
                CAUSE_KEY.into(),
                invocation.cause.map_or(Ok(Ipld::Null), Ipld::try_from)?,
            ),
            (METADATA_KEY.into(), invocation.meta),
            (PROOF_KEY.into(), invocation.prf.into()),
        ]));

        Ok(map)
    }
}

impl<T> TryFrom<Ipld> for Invocation<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        Ok(Invocation {
            v: from_ipld::<String>(
                map.get(VERSION_KEY)
                    .ok_or_else(|| anyhow!("no `version` field set"))?
                    .to_owned(),
            )
            .map(|s| Version::parse(&s))??,
            run: RunTask::try_from(
                map.get(RUN_KEY)
                    .ok_or_else(|| anyhow!("no `run` set"))?
                    .to_owned(),
            )?,
            cause: map
                .get(CAUSE_KEY)
                .and_then(|ipld| match ipld {
                    Ipld::Null => None,
                    ipld => Some(ipld),
                })
                .and_then(|ipld| ipld.try_into().ok()),
            meta: map
                .get(METADATA_KEY)
                .ok_or_else(|| anyhow!("no `metadata` field set"))?
                .to_owned(),
            prf: UcanPrf::try_from(
                map.get(PROOF_KEY)
                    .ok_or_else(|| anyhow!("no proof field set"))?
                    .to_owned(),
            )?,
        })
    }
}

impl<T> TryFrom<&Ipld> for Invocation<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from<'a>(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl<T> TryFrom<Invocation<'_, T>> for InvocationPointer
where
    Ipld: From<T>,
    T: Clone,
{
    type Error = anyhow::Error;

    fn try_from(invocation: Invocation<'_, T>) -> Result<Self, Self::Error> {
        Ok(InvocationPointer::new(Cid::try_from(invocation)?))
    }
}

impl<T> TryFrom<Invocation<'_, T>> for Cid
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(invocation: Invocation<'_, T>) -> Result<Self, Self::Error> {
        let ipld: Ipld = invocation.try_into()?;
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(0x71, hash))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        workflow::{config::Resources, Ability, Input, Task},
        Unit, VERSION,
    };
    use url::Url;

    fn task<'a, T>() -> Task<'a, T> {
        let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
        let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();

        Task::new(
            resource,
            Ability::from("wasm/run"),
            Input::Ipld(Ipld::List(vec![Ipld::Bool(true)])),
            None,
        )
    }

    #[test]
    fn ipld_roundtrip() {
        let task: Task<'_, Unit> = task();

        let config = Resources::default();
        let invocation1 = Invocation::new(
            RunTask::Expanded(task.clone()),
            config.clone().into(),
            UcanPrf::default(),
        )
        .unwrap();

        let ipld1 = Ipld::try_from(invocation1.clone()).unwrap();

        let ipld_task = Ipld::Map(BTreeMap::from([
            (
                "on".into(),
                Ipld::String(
                    "ipfs://bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".into(),
                ),
            ),
            ("call".into(), Ipld::String("wasm/run".to_string())),
            ("input".into(), Ipld::List(vec![Ipld::Bool(true)])),
            ("nnc".into(), Ipld::Null),
        ]));

        assert_eq!(
            ipld1,
            Ipld::Map(BTreeMap::from([
                (VERSION_KEY.into(), Ipld::String(VERSION.into())),
                (RUN_KEY.into(), ipld_task),
                (CAUSE_KEY.into(), Ipld::Null),
                (
                    METADATA_KEY.into(),
                    Ipld::Map(BTreeMap::from([
                        ("fuel".into(), Ipld::Integer(u64::MAX.into())),
                        ("time".into(), Ipld::Integer(100_000))
                    ]))
                ),
                (PROOF_KEY.into(), Ipld::List(vec![]))
            ]))
        );

        assert_eq!(invocation1, ipld1.try_into().unwrap());

        let invocation2 = Invocation::new_with_cause(
            RunTask::Ptr::<Unit>(task.try_into().unwrap()),
            config.into(),
            UcanPrf::default(),
            Some(InvocationPointer::try_from(invocation1.clone()).unwrap()),
        )
        .unwrap();

        let ipld2 = Ipld::try_from(invocation2.clone()).unwrap();
        let invocation1_ptr: InvocationPointer = invocation1.try_into().unwrap();

        assert_eq!(
            ipld2,
            Ipld::Map(BTreeMap::from([
                (VERSION_KEY.into(), Ipld::String(VERSION.into())),
                (RUN_KEY.into(), Ipld::Link(invocation2.task_cid().unwrap())),
                (CAUSE_KEY.into(), Ipld::Link(invocation1_ptr.cid())),
                (
                    METADATA_KEY.into(),
                    Ipld::Map(BTreeMap::from([
                        ("fuel".into(), Ipld::Integer(u64::MAX.into())),
                        ("time".into(), Ipld::Integer(100_000))
                    ]))
                ),
                (PROOF_KEY.into(), Ipld::List(vec![]))
            ]))
        );

        assert_eq!(invocation2, ipld2.try_into().unwrap());
    }
}
