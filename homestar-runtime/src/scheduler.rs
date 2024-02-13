//! [Scheduler] module for executing a series of tasks for a given
//! [Workflow].
//!
//! [Scheduler]: TaskScheduler
//! [Workflow]: homestar_workflow::Workflow

use crate::{
    db::{Connection, Database},
    workflow::{self, IndexedResources, Resource, Vertex},
    Db,
};
use anyhow::{anyhow, Result};
use dagga::Node;
use fnv::FnvHashSet;
use futures::future::BoxFuture;
use homestar_invocation::{task, Pointer};
use homestar_wasm::io::Arg;
use homestar_workflow::LinkMap;
use indexmap::IndexMap;
use libipld::Cid;
use std::{str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use tracing::debug;

/// Type alias for a [Dag] set of batched nodes.
///
/// [Dag]: dagga::Dag
type Schedule<'a> = Vec<Vec<Node<Vertex<'a>, usize>>>;

/// Type for [instruction]-based, batched, execution graph and set of task
/// resources.
///
/// [instruction]: homestar_invocation::task::Instruction
#[derive(Debug, Clone, Default)]
pub(crate) struct ExecutionGraph<'a> {
    /// A built-up [Dag] [Schedule] of batches.
    ///
    /// [Dag]: dagga::Dag
    ///
    pub(crate) awaiting: workflow::Promises,
    pub(crate) schedule: Schedule<'a>,
    /// Vector of [resources] to fetch for executing functions in [Workflow].
    ///
    /// [resources]: Resource
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) indexed_resources: IndexedResources,
}

