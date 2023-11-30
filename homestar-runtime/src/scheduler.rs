//! [Scheduler] module for executing a series of tasks for a given
//! [Workflow].
//!
//! [Scheduler]: TaskScheduler
//! [Workflow]: homestar_core::Workflow

use crate::{
    db::{Connection, Database},
    workflow::{IndexedResources, Resource, Vertex},
    Db,
};
use anyhow::{anyhow, Context, Result};
use dagga::Node;
use fnv::FnvHashSet;
use futures::future::BoxFuture;
use homestar_core::workflow::{InstructionResult, LinkMap, Pointer};
use homestar_wasm::io::Arg;
use indexmap::IndexMap;
use libipld::Cid;
use std::{ops::ControlFlow, str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use tracing::debug;

/// Type alias for a [Dag] set of batched nodes.
///
/// [Dag]: dagga::Dag
type Schedule<'a> = Vec<Vec<Node<Vertex<'a>, usize>>>;

/// Type for [instruction]-based, batched, execution graph and set of task
/// resources.
///
/// [instruction]: homestar_core::workflow::Instruction
#[derive(Debug, Clone, Default)]
pub(crate) struct ExecutionGraph<'a> {
    /// A built-up [Dag] [Schedule] of batches.
    ///
    /// [Dag]: dagga::Dag
    pub(crate) schedule: Schedule<'a>,
    /// Vector of [resources] to fetch for executing functions in [Workflow].
    ///
    /// [resources]: Resource
    /// [Workflow]: homestar_core::Workflow
    pub(crate) indexed_resources: IndexedResources,
}

/// Scheduler for a series of tasks, including what's run,
/// what's left to run, and data structures to track resources
/// and what's been executed in memory.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(crate) struct TaskScheduler<'a> {
    /// In-memory map of task/instruction results.
    pub(crate) linkmap: Arc<RwLock<LinkMap<InstructionResult<Arg>>>>,
    /// [ExecutionGraph] of what's been run so far for a [Workflow] of `batched`
    /// [Tasks].
    ///
    /// [Workflow]: homestar_core::Workflow
    /// [Tasks]: homestar_core::workflow::Task
    pub(crate) ran: Option<Schedule<'a>>,

    /// [ExecutionGraph] of what's left to run for a [Workflow] of `batched`
    /// [Tasks].
    ///
    /// [Workflow]: homestar_core::Workflow
    /// [Tasks]: homestar_core::workflow::Task
    pub(crate) run: Schedule<'a>,

    /// Step/batch to resume from.
    pub(crate) resume_step: Option<usize>,

    /// Resources that tasks within a [Workflow] rely on, retrieved
    /// through over the network, ahead-of-time.
    ///
    /// This is transferred from the [ExecutionGraph] for executing the
    /// schedule by a worker.
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) resources: Arc<RwLock<IndexMap<Resource, Vec<u8>>>>,
}

/// Scheduler context containing the a schedule for executing tasks.
pub(crate) struct SchedulerContext<'a> {
    /// Scheduler for a series of tasks, including what's run.
    pub(crate) scheduler: TaskScheduler<'a>,
}

