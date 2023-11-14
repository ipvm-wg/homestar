//! A [Workflow] is a declarative configuration of a series of
//! [UCAN Invocation] `Tasks`.
//!
//! [UCAN Invocation]: <https://github.com/ucan-wg/invocation>

use crate::scheduler::ExecutionGraph;
use anyhow::{anyhow, bail};
use core::fmt;
use dagga::{self, dot::DagLegend, Node};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Binary,
    sqlite::Sqlite,
    AsExpression, FromSqlRow,
};
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
use itertools::Itertools;
use libipld::{cbor::DagCborCodec, cid::Cid, prelude::Codec, serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use tracing::debug;
use url::Url;

mod info;
pub mod settings;
pub use info::WORKFLOW_TAG;
pub(crate) use info::{Info, Stored, StoredReceipt};
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) enum Resource {
    /// Resource fetched by [Url].
    Url(Url),
    /// Resource fetched by [Cid].
    Cid(Cid),
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Resource::Cid(cid) => write!(f, "{}", cid),
            Resource::Url(ref url) => write!(f, "{}", url),
        }
    }
}

/// Ahead-of-time (AOT) context object, which includes the given
/// [Workflow] as a executable [Dag] (directed acyclic graph) and
/// the [Task] resources retrieved through IPFS Client or the DHT directly
/// ahead-of-time.
///
/// [Dag]: dagga::Dag
/// [Task]: homestar_core::workflow::Task
#[derive(Debug, Clone)]
pub(crate) struct AOTContext<'a> {
    dag: Dag<'a>,
    //side_effects: Vec<Node<Vertex<'a>, usize>>,
    indexed_resources: IndexedResources,
}

impl AOTContext<'static> {
    /// Convert [Dag] to a [dot] file, to be read by graphviz, etc.
    ///
    /// [Dag]: dagga::Dag
    /// [dot]: <https://graphviz.org/doc/info/lang.html>
    #[allow(dead_code)]
    pub(crate) fn dot(&self, name: &str, path: &Path) -> anyhow::Result<()> {
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
pub(crate) struct Vertex<'a> {
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
    pub(crate) fn graph(self) -> anyhow::Result<ExecutionGraph<'a>> {
        let aot = self.aot()?;
        match aot.dag.build_schedule() {
            Ok(schedule) => Ok(ExecutionGraph {
                schedule: schedule.batches,
                indexed_resources: aot.indexed_resources,
            }),
            Err(e) => bail!("schedule could not be built from given workflow: {e}"),
        }
    }

    fn aot(self) -> anyhow::Result<AOTContext<'a>> {
        let lookup_table = self.lookup_table()?;
        let (mut dag, unawaits, awaited, resources) =
            self.into_inner().tasks().into_iter().enumerate().try_fold(
                (Dag::default(), vec![], vec![], IndexMap::new()),
                |(mut dag, mut unawaits, mut awaited, mut resources), (i, task)| {
                    let instr_cid = task.instruction_cid()?;
                    debug!("instruction cid: {}", instr_cid);

                    // Clone as we're owning the struct going backward.
                    let ptr: Pointer = Invocation::<Arg>::from(task.clone()).try_into()?;

                    let RunInstruction::Expanded(instr) = task.into_instruction() else {
                        bail!("workflow tasks/instructions must be expanded / inlined")
                    };

                    resources
                        .entry(instr_cid)
                        .or_insert_with(|| vec![Resource::Url(instr.resource().to_owned())]);
                    let parsed = instr.input().parse()?;
                    let reads = parsed
                        .args()
                        .deferreds()
                        .fold(vec![], |mut in_flow_reads, cid| {
                            if let Some(v) = lookup_table.get(&cid) {
                                in_flow_reads.push(*v)
                            }
                            // TODO: else, it's a Promise from another task outside
                            // of the workflow.
                            in_flow_reads
                        });

                    parsed.args().links().for_each(|cid| {
                        resources
                            .entry(instr_cid)
                            .and_modify(|prev_rscs| {
                                prev_rscs.push(Resource::Cid(cid.to_owned()));
                            })
                            .or_insert_with(|| vec![Resource::Cid(cid.to_owned())]);
                    });

                    let node = Node::new(Vertex::new(instr.to_owned(), parsed, ptr))
                        .with_name(instr_cid.to_string())
                        .with_result(i);

                    if !reads.is_empty() {
                        dag.add_node(node.with_reads(reads.clone()));
                        awaited.extend(reads);
                    } else {
                        unawaits.push(node);
                    }

                    Ok::<_, anyhow::Error>((dag, unawaits, awaited, resources))
                },
            )?;

        for mut node in unawaits.clone().into_iter() {
            if node.get_results().any(|r| awaited.contains(r)) {
                dag.add_node(node);
            } else {
                // set barrier for non-awaited nodes
                node.set_barrier(1);
                dag.add_node(node);
            }
        }

        Ok(AOTContext {
            dag,
            indexed_resources: IndexedResources(resources),
        })
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

