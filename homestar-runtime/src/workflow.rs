//! A [Workflow] is a declarative configuration of a series of
//! [UCAN Invocation] `Tasks`.
//!
//! [UCAN Invocation]: <https://github.com/ucan-wg/invocation>

use crate::scheduler::ExecutionGraph;
use anyhow::{anyhow, bail};
use dagga::{self, dot::DagLegend, Node};
use homestar_core::workflow::{
    input::{Parse, Parsed},
    instruction::RunInstruction,
    Instruction, Invocation, Pointer, Task,
};
use homestar_wasm::io::Arg;
use indexmap::IndexMap;
use libipld::{
    cbor::DagCborCodec,
    json::DagJsonCodec,
    multihash::{Code, MultihashDigest},
    prelude::Codec,
    serde::from_ipld,
    Cid, Ipld,
};
use std::{collections::BTreeMap, path::Path};
use url::Url;

pub(crate) mod settings;

type Dag<'a> = dagga::Dag<Vertex<'a>, usize>;

const DAG_CBOR: u64 = 0x71;
const CID_KEY: &str = "cid";
const TASKS_KEY: &str = "tasks";
const PROGRESS_KEY: &str = "progress";
const NUM_TASKS_KEY: &str = "num_tasks";

/// A resource can refer to a [URI] or [Cid]
/// being accessed.
///
/// [URI]: <https://en.wikipedia.org/wiki/Uniform_Resource_Identifier>
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Resource {
    /// Resource fetched by [Url].
    Url(Url),
    /// Resource fetched by [Cid].
    Cid(Cid),
}

/// Ahead-of-time (AOT) context object, which includes the given
/// [Workflow] as a executable [Dag] (directed acyclic graph) and
/// the [Task] resources retrieved through IPFS Client or the DHT directly
/// ahead-of-time.
///
/// [Dag]: dagga::Dag
#[derive(Debug, Clone)]
pub struct AOTContext<'a> {
    dag: Dag<'a>,
    resources: Vec<Resource>,
}

impl AOTContext<'static> {
    /// Convert [Dag] to a [dot] file, to be read by graphviz, etc.
    ///
    /// [Dag]: dagga::Dag
    /// [dot]: <https://graphviz.org/doc/info/lang.html>
    pub fn dot(&self, name: &str, path: &Path) -> anyhow::Result<()> {
        DagLegend::new(self.dag.nodes())
            .with_name(name)
            .save_to(
                path.to_str()
                    .ok_or_else(|| anyhow!("path is not correctly formatted"))?,
            )
            .map_err(|e| anyhow!(e))
    }
}

/// Vertex information for [Dag] [Node].
///
/// [Dag]: dagga::Dag
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex<'a> {
    pub(crate) instruction: Instruction<'a, Arg>,
    pub(crate) parsed: Parsed<Arg>,
    pub(crate) invocation: Pointer,
}

impl<'a> Vertex<'a> {
    fn new(
        instruction: Instruction<'a, Arg>,
        parsed: Parsed<Arg>,
        invocation: Pointer,
    ) -> Vertex<'a> {
        Vertex {
            instruction,
            parsed,
            invocation,
        }
    }
}

/// Associated [Workflow] information, separated from [Workflow] struct in order
/// to relate to it as a key-value relationship of (workflow)
/// cid => [WorkflowInfo].
///
/// TODO: map of task cids completed
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowInfo {
    pub(crate) cid: Cid,
    pub(crate) progress: usize,
    pub(crate) num_tasks: usize,
}

impl WorkflowInfo {
    /// Create a new [WorkflowInfo] given a [Cid], progress / step, and number
    /// of tasks.
    pub fn new(cid: Cid, progress: usize, num_tasks: usize) -> Self {
        Self {
            cid,
            progress,
            num_tasks,
        }
    }

    /// Create a default [WorkflowInfo] given a [Cid] and number of tasks.
    pub fn default(cid: Cid, num_tasks: usize) -> Self {
        Self {
            cid,
            progress: 0,
            num_tasks,
        }
    }

    /// Get the [Cid] of a [Workflow] as a [String].
    pub fn cid(&self) -> String {
        self.cid.to_string()
    }

    /// Set the progress / step of the [WorkflowInfo].
    pub fn set_progress(&mut self, progress: usize) {
        self.progress = progress;
    }

    /// Increment the progress / step of the [WorkflowInfo].
    pub fn increment_progress(&mut self) {
        self.progress += 1;
    }
}

impl From<WorkflowInfo> for Ipld {
    fn from(workflow: WorkflowInfo) -> Self {
        Ipld::Map(BTreeMap::from([
            (CID_KEY.into(), Ipld::Link(workflow.cid)),
            (
                PROGRESS_KEY.into(),
                Ipld::Integer(workflow.progress as i128),
            ),
            (
                NUM_TASKS_KEY.into(),
                Ipld::Integer(workflow.num_tasks as i128),
            ),
        ]))
    }
}

