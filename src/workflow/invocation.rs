use crate::workflow::{pointer::TaskLabel, task::Task};
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
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => {
                let run: Batch = match assoc.get("run") {
                    Some(ipld) => Batch::try_from(ipld.clone()),
                    _ => Err(()),
                }?;

                let meta = match assoc.get("meta") {
                    Some(ipld) => ipld.clone(),
                    None => Ipld::Null,
                };

                let prf = match assoc.get("prf") {
                    Some(Ipld::List(vec)) => {
                        vec.iter().try_fold(Vec::new(), |mut acc, ipld| match ipld {
                            Ipld::Link(cid) => {
                                acc.push(Link::new(*cid));
                                Ok(acc)
                            }
                            _ => Err(()),
                        })
                    }
                    _ => Err(()),
                }?;

                Ok(Invocation { meta, prf, run })
            }
            _ => Err(()),
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

impl TryFrom<Ipld> for Batch {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => {
                let mut batch = BTreeMap::new();

                let flow =
                    assoc
                        .iter()
                        .try_for_each(|(key, value)| match Task::try_from(value.clone()) {
                            Ok(task) => {
                                batch.insert(TaskLabel(key.to_string()), task);
                                ControlFlow::Continue(())
                            }
                            _ => ControlFlow::Break(()),
                        });

                match flow {
                    ControlFlow::Continue(_) => Ok(Batch(batch)),
                    _ => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}
