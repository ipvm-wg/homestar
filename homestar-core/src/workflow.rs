//! Workflow and [Ucan invocation] componets for building Homestar pipelines.
//!
//! [Ucan invocation]: <https://github.com/ucan-wg/invocation>

use self::Error as WorkflowError;
use crate::{
    bail,
    ipld::{DagCbor, DagJson},
    Unit,
};
use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            .ok_or_else(|| WorkflowError::<Unit>::MissingField(TASKS_KEY.to_string()))?;

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

impl<'a, T> DagCbor for Workflow<'a, T>
where
    T: Clone,
    Ipld: From<T>,
{
}

impl<'a, T> DagJson for Workflow<'a, T>
where
    T: From<Ipld> + Clone,
    Ipld: From<T>,
{
}

#[cfg(test)]
mod test {
    use std::assert_eq;

    use super::*;
    use crate::{
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf},
        Unit,
    };

    #[test]
    fn workflow_to_json_roundtrip() {
        let config = Resources::default();
        let instruction1 = test_utils::workflow::instruction::<Unit>();
        let (instruction2, _) = test_utils::workflow::wasm_instruction_with_nonce::<Unit>();

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

        let json_bytes = workflow.to_json().unwrap();
        let json_string = workflow.to_json_string().unwrap();
        let json_val = json::from(json_string.clone());

        assert_eq!(json_string, json_val.to_string());
        assert_eq!(json_bytes, json_string.as_bytes());
        let wf_from_json1: Workflow<'_, Unit> = DagJson::from_json(json_string.as_bytes()).unwrap();
        assert_eq!(workflow, wf_from_json1);
        let wf_from_json2: Workflow<'_, Unit> = DagJson::from_json_string(json_string).unwrap();
        assert_eq!(workflow, wf_from_json2);
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

    #[test]
    fn ser_de() {
        let config = Resources::default();
        let instruction1 = test_utils::workflow::instruction::<Unit>();
        let (instruction2, _) = test_utils::workflow::wasm_instruction_with_nonce::<Unit>();

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

        let ser = serde_json::to_string(&workflow).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(workflow, de);
    }
}