impl<'a> TaskScheduler<'a> {
    /// Initialize Task Scheduler for a [Workflow]
    ///
    /// The scheduler will attempt to find already-executed tasks (via [Receipts])
    /// either in the database or through a [Swarm]/DHT query on a short(er)
    /// timeout.
    ///
    /// [Receipts]: crate::Receipt
    /// [Swarm]: crate::network::swarm
    /// [Workflow]: homestar_core::Workflow
    #[allow(unknown_lints, clippy::needless_pass_by_ref_mut)]
    pub(crate) async fn init<F>(
        mut graph: Arc<ExecutionGraph<'a>>,
        conn: &mut Connection,
        fetch_fn: F,
    ) -> Result<SchedulerContext<'a>>
    where
        F: FnOnce(FnvHashSet<Resource>) -> BoxFuture<'a, Result<IndexMap<Resource, Vec<u8>>>>,
    {
        let mut_graph = Arc::make_mut(&mut graph);
        let schedule: &mut Schedule<'a> = mut_graph.schedule.as_mut();
        let schedule_length = schedule.len();
        let mut resources_to_fetch = vec![];
        let linkmap = LinkMap::<InstructionResult<Arg>>::new();

        let resume = 'resume: {
            for (idx, vec) in schedule.iter().enumerate().rev() {
                let folded_pointers = vec.iter().try_fold(vec![], |mut ptrs, node| {
                    let cid = Cid::from_str(node.name())?;
                    mut_graph
                        .indexed_resources
                        .get(&cid)
                        .map(|resource| {
                            resource.iter().for_each(|rsc| {
                                resources_to_fetch.push((cid, rsc));
                            });
                            ptrs.push(Pointer::new(cid));
                        })
                        .ok_or_else(|| anyhow!("resource not found for instruction {cid}"))?;
                    Ok::<_, anyhow::Error>(ptrs)
                });

                if let Ok(pointers) = folded_pointers {
                    match Db::find_instruction_pointers(&pointers, conn) {
                        Ok(found) => {
                            let linkmap = found.iter().fold(linkmap.clone(), |mut map, receipt| {
                                if let Some(idx) = resources_to_fetch
                                    .iter()
                                    .position(|(cid, _rsc)| cid == &receipt.instruction().cid())
                                {
                                    resources_to_fetch.swap_remove(idx);
                                }

                                let _ = map
                                    .insert(receipt.instruction().cid(), receipt.output_as_arg());

                                map
                            });

                            if found.len() == vec.len() {
                                break 'resume ControlFlow::Break((idx + 1, linkmap));
                            } else {
                                continue;
                            }
                        }
                        Err(_) => {
                            debug!(
                                subject = "receipt.db.check",
                                category = "scheduler.run",
                                "receipt not available in the database"
                            );
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            }
            ControlFlow::Continue(())
        };

        let resources_to_fetch: FnvHashSet<Resource> = resources_to_fetch
            .into_iter()
            .map(|(_, rsc)| rsc.to_owned())
            .collect();

        let fetched = fetch_fn(resources_to_fetch)
            .await
            .with_context(|| "unable to fetch resources")?;

        match resume {
            ControlFlow::Break((idx, linkmap)) => {
                let pivot = schedule.split_off(idx);
                let step = if idx >= schedule_length || idx == 0 {
                    None
                } else {
                    Some(idx)
                };

                Ok(SchedulerContext {
                    scheduler: Self {
                        linkmap: Arc::new(linkmap.into()),
                        ran: Some(schedule.to_vec()),
                        run: pivot,
                        resume_step: step,
                        resources: Arc::new(fetched.into()),
                    },
                })
            }
            _ => Ok(SchedulerContext {
                scheduler: Self {
                    linkmap: Arc::new(linkmap.into()),
                    ran: None,
                    run: schedule.to_vec(),
                    resume_step: None,
                    resources: Arc::new(fetched.into()),
                },
            }),
        }
    }

    /// Get the number of tasks that have already ran in the [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    #[allow(dead_code)]
    pub(crate) fn ran_length(&self) -> usize {
        self.ran
            .as_ref()
            .map(|ran| ran.iter().flatten().collect::<Vec<_>>().len())
            .unwrap_or_default()
    }

    /// Get the number of tasks left to run in the [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    #[allow(dead_code)]
    pub(crate) fn run_length(&self) -> usize {
        self.run.iter().flatten().collect::<Vec<_>>().len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{db::Database, test_utils::db::MemoryDb, workflow, Receipt};
    use futures::FutureExt;
    use homestar_core::{
        ipld::DagCbor,
        test_utils::workflow as workflow_test_utils,
        workflow::{
            config::Resources, instruction::RunInstruction, prf::UcanPrf, Invocation,
            Receipt as InvocationReceipt, Task,
        },
        Workflow,
    };
    use libipld::Ipld;

    #[homestar_runtime_proc_macro::db_async_test]
    fn initialize_task_scheduler() {
        let settings = TestSettings::load();
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            workflow_test_utils::related_wasm_instructions::<Arg>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1.clone()),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2.clone()),
            config.into(),
            UcanPrf::default(),
        );

        let db = MemoryDb::setup_connection_pool(&settings.node, None).unwrap();
        let mut conn = db.conn().unwrap();
        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let fetch_fn = |_rscs: FnvHashSet<Resource>| {
            {
                async {
                    let mut index_map = IndexMap::new();
                    index_map.insert(Resource::Url(instruction1.resource().to_owned()), vec![]);
                    index_map.insert(Resource::Url(instruction2.resource().to_owned()), vec![]);

                    Ok(index_map)
                }
            }
            .boxed()
        };

        let builder = workflow::Builder::new(workflow);
        let graph = builder.graph().unwrap();

        let scheduler_ctx = TaskScheduler::init(graph.into(), &mut conn, fetch_fn)
            .await
            .unwrap();

        let ctx = scheduler_ctx.scheduler;

        assert!(ctx.linkmap.read().await.is_empty());
        assert!(ctx.ran.is_none());
        assert_eq!(ctx.run.len(), 2);
        assert_eq!(ctx.resume_step, None);
    }