/// A container for [IndexMap]s from [Cid] => resource.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Binary)]
pub struct IndexedResources(IndexMap<Cid, Vec<Resource>>);

impl IndexedResources {
    /// Create a new [IndexedResources] container from an [IndexMap] of
    /// [Resource]s.
    #[allow(dead_code)]
    pub(crate) fn new(map: IndexMap<Cid, Vec<Resource>>) -> IndexedResources {
        IndexedResources(map)
    }

    /// Reutrn a referenced [IndexMap] of [Resource]s.
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &IndexMap<Cid, Vec<Resource>> {
        &self.0
    }

    /// Return an owned [IndexMap] of [Resource]s.
    #[allow(dead_code)]
    pub(crate) fn into_inner(self) -> IndexMap<Cid, Vec<Resource>> {
        self.0
    }

    /// Get length of [IndexedResources].
    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if [IndexedResources] is empty.
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a [Resource] by [Instruction] [Cid].
    ///
    /// [Instruction]: homestar_core::workflow::Instruction
    #[allow(dead_code)]
    pub(crate) fn get(&self, cid: &Cid) -> Option<&Vec<Resource>> {
        self.0.get(cid)
    }

    /// Iterate over all [Resource]s as references.
    #[allow(dead_code)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Resource> {
        self.0.values().flatten().unique()
    }

    /// Iterate over all [Resource]s.
    #[allow(dead_code)]
    pub(crate) fn into_iter(self) -> impl Iterator<Item = Resource> {
        self.0.into_values().flatten().unique()
    }
}

impl From<IndexedResources> for Ipld {
    fn from(resources: IndexedResources) -> Self {
        let btreemap: BTreeMap<String, Ipld> = resources
            .0
            .into_iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    Ipld::List(
                        v.into_iter()
                            .map(|v| match v {
                                Resource::Url(url) => Ipld::String(url.to_string()),
                                Resource::Cid(cid) => Ipld::Link(cid),
                            })
                            .collect(),
                    ),
                )
            })
            .collect();
        Ipld::Map(btreemap)
    }
}

impl TryFrom<Ipld> for IndexedResources {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?
            .into_iter()
            .map(|(k, v)| {
                let cid = Cid::try_from(k)?;
                let list = from_ipld::<Vec<Ipld>>(v)?;
                let rscs = list
                    .into_iter()
                    .map(|v| {
                        Ok(match v {
                            Ipld::String(url) => Resource::Url(Url::parse(&url)?),
                            Ipld::Link(cid) => Resource::Cid(cid),
                            _ => bail!("invalid resource type"),
                        })
                    })
                    .collect::<Result<Vec<Resource>, anyhow::Error>>()?;

                Ok((cid, rscs))
            })
            .collect::<Result<IndexMap<Cid, Vec<Resource>>, anyhow::Error>>()?;

