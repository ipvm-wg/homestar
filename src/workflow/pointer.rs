//! Pointers to workflow types
use cid::Cid;
use derive_more::Into;
use libipld::Ipld;
use std::{collections::btree_map::BTreeMap, result::Result};

/// A pointer to an unresolved `Invocation` and `Task`,
/// optionally including the `Success` or `Failure` branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promise {
    /// Reference to an unresolved [Task] inside a specific [Invocation]
    pub invoked_task: InvokedTaskPointer,

    /// An optional narrowing to a particular [Status] branch.
    pub branch_selector: Option<Status>,
}

impl From<Promise> for Ipld {
    fn from(promise: Promise) -> Self {
        let key: String = match promise.branch_selector {
            Some(Status::Success) => "ucan/ok".to_string(),
            Some(Status::Failure) => "ucan/err".to_string(),
            None => "ucan/promise".to_string(),
        };

        Ipld::Map(BTreeMap::from([(key, promise.invoked_task.into())]))
    }
}

impl TryFrom<Ipld> for Promise {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(assoc) => {
                if assoc.len() != 1 {
                    return Err(());
                }

                let (key, value) = assoc.iter().next().unwrap();
                let invoked_task = InvokedTaskPointer::try_from(value.clone())?;

                let branch_selector = match key.as_str() {
                    "ucan/ok" => Ok(Some(Status::Success)),
                    "ucan/err" => Ok(Some(Status::Failure)),
                    "ucan/promise" => Ok(None),
                    _ => Err(()),
                }?;

                Ok(Promise {
                    invoked_task,
                    branch_selector,
                });
            }
            _ => Err(()),
        }
    }
}

/// The [Promise] result branch
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Success,
    Failure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvocationPointer {
    Remote(Cid),
    Local,
}

impl From<InvocationPointer> for Ipld {
    fn from(ptr: InvocationPointer) -> Self {
        match ptr {
            InvocationPointer::Local => Ipld::String("/".to_string()),
            InvocationPointer::Remote(cid) => Ipld::Link(cid),
        }
    }
}

impl TryFrom<Ipld> for InvocationPointer {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(s) => match s.as_str() {
                "/" => Ok(InvocationPointer::Local),
                other => match Cid::try_from(other) {
                    Ok(cid) => Ok(InvocationPointer::Remote(cid)),
                    Err(_) => Err(()),
                },
            },
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvokedTaskPointer {
    pub invocation: InvocationPointer,
    pub label: TaskLabel,
}

impl From<InvokedTaskPointer> for Ipld {
    fn from(ptr: InvokedTaskPointer) -> Self {
        Ipld::List(vec![ptr.invocation.into(), ptr.label.into()])
    }
}

impl TryFrom<Ipld> for InvokedTaskPointer {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::List(list) => match &list[..] {
                [Ipld::String(s), Ipld::String(label)] => match s.as_str() {
                    "/" => Ok(InvokedTaskPointer {
                        invocation: InvocationPointer::Local,
                        label: TaskLabel(label.to_string()),
                    }),
                    _ => Err(()),
                },

                [Ipld::Link(ptr), Ipld::String(label)] => Ok(InvokedTaskPointer {
                    invocation: InvocationPointer::Remote(*ptr),
                    label: TaskLabel(label.to_string()),
                }),

                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Into)]
pub struct TaskLabel(pub String);

impl From<TaskLabel> for Ipld {
    fn from(label: TaskLabel) -> Self {
        Ipld::String(label.0.to_string())
    }
}

impl TryFrom<Ipld> for TaskLabel {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(label) => Ok(TaskLabel(label)),
            _ => Err(()),
        }
    }
}
