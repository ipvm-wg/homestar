//! [Invocation] is a signed [Task].
//!
//! [Task]: super::Task

use crate::{
    ipld::DagCbor,
    workflow::{Error as WorkflowError, Pointer, Task},
    Unit,
};
use libipld::{self, serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const TASK_KEY: &str = "task";

/// A signed [Task] wrapper/container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invocation<'a, T> {
    task: Task<'a, T>,
}

impl<'a, T> From<Task<'a, T>> for Invocation<'a, T>
where
    Ipld: From<T>,
{
    fn from(task: Task<'a, T>) -> Self {
        Invocation::new(task)
    }
}

impl<'a, T> Invocation<'a, T>
where
    Ipld: From<T>,
{
    /// Create a new [Invocation] container.
    pub fn new(task: Task<'a, T>) -> Self {
        Self { task }
    }
}

impl<T> From<Invocation<'_, T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(invocation: Invocation<'_, T>) -> Self {
        Ipld::Map(BTreeMap::from([(TASK_KEY.into(), invocation.task.into())]))
    }
}

impl<T> TryFrom<Ipld> for Invocation<'_, T>
where
    T: From<Ipld>,
{
    type Error = WorkflowError<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        Ok(Self {
            task: Task::try_from(
                map.get(TASK_KEY)
                    .ok_or_else(|| WorkflowError::<Unit>::MissingField(TASK_KEY.to_string()))?
                    .to_owned(),
            )?,
        })
    }
}

impl<T> TryFrom<Invocation<'_, T>> for Pointer
where
    Ipld: From<T>,
{
    type Error = WorkflowError<Unit>;

    fn try_from(invocation: Invocation<'_, T>) -> Result<Self, Self::Error> {
        Ok(Pointer::new(invocation.to_cid()?))
    }
}

impl<'a, T> DagCbor for Invocation<'a, T> where Ipld: From<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
        Unit,
    };

    #[test]
    fn ipld_roundtrip() {
        let config = Resources::default();
        let instruction = test_utils::workflow::instruction::<Unit>();
        let task = Task::new(
            RunInstruction::Expanded(instruction.clone()),
            config.into(),
            UcanPrf::default(),
        );

        let invocation = Invocation::new(task);
        let ipld = Ipld::try_from(invocation.clone()).unwrap();
        assert_eq!(invocation, Invocation::try_from(ipld).unwrap());
    }

    #[test]
    fn ser_de() {
        let config = Resources::default();
        let instruction = test_utils::workflow::instruction::<Unit>();
        let task = Task::new(
            RunInstruction::Expanded(instruction.clone()),
            config.into(),
            UcanPrf::default(),
        );
        let invocation = Invocation::new(task);

        let ser = serde_json::to_string(&invocation).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(invocation, de);
    }
}