impl TryFrom<Ipld> for WorkflowInfo {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("no `cid` set"))?
                .to_owned(),
        )?;
        let progress = from_ipld(
            map.get(PROGRESS_KEY)
                .ok_or_else(|| anyhow!("no `progress` set"))?
                .to_owned(),
        )?;
        let num_tasks = from_ipld(
            map.get(NUM_TASKS_KEY)
                .ok_or_else(|| anyhow!("no `num_tasks` set"))?
                .to_owned(),
        )?;

        Ok(Self {
            cid,
            progress,
            num_tasks,
        })
    }
}

impl TryFrom<WorkflowInfo> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(receipt: WorkflowInfo) -> Result<Self, Self::Error> {
        let receipt_ipld = Ipld::from(receipt);
        DagCborCodec.encode(&receipt_ipld)
    }
}

impl TryFrom<Vec<u8>> for WorkflowInfo {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let ipld: Ipld = DagCborCodec.decode(&bytes)?;
        ipld.try_into()
    }
}

/// Workflow composed of [tasks].
///
/// [tasks]: Task
#[derive(Debug, Clone, PartialEq)]
pub struct Workflow<'a, T> {
    tasks: Vec<Task<'a, T>>,
}

impl<'a> Workflow<'a, Arg> {
    /// Create a new [Workflow] given a set of tasks.
    pub fn new(tasks: Vec<Task<'a, Arg>>) -> Self {
        Self { tasks }
    }

    /// Length of workflow given a series of [tasks].
    ///
    /// [tasks]: Task
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Whether [Workflow] contains [tasks] or not.
    ///
    /// [tasks]: Task
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Convert the [Workflow] into an batch-separated [ExecutionGraph].
    pub fn graph(self) -> anyhow::Result<ExecutionGraph<'a>> {
        let aot = self.aot()?;
        match aot.dag.build_schedule() {
            Ok(schedule) => Ok(ExecutionGraph {
                schedule: schedule.batches,
                resources: aot.resources,
            }),
            Err(e) => bail!("schedule could not be built from given workflow: {e}"),
        }
    }

    /// Return workflow as stringified Json.
    pub fn to_json(&self) -> anyhow::Result<String> {
        let encoded = DagJsonCodec.encode(&Ipld::from(self.to_owned()))?;
        let s = std::str::from_utf8(&encoded)
            .map_err(|e| anyhow!("cannot stringify encoded value: {e}"))?;
        Ok(s.to_string())
    }

    fn aot(self) -> anyhow::Result<AOTContext<'a>> {
        let lookup_table = self.lookup_table()?;

        let (dag, resources) =
            self.tasks.into_iter().enumerate().try_fold(
                (Dag::default(), vec![]),
                |(mut dag, mut resources), (i, task)| {
                    // Clone as we're owning the struct going backward.
                    let ptr: Pointer = Invocation::<Arg>::from(task.clone()).try_into()?;
                    let instr_cid = task.instruction_cid()?.to_string();

                    let RunInstruction::Expanded(instr) =  task.into_instruction() else {
                    bail!("workflow tasks/instructions must be expanded / inlined")
                };

                    // TODO: check if op is runnable on current node
                    // TODO LATER: check if op is registered on the network

                    resources.push(Resource::Url(instr.resource().to_owned()));

                    let parsed = instr.input().parse()?;
                    let reads = parsed.args().deferreds().into_iter().fold(
                        vec![],
                        |mut in_flow_reads, cid| {
                            if let Some(v) = lookup_table.get(&cid) {
                                in_flow_reads.push(*v)
                            } else {
                                resources.push(Resource::Url(instr.resource().to_owned()));
                            }
                            in_flow_reads
                        },
                    );

                    let node = Node::new(Vertex::new(instr.to_owned(), parsed, ptr))
                        .with_name(instr_cid)
                        .with_result(i);

                    dag.add_node(node.with_reads(reads));
                    Ok::<_, anyhow::Error>((dag, resources))
                },
            )?;

        Ok(AOTContext { dag, resources })
    }

    /// Generate an [IndexMap] lookup table of task instruction CIDs to a
    /// unique enumeration.
    fn lookup_table(&self) -> anyhow::Result<IndexMap<Cid, usize>> {
        self.tasks
            .iter()
            .enumerate()
            .try_fold(IndexMap::new(), |mut acc, (i, t)| {
                acc.insert(t.instruction_cid()?, i);
                Ok::<_, anyhow::Error>(acc)
            })
    }
}

