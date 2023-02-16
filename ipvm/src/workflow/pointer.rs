//! Pointers to workflow types.

use anyhow::{anyhow, ensure};
use libipld::{cid::Cid, serde::from_ipld, Ipld};
use std::{collections::btree_map::BTreeMap, result::Result};

/// Successful [Promise] result.
///
/// [Promise]: super::pointer::Promise
pub const OK_BRANCH: &str = "ucan/ok";
const ERR_BRANCH: &str = "ucan/err";
const PTR_BRANCH: &str = "ucan/ptr";

/// A pointer to an unresolved [Invocation] and
/// [Task], optionally including the [Status::Success] or
/// [Status::Failure] branch.
///
/// [Invocation]: super::invocation::Invocation
/// [Task]: super::task::Task

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promise {
    /// Reference to an unresolved [Task] inside a specific [Invocation].
    ///
    /// [Invocation]: super::invocation::Invocation
    /// [Task]: super::task::Task
    pub invoked_task: InvokedTaskPointer,

    /// An optional narrowing to a particular [Status] branch.
    pub result: Option<Status>,
}

impl From<Promise> for Ipld {
    fn from(promise: Promise) -> Self {
        let key = match promise.result {
            Some(Status::Success) => OK_BRANCH,
            Some(Status::Failure) => ERR_BRANCH,
            None => PTR_BRANCH,
        };

        Ipld::Map(BTreeMap::from([(
            key.to_string(),
            promise.invoked_task.into(),
        )]))
    }
}

impl TryFrom<Ipld> for Promise {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        ensure!(map.len() == 1, "unexpected keys in Promise");

        let (key, value) = map.iter().next().unwrap();
        let invoked_task = InvokedTaskPointer::try_from(value.clone())?;

        let result = match key.as_str() {
            OK_BRANCH => Ok(Some(Status::Success)),
            ERR_BRANCH => Ok(Some(Status::Failure)),
            PTR_BRANCH => Ok(None),
            other => Err(anyhow!("unexpected Promise branch selector: {other}")),
        }?;

        Ok(Promise {
            invoked_task,
            result,
        })
    }
}

/// The [Promise] result branch
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// `Success` or `Ok` branch.
    Success,
    /// `Failure` or `Err` branch.
    Failure,
}

/// References a specific [Invocation], either directly by [Cid] (absolute), or
/// local to the [Invocation] itself (relative).
///
/// [Invocation]: super::invocation::Invocation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvocationPointer {
    /// Remote pointer.
    Remote(Cid),
    /// Local or relative pointer.
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
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let s = from_ipld::<String>(ipld)?;

        match s.as_str() {
            "/" => Ok(InvocationPointer::Local),
            other => Ok(InvocationPointer::Remote(Cid::try_from(other)?)),
        }
    }
}

/// References an [InvocationPointer] with a specific
/// [TaskLabel].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvokedTaskPointer {
    invocation: InvocationPointer,
    label: TaskLabel,
}

impl InvokedTaskPointer {
    /// Return [InvocationPointer] from task pointer.
    pub fn invocation(&self) -> &InvocationPointer {
        &self.invocation
    }

    /// Return [TaskLabel] from task pointer.
    pub fn label(&self) -> &TaskLabel {
        &self.label
    }
}

impl From<InvokedTaskPointer> for Ipld {
    fn from(ptr: InvokedTaskPointer) -> Self {
        Ipld::List(vec![ptr.invocation.into(), ptr.label.into()])
    }
}

impl TryFrom<Ipld> for InvokedTaskPointer {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let list: Vec<Ipld> = from_ipld(ipld)?;

        match &list[..] {
            [Ipld::String(s), Ipld::String(label)] => {
                if s.as_str() == "/" {
                    Ok(InvokedTaskPointer {
                        invocation: InvocationPointer::Local,
                        label: TaskLabel(label.to_string()),
                    })
                } else {
                    Err(anyhow!("unexpected format for local InvokedTaskPointer"))
                }
            }
            [Ipld::Link(ptr), Ipld::String(label)] => Ok(InvokedTaskPointer {
                invocation: InvocationPointer::Remote(*ptr),
                label: TaskLabel(label.to_string()),
            }),

            _ => Err(anyhow!("unexpected number of segments in IPLD tuple")),
        }
    }
}

/// A Task label.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct TaskLabel(String);

impl TaskLabel {
    /// Create a new [TaskLabel].
    pub fn new(label: String) -> Self {
        TaskLabel(label)
    }
    /// Get the inner label from [TaskLabel] wrapper.
    pub fn label(&self) -> &str {
        &self.0
    }
}

impl From<TaskLabel> for Ipld {
    fn from(label: TaskLabel) -> Ipld {
        Ipld::String(label.0)
    }
}

impl TryFrom<Ipld> for TaskLabel {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let label = from_ipld(ipld)?;
        Ok(TaskLabel(label))
    }
}
