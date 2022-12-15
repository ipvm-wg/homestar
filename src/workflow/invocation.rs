use crate::workflow::{pointer::TaskLabel, task::Task};
use anyhow::{anyhow, bail};
use core::ops::ControlFlow;
use derive_more::{Into, IntoIterator};
use libipld::{Ipld, Link};
use std::collections::BTreeMap;
use ucan::ipld::UcanIpld;

#[derive(Clone, PartialEq)]
pub struct Invocation {
    pub run: Batch,
    // pub sig: Sig,
    pub meta: Ipld,
    pub prf: Vec<Link<UcanIpld>>,
}

impl From<Invocation> for Ipld {
    fn from(invocation: Invocation) -> Self {
        Ipld::Map(BTreeMap::from([
            ("run".to_string(), invocation.run.clone().into()),
            ("meta".to_string(), invocation.meta),
            (
                "prf".to_string(),
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
        match ipld {
            Ipld::Map(assoc) => Ok(Invocation {
                meta: assoc.get("meta").map(Clone::clone).unwrap_or(Ipld::Null),
                run: assoc
                    .get("run")
                    .ok_or(anyhow!("run field is empty"))
                    .and_then(Batch::try_from)
                    .unwrap(),
                prf: match assoc.get("prf") {
                    Some(Ipld::List(vec)) => {
                        vec.iter().try_fold(Vec::new(), |mut acc, ipld| match ipld {
                            Ipld::Link(cid) => {
                                acc.push(Link::new(*cid));
                                Ok(acc)
                            }
                            _ => bail!("Not a link"),
                        })
                    }
                    other => bail!("Expected a List, but got something else: {:?}", other),
                }?,
            }),
            other => bail!("Expected an IPLD map, but got {:?}", other),
        }
    }
}

#[derive(Clone, Debug, PartialEq, IntoIterator, Into)]
pub struct Batch(BTreeMap<TaskLabel, Task>);

impl From<Batch> for Ipld {
    fn from(batch: Batch) -> Self {
        let mut assoc = BTreeMap::new();

        batch.0.iter().for_each(|(TaskLabel(label), task)| {
            assoc.insert(label.clone(), task.clone().into());
        });

        Ipld::Map(assoc)
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
        match ipld {
            Ipld::Map(assoc) => {
                let mut batch = BTreeMap::new();

                let flow = assoc
                    .iter()
                    .try_for_each(|(key, value)| match Task::try_from(value) {
                        Ok(task) => {
                            batch.insert(TaskLabel(key.to_string()), task);
                            ControlFlow::Continue(())
                        }
                        _ => ControlFlow::Break("invalid IPLD Task"),
                    });

                match flow {
                    ControlFlow::Continue(_) => Ok(Batch(batch)),
                    ControlFlow::Break(reason) => bail!(reason),
                }
            }
            _ => bail!("Can only convert from a map"),
        }
    }
}