/// Scheduler for a series of tasks, including what's run,
/// what's left to run, and data structures to track resources
/// and what's been executed in memory.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(crate) struct TaskScheduler<'a> {
    /// In-memory map of task/instruction results.
    pub(crate) linkmap: Arc<RwLock<LinkMap<task::Result<Arg>>>>,
    /// [ExecutionGraph] of what's been run so far for a [Workflow] of `batched`
    /// [Tasks].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    /// [Tasks]: homestar_invocation::Task
    pub(crate) ran: Option<Schedule<'a>>,

    /// [ExecutionGraph] of what's left to run for a [Workflow] of `batched`
    /// [Tasks].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    /// [Tasks]: homestar_invocation::Task
    pub(crate) run: Schedule<'a>,

    /// Set of Cids to possibly fetch from the DHT.
    pub(crate) promises_to_resolve: FnvHashSet<Cid>,

    /// Step/batch to resume from.
    pub(crate) resume_step: Option<usize>,

    /// Resources that tasks within a [Workflow] rely on, retrieved
    /// through over the network, ahead-of-time.
    ///
    /// This is transferred from the [ExecutionGraph] for executing the
    /// schedule by a worker.
    ///
    /// [Workflow]: homestar_workflow::Workflow
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
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) async fn init<F>(
        mut graph: Arc<ExecutionGraph<'a>>,
        conn: &mut Connection,
        fetch_fn: F,
    ) -> Result<SchedulerContext<'a>>
    where
        F: FnOnce(FnvHashSet<Resource>) -> BoxFuture<'a, Result<IndexMap<Resource, Vec<u8>>>>,
    {
        let mut_graph = Arc::make_mut(&mut graph);
        let schedule = &mut mut_graph.schedule;
        let schedule_length = schedule.len();
        let mut cids_to_resolve = Vec::new();
        let mut resources_to_fetch = Vec::new();
        let mut linkmap = LinkMap::<task::Result<Arg>>::default();

        let mut last_idx = 0;
        for (idx, vec) in schedule.iter().enumerate().rev() {
            let pointers: Result<Vec<_>, _> = vec
                .iter()
                .map(|node| {
                    let cid = Cid::from_str(node.name())?;
                    if let Some(resource) = mut_graph.indexed_resources.get(&cid) {
                        for rsc in resource.iter() {
                            resources_to_fetch.push((cid, rsc.clone()));
                        }
                        cids_to_resolve.push(cid);
                    } else {
                        return Err(anyhow!("Resource not found for instruction {cid}"));
                    }
                    Ok(Pointer::new(cid))
                })
                .collect();

            if let Ok(pointers) = pointers {
                if let Ok(found) = Db::find_instruction_pointers(&pointers, conn) {
                    for receipt in found.iter() {
                        resources_to_fetch.retain(|(cid, _)| *cid != receipt.instruction().cid());
                        cids_to_resolve.retain(|cid| *cid != receipt.instruction().cid());
                        linkmap.insert(receipt.instruction().cid(), receipt.output_as_arg());
                    }

                    if found.len() == vec.len() {
                        last_idx = idx + 1;
                        break;
                    }
                } else {
                    debug!("Receipt not available in the database");
                }
            }
        }

        // Fetch resources from the DHT.
        let resources_to_fetch: FnvHashSet<Resource> =
            resources_to_fetch.into_iter().map(|(_, rsc)| rsc).collect();
        let fetched_resources = fetch_fn(resources_to_fetch).await?;

        // Store awaits outside of the workflow in our In-memory cache for resolving.
        let promises_as_pointers =
            mut_graph
                .awaiting
                .iter()
                .fold(
                    vec![],
                    |mut acc, (in_or_out_flow, cid)| match in_or_out_flow {
                        workflow::Origin::InFlow => acc,
                        workflow::Origin::OutFlow => {
                            acc.push(Pointer::new(*cid));
                            acc
                        }
                    },
                );
        if let Ok(found) = Db::find_instruction_pointers(&promises_as_pointers, conn) {
            for receipt in found.iter() {
                cids_to_resolve.retain(|cid| *cid != receipt.instruction().cid());
                linkmap.insert(receipt.instruction().cid(), receipt.output_as_arg());
            }
        }
        let promises_to_resolve: FnvHashSet<Cid> = cids_to_resolve.into_iter().collect();

        let (ran, run, resume_step) = if last_idx > 0 {
            let pivot = schedule.split_off(last_idx);
            if last_idx >= schedule_length || last_idx == 0 {
                (Some(schedule.to_vec()), pivot, None)
            } else {
                (Some(schedule.to_vec()), pivot, Some(last_idx))
            }
        } else {
            (None, schedule.to_vec(), None)
        };

        Ok(SchedulerContext {
            scheduler: Self {
                linkmap: Arc::new(RwLock::new(linkmap)),
                promises_to_resolve,
                ran,
                run,
                resume_step,
                resources: Arc::new(fetched_resources.into()),
            },
        })
    }

    /// Get the number of tasks that have already ran in the [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    #[allow(dead_code)]
    pub(crate) fn ran_length(&self) -> usize {
        self.ran
            .as_ref()
            .map(|ran| ran.iter().flatten().collect::<Vec<_>>().len())
            .unwrap_or_default()
    }

    /// Get the number of tasks left to run in the [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    #[allow(dead_code)]
    pub(crate) fn run_length(&self) -> usize {
        self.run.iter().flatten().collect::<Vec<_>>().len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils::db::MemoryDb, workflow, Receipt};
    use futures::FutureExt;
    use homestar_invocation::{
        authority::UcanPrf,
        ipld::DagCbor,
        task::{instruction::RunInstruction, Resources},
        test_utils, Invocation, Receipt as InvocationReceipt, Task,
    };
    use homestar_workflow::Workflow;
    use libipld::Ipld;

    #[homestar_runtime_proc_macro::db_async_test]
    fn initialize_task_scheduler() {
        let settings = TestSettings::load();
        let config = Resources::default();
        let (instruction1, instruction2, _) = test_utils::related_wasm_instructions::<Arg>();
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
        let (instruction1, instruction2, _) = test_utils::related_wasm_instructions::<Arg>();
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
            task::Result::Ok(Ipld::Integer(4)),
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
        let (instruction1, instruction2, _) = test_utils::related_wasm_instructions::<Arg>();

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
            task::Result::Ok(Ipld::Integer(4)),
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
            task::Result::Ok(Ipld::Integer(44)),
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

    #[test]
    fn duplicate_task_no_nonce() {
        let config = Resources::default();
        let (instruction1, instruction2, _) = test_utils::related_wasm_instructions::<Arg>();

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

        let workflow = Workflow::new(vec![task1.clone(), task2.clone(), task1.clone()]);
        let builder = workflow::Builder::new(workflow);
        let graph = builder.graph();
        assert!(graph.is_err());
        assert_eq!(
            graph.unwrap_err().to_string(),
            "workflow cannot contain duplicate tasks: use a nonce (nnc field) to ensure uniqueness"
        );
    }

    #[test]
    fn duplicate_task_with_nonce() {
        let config = Resources::default();
        let (instruction1, _) = test_utils::wasm_instruction_with_nonce::<Arg>();
        let (instruction2, _) = test_utils::wasm_instruction_with_nonce::<Arg>();

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

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let builder = workflow::Builder::new(workflow);
        let graph = builder.graph();
        assert!(graph.is_ok());
    }
}
