use crate::workflow::{pointer::TaskLabel, task::Task};
use core::ops::ControlFlow;
use libipld::Ipld;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Batch(BTreeMap<TaskLabel, Task>);

impl Into<Ipld> for Batch {
    fn into(self) -> Ipld {
        match self {
            Batch(assoc) => {
                let mut batch = BTreeMap::new();

                assoc.iter().for_each(|(TaskLabel(label), task)| {
                    batch.insert(label.clone(), task.clone().into());
                });

                Ipld::Map(batch)
            }
        }
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

#[derive(Clone, Debug, PartialEq)]
pub struct Invocation {
    pub run: Batch,
    // pub sig: Sig,
    pub meta: Ipld,
    // pub prf: Vec<Link<Ucan>>,
}

impl Into<Ipld> for Invocation {
    fn into(self) -> Ipld {
        Ipld::Map(BTreeMap::from([
            ("run".to_string(), self.run.clone().into()),
            ("meta".to_string(), self.meta),
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

                Ok(Invocation { run, meta })
            }
            _ => Err(()),
        }
    }
}