        Ok(IndexedResources(map))
    }
}

impl TryFrom<IndexedResources> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(resources: IndexedResources) -> Result<Self, Self::Error> {
        let ipld = Ipld::from(resources);
        DagCborCodec.encode(&ipld)
    }
}

impl ToSql<Binary, Sqlite> for IndexedResources
where
    [u8]: ToSql<Binary, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let bytes: Vec<u8> = self.to_owned().try_into()?;
        out.set_value(bytes);
        Ok(IsNull::No)
    }
}

impl<DB> FromSql<Binary, DB> for IndexedResources
where
    DB: Backend,
    *const [u8]: FromSql<Binary, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let raw_bytes = <*const [u8] as FromSql<Binary, DB>>::from_sql(bytes)?;
        let raw_bytes: &[u8] = unsafe { &*raw_bytes };
        let ipld: Ipld = DagCborCodec.decode(raw_bytes)?;
        let decoded: IndexedResources = ipld.try_into()?;
        Ok(decoded)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use homestar_core::{
        ipld::DagCbor,
        test_utils,
        workflow::{
            config::Resources,
            instruction::RunInstruction,
            pointer::{Await, AwaitResult},
            prf::UcanPrf,
            Ability, Input, Task,
        },
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

        assert!(dag
            .nodes()
            .any(|node| node.name() == instr1 || node.name() == instr2));
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
            RunInstruction::Expanded(instruction1.clone()),
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
            config.clone().into(),
            UcanPrf::default(),
        );

        let (instruction5, _) = test_utils::workflow::wasm_instruction_with_nonce::<Arg>();
        let task5 = Task::new(
            RunInstruction::Expanded(instruction5),
            config.clone().into(),
            UcanPrf::default(),
        );

        let promise1 = Await::new(
            Pointer::new(instruction1.clone().to_cid().unwrap()),
            AwaitResult::Ok,
        );

        let dep_instr = Instruction::new(
            instruction1.resource().to_owned(),
            Ability::from("wasm/run"),
            Input::<Arg>::Ipld(Ipld::Map(BTreeMap::from([
                ("func".into(), Ipld::String("add_two".to_string())),
                (
                    "args".into(),
                    Ipld::List(vec![Ipld::try_from(promise1.clone()).unwrap()]),
                ),
            ]))),
        );

        let task6 = Task::new(
            RunInstruction::Expanded(dep_instr),
            config.into(),
            UcanPrf::default(),
        );

        let tasks = vec![
            task6.clone(),
            task1.clone(),
            task2.clone(),
            task3.clone(),
            task4.clone(),
            task5.clone(),
        ];
        let workflow = Workflow::new(tasks);

        let instr1 = task1.instruction_cid().unwrap().to_string();
        let instr2 = task2.instruction_cid().unwrap().to_string();
        let instr3 = task3.instruction_cid().unwrap().to_string();
        let instr4 = task4.instruction_cid().unwrap().to_string();
        let instr5 = task5.instruction_cid().unwrap().to_string();
        let instr6 = task6.instruction_cid().unwrap().to_string();

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
                    format!("{instr1}"),
                    format!("{instr6}, {instr2}"),
                    format!("{instr3}"),
                    format!("{instr4}, {instr5}")
                ]
                || nodes
                    == vec![
                        format!("{instr1}"),
                        format!("{instr6}, {instr2}"),
                        format!("{instr3}"),
                        format!("{instr5}, {instr4}")
                    ]
                || nodes
                    == vec![
                        format!("{instr1}"),
                        format!("{instr2}, {instr6}"),
                        format!("{instr3}"),
                        format!("{instr4}, {instr5}")
                    ]
                || nodes
                    == vec![
                        format!("{instr1}"),
                        format!("{instr2}, {instr6}"),
                        format!("{instr3}"),
                        format!("{instr5}, {instr4}")
                    ]
        );
    }
}
