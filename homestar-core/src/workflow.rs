//! Workflow and [Ucan invocation] componets for building Homestar pipelines.
//!
//! [Ucan invocation]: <https://github.com/ucan-wg/invocation>

use self::Error as WorkflowError;
use crate::{bail, Unit, DAG_CBOR};
use libipld::{
    cbor::DagCborCodec,
    json::DagJsonCodec,
    multihash::{Code, MultihashDigest},
    prelude::Codec,
    serde::from_ipld,
    Cid, Ipld,
};
use std::collections::BTreeMap;

mod ability;
pub mod config;
pub mod error;
pub mod input;
pub mod instruction;
mod instruction_result;
mod invocation;
mod issuer;
mod nonce;
pub mod pointer;
pub mod prf;
pub mod receipt;
pub mod task;

pub use ability::*;
pub use error::Error;
pub use input::Input;
pub use instruction::Instruction;
pub use instruction_result::*;
pub use invocation::*;
pub use issuer::Issuer;
pub use nonce::*;
pub use pointer::Pointer;
pub use receipt::Receipt;
pub use task::Task;

const TASKS_KEY: &str = "tasks";

/// Generic link, cid => T [IndexMap] for storing
/// invoked, raw values in-memory and using them to
/// resolve other steps within a runtime's workflow.
///
/// [IndexMap]: indexmap::IndexMap
pub type LinkMap<T> = indexmap::IndexMap<libipld::Cid, T>;

/// Workflow composed of [tasks].
///
/// [tasks]: Task
#[derive(Debug, Clone, PartialEq)]
pub struct Workflow<'a, T> {
    tasks: Vec<Task<'a, T>>,
}

impl<'a, T> Workflow<'a, T> {
    /// Create a new [Workflow] given a set of tasks.
    pub fn new(tasks: Vec<Task<'a, T>>) -> Self {
        Self { tasks }
    }

    /// Return a [Workflow]'s [tasks] vector.
    ///
    /// [tasks]: Task
    pub fn tasks(self) -> Vec<Task<'a, T>> {
        self.tasks
    }

    /// Return a reference to [Workflow]'s [tasks] vector.
    ///
    /// [tasks]: Task
    pub fn tasks_ref(&self) -> &Vec<Task<'a, T>> {
        &self.tasks
    }

    /// Length of workflow given a series of [tasks].
    ///
    /// [tasks]: Task
    pub fn len(&self) -> u32 {
        self.tasks.len() as u32
    }

    /// Whether [Workflow] contains [tasks] or not.
    ///
    /// [tasks]: Task
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Return workflow as stringified Json.
    pub fn to_json(self) -> Result<String, WorkflowError<Unit>>
    where
        Ipld: From<Workflow<'a, T>>,
    {
        let encoded = DagJsonCodec.encode(&Ipld::from(self))?;
        let s = std::str::from_utf8(&encoded)?;
        Ok(s.to_string())
    }
}

impl<'a, T> From<Workflow<'a, T>> for Ipld
where
    Ipld: From<Task<'a, T>>,
{
    fn from(workflow: Workflow<'a, T>) -> Self {
        Ipld::Map(BTreeMap::from([(
            TASKS_KEY.into(),
            Ipld::List(
                workflow
                    .tasks
                    .into_iter()
                    .map(Ipld::from)
                    .collect::<Vec<Ipld>>(),
            ),
        )]))
    }
}

impl<'a, T> TryFrom<Ipld> for Workflow<'a, T>
where
    T: From<Ipld>,
{
    type Error = WorkflowError<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let ipld = map
            .get(TASKS_KEY)
            .ok_or_else(|| WorkflowError::<Unit>::MissingFieldError(TASKS_KEY.to_string()))?;

        let tasks = if let Ipld::List(tasks) = ipld {
            tasks.iter().try_fold(vec![], |mut acc, ipld| {
                acc.push(ipld.to_owned().try_into()?);
                Ok::<_, Self::Error>(acc)
            })?
        } else {
            bail!(WorkflowError::not_an_ipld_list());
        };

        Ok(Self { tasks })
    }
}

impl<'a, T> TryFrom<Workflow<'a, T>> for Cid
where
    Ipld: From<Workflow<'a, T>>,
{
    type Error = WorkflowError<Unit>;

    fn try_from(workflow: Workflow<'a, T>) -> Result<Self, Self::Error> {
        let ipld: Ipld = workflow.into();
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(DAG_CBOR, hash))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf},
        Unit,
    };

    #[test]
    fn workflow_to_json() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            test_utils::workflow::related_wasm_instructions::<Unit>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.into(),
            UcanPrf::default(),
        );

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);

        let json_string = workflow.to_json().unwrap();

        let json_val = json::from(json_string.clone());
        assert_eq!(json_string, json_val.to_string());
    }

    #[test]
    fn ipld_roundtrip_workflow() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            test_utils::workflow::related_wasm_instructions::<Unit>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.into(),
            UcanPrf::default(),
        );

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let ipld = Ipld::from(workflow.clone());
        let ipld_to_workflow = ipld.try_into().unwrap();
        assert_eq!(workflow, ipld_to_workflow);
    }
}