    #[homestar_runtime_proc_macro::db_async_test]
    fn initialize_task_scheduler_with_receipted_instruction() {
        let settings = TestSettings::load();
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            workflow_test_utils::related_wasm_instructions::<Arg>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1.clone()),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2.clone()),
            config.into(),
            UcanPrf::default(),
        );

        let invocation_receipt = InvocationReceipt::new(
            Invocation::new(task1.clone()).try_into().unwrap(),
            InstructionResult::Ok(Ipld::Integer(4)),
            Ipld::Null,
            None,
            UcanPrf::default(),
        );
        let receipt = Receipt::try_with(
            instruction1.clone().try_into().unwrap(),
            &invocation_receipt,
        )
        .unwrap();

        let db = MemoryDb::setup_connection_pool(&settings.node, None).unwrap();
        let mut conn = db.conn().unwrap();
        let stored_receipt = MemoryDb::store_receipt(receipt.clone(), &mut conn).unwrap();

        assert_eq!(receipt, stored_receipt.unwrap());

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let fetch_fn = |_rscs: FnvHashSet<Resource>| {
            {
                async {
                    let mut index_map = IndexMap::new();
                    index_map.insert(Resource::Url(instruction1.resource().to_owned()), vec![]);
                    index_map.insert(Resource::Url(instruction2.resource().to_owned()), vec![]);

                    Ok(index_map)
                }
            }
            .boxed()
        };

        let builder = workflow::Builder::new(workflow);
        let graph = builder.graph().unwrap();

        let scheduler_ctx = TaskScheduler::init(graph.into(), &mut conn, fetch_fn)
            .await
            .unwrap();

        let ctx = scheduler_ctx.scheduler;
        let ran = ctx.ran.as_ref().unwrap();

        assert_eq!(ctx.linkmap.read().await.len(), 1);
        assert!(ctx
            .linkmap
            .read()
            .await
            .contains_key(&instruction1.clone().to_cid().unwrap()));
        assert_eq!(ran.len(), 1);
        assert_eq!(ctx.run.len(), 1);
        assert_eq!(ctx.resume_step, Some(1));
    }

    #[homestar_runtime_proc_macro::db_async_test]
    fn initialize_task_scheduler_with_all_receipted_instruction() {
        let settings = TestSettings::load();
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            workflow_test_utils::related_wasm_instructions::<Arg>();

        let task1 = Task::new(
            RunInstruction::Expanded(instruction1.clone()),
            config.clone().into(),
            UcanPrf::default(),
        );

        let task2 = Task::new(
            RunInstruction::Expanded(instruction2.clone()),
            config.into(),
            UcanPrf::default(),
        );

        let invocation_receipt1 = InvocationReceipt::new(
            Invocation::new(task1.clone()).try_into().unwrap(),
            InstructionResult::Ok(Ipld::Integer(4)),
            Ipld::Null,
            None,
            UcanPrf::default(),
        );
        let receipt1 = Receipt::try_with(
            instruction1.clone().try_into().unwrap(),
            &invocation_receipt1,
        )
        .unwrap();

        let invocation_receipt2 = InvocationReceipt::new(
            Invocation::new(task2.clone()).try_into().unwrap(),
            InstructionResult::Ok(Ipld::Integer(44)),
            Ipld::Null,
            None,
            UcanPrf::default(),
        );
        let receipt2 = Receipt::try_with(
            instruction2.clone().try_into().unwrap(),
            &invocation_receipt2,
        )
        .unwrap();

        let db = MemoryDb::setup_connection_pool(&settings.node, None).unwrap();
        let mut conn = db.conn().unwrap();
        let rows_inserted = MemoryDb::store_receipts(vec![receipt1, receipt2], &mut conn).unwrap();
        assert_eq!(2, rows_inserted);

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let fetch_fn = |_rscs: FnvHashSet<Resource>| {
            async {
                let mut index_map = IndexMap::new();
                index_map.insert(Resource::Url(instruction1.resource().to_owned()), vec![]);
                index_map.insert(Resource::Url(instruction2.resource().to_owned()), vec![]);
                Ok(index_map)
            }
            .boxed()
        };

        let builder = workflow::Builder::new(workflow);
        let graph = builder.graph().unwrap();

        let scheduler_ctx = TaskScheduler::init(graph.into(), &mut conn, fetch_fn)
            .await
            .unwrap();

        let ctx = scheduler_ctx.scheduler;
        let ran = ctx.ran.as_ref().unwrap();

        assert_eq!(ctx.linkmap.read().await.len(), 1);
        assert!(!ctx
            .linkmap
            .read()
            .await
            .contains_key(&instruction1.clone().to_cid().unwrap()));
        assert!(ctx
            .linkmap
            .read()
            .await
            .contains_key(&instruction2.clone().to_cid().unwrap()));
        assert_eq!(ran.len(), 2);
        assert!(ctx.run.is_empty());
        assert_eq!(ctx.resume_step, None);
    }
}
