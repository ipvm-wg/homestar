//! A [Workflow] is a declarative configuration of a series of
//! [UCAN Invocation] `Tasks`.
//!
//! [UCAN Invocation]: <https://github.com/ucan-wg/invocation>

use crate::scheduler::ExecutionGraph;
use anyhow::{anyhow, bail};
use dagga::{self, dot::DagLegend, Node};
use homestar_core::{
    workflow::{
        input::{Parse, Parsed},
        instruction::RunInstruction,
        Instruction, Invocation, Pointer,
    },
    Workflow,
};
use homestar_wasm::io::Arg;
use indexmap::IndexMap;
use libipld::Cid;
use std::path::Path;
use url::Url;

mod info;
pub(crate) mod settings;
pub use info::{Info, WORKFLOW_TAG};
pub(crate) use info::{Stored, StoredReceipt};
#[allow(unused_imports)]
pub use settings::Settings;

type Dag<'a> = dagga::Dag<Vertex<'a>, usize>;

/// A [Workflow] [Builder] wrapper for the runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct Builder<'a>(Workflow<'a, Arg>);

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
/// [Task]: homestar_core::workflow::Task
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

impl<'a> Builder<'a> {
    /// Create a new [Workflow] [Builder] given a [Workflow].
    pub fn new(workflow: Workflow<'a, Arg>) -> Builder<'a> {
        Builder(workflow)
    }

    /// Return an owned [Workflow] from the [Builder].
    pub fn into_inner(self) -> Workflow<'a, Arg> {
        self.0
    }

    /// Return a referenced [Workflow] from the [Builder].
    pub fn inner(&self) -> &Workflow<'a, Arg> {
        &self.0
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

    fn aot(self) -> anyhow::Result<AOTContext<'a>> {
        let lookup_table = self.lookup_table()?;

        let (dag, resources) =
            self.into_inner().tasks().into_iter().enumerate().try_fold(
                (Dag::default(), vec![]),
                |(mut dag, mut resources), (i, task)| {
                    let instr_cid = task.instruction_cid()?.to_string();
                    // Clone as we're owning the struct going backward.
                    let ptr: Pointer = Invocation::<Arg>::from(task.clone()).try_into()?;

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
        self.inner()
            .tasks_ref()
            .iter()
            .enumerate()
            .try_fold(IndexMap::new(), |mut acc, (i, t)| {
                acc.insert(t.instruction_cid()?, i);
                Ok::<_, anyhow::Error>(acc)
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use homestar_core::{
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
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
        let builder = Builder::new(workflow);
        let aot = builder.aot().unwrap();

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
        let builder = Builder::new(workflow);
        let dag = builder.aot().unwrap().dag;

        let instr1 = task1.instruction_cid().unwrap().to_string();
        let instr2 = task2.instruction_cid().unwrap().to_string();

        dagga::assert_batches(&[format!("{instr2}, {instr1}").as_str()], dag);
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
        let builder = Builder::new(workflow);
        let dag = builder.aot().unwrap().dag;

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

        let builder = Builder::new(workflow);
        let schedule = builder.graph().unwrap().schedule;
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
                    format!("{instr1}, {instr4}"),
                    instr2.clone(),
                    instr3.clone()
                ]
                || nodes == vec![format!("{instr4}, {instr1}"), instr2, instr3]
        );
    }
}
