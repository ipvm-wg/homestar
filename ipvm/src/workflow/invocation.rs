//! Signed [Invocation] container for running a
//! [Batch] of [Task]s.

use crate::workflow::{pointer::TaskLabel, task::Task};
use anyhow::anyhow;
use derive_more::{Into, IntoIterator};
use libipld::{cid::Cid, serde::from_ipld, Ipld, Link};
use std::collections::BTreeMap;
use ucan::ipld::UcanIpld;

const RUN_KEY: &str = "run";
const METADATA_KEY: &str = "meta";
const PROOF_KEY: &str = "prf";

/// [Ucan] invocation for for expressing the intention to run delegated
/// capabilities from a [Ucan].
///
/// [Ucan]: https://github.com/ucan-wg/spec/
#[allow(missing_debug_implementations)]
#[derive(Clone, PartialEq)]
pub struct Invocation {
    /// [Batch] of tasks/task-labels.
    pub run: Batch,
    // TODO: Finish sig.
    // pub sig: Sig,
    /// Metadata field.
    pub meta: Ipld,
    /// [ucan] links to provide authority to run actions.
    pub prf: Vec<Link<UcanIpld>>,
}

impl From<Invocation> for Ipld {
    fn from(invocation: Invocation) -> Self {
        Ipld::Map(BTreeMap::from([
            (RUN_KEY.into(), invocation.run.clone().into()),
            (METADATA_KEY.into(), invocation.meta),
            (
                PROOF_KEY.into(),
                Ipld::List(
                    invocation
                        .prf
                        .iter()
                        .map(|link| Ipld::Link(*link.cid()))
                        .collect(),
                ),
            ),
        ]))
    }
}

impl TryFrom<Ipld> for Invocation {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let run = Batch::try_from(
            map.get(RUN_KEY)
                .ok_or_else(|| anyhow!("no run/batch set"))?
                .to_owned(),
        )?;

        let meta = map
            .get(METADATA_KEY)
            .ok_or_else(|| anyhow!("no metadata set"))?
            .to_owned();

        let prf = map
            .get(PROOF_KEY)
            .ok_or_else(|| anyhow!("no proof set"))?
            .to_owned()
            .iter()
            .try_fold(vec![], |mut acc, ipld| {
                let cid = from_ipld::<Cid>(ipld.clone())?;
                acc.push(Link::new(cid));
                Ok::<_, anyhow::Error>(acc)
            })?;

        Ok(Invocation { meta, prf, run })
    }
}

/// Batch of [Task]s with [Closure]s to execute.
///
/// [Closure]: super::closure::Closure.
#[derive(Clone, Debug, PartialEq, IntoIterator, Into)]
pub struct Batch(BTreeMap<TaskLabel, Task>);

impl From<Batch> for Ipld {
    fn from(batch: Batch) -> Self {
        let new_batch = batch
            .0
            .iter()
            .fold(BTreeMap::new(), |mut acc, (task_label, task)| {
                acc.insert(task_label.label().into(), task.clone().into());
                acc
            });

        Ipld::Map(new_batch)
    }
}

impl TryFrom<&Ipld> for Batch {
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl TryFrom<Ipld> for Batch {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let flow = map
            .iter()
            .try_fold(BTreeMap::new(), |mut acc, (key, value)| {
                let task = Task::try_from(value)?;
                acc.insert(TaskLabel::new(key.to_string()), task);
                Ok::<_, anyhow::Error>(acc)
            })?;

        Ok(Batch(flow))
    }
}