impl From<Workflow<'_, Arg>> for Ipld {
    fn from(workflow: Workflow<'_, Arg>) -> Self {
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

impl TryFrom<Ipld> for Workflow<'_, Arg> {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let ipld = map
            .get(TASKS_KEY)
            .ok_or_else(|| anyhow!("no `tasks` set"))?;

        let tasks = if let Ipld::List(tasks) = ipld {
            let tasks = tasks.iter().fold(vec![], |mut acc, ipld| {
                acc.push(ipld.try_into().unwrap());
                acc
            });
            tasks
        } else {
            bail!("unexpected conversion type")
        };

        Ok(Self { tasks })
    }
}

impl TryFrom<Workflow<'_, Arg>> for Cid {
    type Error = anyhow::Error;

    fn try_from(workflow: Workflow<'_, Arg>) -> Result<Self, Self::Error> {
        let ipld: Ipld = workflow.into();
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(DAG_CBOR, hash))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use homestar_core::{
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf},
    };
    use std::path::Path;

    #[test]
    fn dag_to_dot() {
        let config = Resources::default();
        let instruction1 = test_utils::workflow::wasm_instruction::<Arg>();
        let (instruction2, _) = test_utils::workflow::wasm_instruction_with_nonce::<Arg>();
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

        let workflow = Workflow::new(vec![task1, task2]);
        let aot = workflow.aot().unwrap();

        aot.dot("test", Path::new("test.dot")).unwrap();
        assert!(Path::new("test.dot").exists());
    }

    #[test]
    fn build_parallel_schedule() {
        let config = Resources::default();
        let instruction1 = test_utils::workflow::wasm_instruction::<Arg>();
        let (instruction2, _) = test_utils::workflow::wasm_instruction_with_nonce::<Arg>();
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

        let tasks = vec![task1.clone(), task2.clone()];

        let workflow = Workflow::new(tasks);
        let dag = workflow.aot().unwrap().dag;

        let instr1 = task1.instruction_cid().unwrap().to_string();
        let instr2 = task2.instruction_cid().unwrap().to_string();

        dagga::assert_batches(&[format!("{}, {}", instr2, instr1).as_str()], dag);
    }

    #[test]
    fn build_seq_schedule() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            test_utils::workflow::related_wasm_instructions::<Arg>();
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
        let dag = workflow.aot().unwrap().dag;

        let instr1 = task1.instruction_cid().unwrap().to_string();
        let instr2 = task2.instruction_cid().unwrap().to_string();

        // separate
        dagga::assert_batches(&[&instr1, &instr2], dag);
    }

    #[test]
    fn build_mixed_graph() {
        let config = Resources::default();
        let (instruction1, instruction2, instruction3) =
            test_utils::workflow::related_wasm_instructions::<Arg>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task3 = Task::new(
            RunInstruction::Expanded(instruction3),
            config.clone().into(),
            UcanPrf::default(),
        );

        let (instruction4, _) = test_utils::workflow::wasm_instruction_with_nonce::<Arg>();
        let task4 = Task::new(
            RunInstruction::Expanded(instruction4),
            config.into(),
            UcanPrf::default(),
        );

        let tasks = vec![task1.clone(), task2.clone(), task3.clone(), task4.clone()];
        let workflow = Workflow::new(tasks);

        let instr1 = task1.instruction_cid().unwrap().to_string();
        let instr2 = task2.instruction_cid().unwrap().to_string();
        let instr3 = task3.instruction_cid().unwrap().to_string();
        let instr4 = task4.instruction_cid().unwrap().to_string();

        let schedule = workflow.graph().unwrap().schedule;
        let nodes = schedule
            .into_iter()
            .fold(vec![], |mut acc: Vec<String>, vec| {
                if vec.len() == 1 {
                    acc.push(vec.first().unwrap().name().to_string())
                } else {
                    let mut tmp = vec![];
                    for node in vec {
                        tmp.push(node.name().to_string());
                    }
                    acc.push(tmp.join(", "))
                }

                acc
            });

        assert!(
            nodes
                == vec![
                    format!("{}, {}", instr1, instr4),
                    instr2.clone(),
                    instr3.clone()
                ]
                || nodes == vec![format!("{}, {}", instr4, instr1), instr2, instr3]
        );
    }

    #[test]
    fn workflow_to_json() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            test_utils::workflow::related_wasm_instructions::<Arg>();
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
            test_utils::workflow::related_wasm_instructions::<Arg>();
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

        assert_eq!(workflow, ipld.try_into().unwrap())
    }

    #[test]
    fn ipld_roundtrip_workflow_info() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            test_utils::workflow::related_wasm_instructions::<Arg>();
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
        let mut workflow_info =
            WorkflowInfo::default(Cid::try_from(workflow.clone()).unwrap(), workflow.len());
        let ipld = Ipld::from(workflow_info.clone());
        assert_eq!(workflow_info, ipld.try_into().unwrap());
        workflow_info.increment_progress();
        workflow_info.increment_progress();
        assert_eq!(workflow_info.progress, 2);
    }
}
