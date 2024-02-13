//! Workflow and [Ucan invocation] componets for building Homestar pipelines.
//!
//! [Ucan invocation]: <https://github.com/ucan-wg/invocation>

use homestar_invocation::{
    bail,
    error::Error,
    ipld::{DagCbor, DagJson},
    Task, Unit,
};
use libipld::{serde::from_ipld, Ipld};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const TASKS_KEY: &str = "tasks";

/// Workflow composed of [tasks].
///
/// [tasks]: Task
#[derive(Debug, Clone, JsonSchema, PartialEq, Serialize, Deserialize)]
#[schemars(title = "Workflow", description = "Workflow composed of tasks")]
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
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let ipld = map
            .get(TASKS_KEY)
            .ok_or_else(|| Error::<Unit>::MissingField(TASKS_KEY.to_string()))?;

        let tasks = if let Ipld::List(tasks) = ipld {
            tasks.iter().try_fold(vec![], |mut acc, ipld| {
                acc.push(ipld.to_owned().try_into()?);
                Ok::<_, Self::Error>(acc)
            })?
        } else {
            bail!(Error::not_an_ipld_list());
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
    use super::*;
    use homestar_invocation::{
        authority::UcanPrf,
        task::{instruction::RunInstruction, Resources},
        test_utils, Unit,
    };
    use std::assert_eq;

    #[test]
    fn workflow_to_json_roundtrip() {
        let config = Resources::default();
        let instruction1 = test_utils::instruction::<Unit>();
        let (instruction2, _) = test_utils::wasm_instruction_with_nonce::<Unit>();

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
        let (instruction1, instruction2, _) = test_utils::related_wasm_instructions::<Unit>();
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
        let instruction1 = test_utils::instruction::<Unit>();
        let (instruction2, _) = test_utils::wasm_instruction_with_nonce::<Unit>();

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
