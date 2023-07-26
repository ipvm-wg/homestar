//! [Scheduler] module for executing a series of tasks for a given
//! [Workflow].
//!
//! [Scheduler]: TaskScheduler

use crate::{
    db::{Connection, Database},
    event_handler::{
        channel::BoundedChannel,
        event::QueryRecord,
        swarm_event::{FoundEvent, ResponseEvent},
        Event,
    },
    network::swarm::CapsuleTag,
    workflow::{self, Builder, IndexedResources, Resource, Vertex},
    Db,
};
use anyhow::{anyhow, Result};
use dagga::Node;
use futures::future::LocalBoxFuture;
use homestar_core::{
    workflow::{InstructionResult, LinkMap, Pointer},
    Workflow,
};
use homestar_wasm::io::Arg;
use indexmap::IndexMap;
use libipld::Cid;
use std::{ops::ControlFlow, str::FromStr, sync::Arc, time::Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::info;

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
    /// [Tasks]: homestar_core::workflow::Task
    pub(crate) ran: Option<Schedule<'a>>,

    /// [ExecutionGraph] of what's left to run for a [Workflow] of `batched`
    /// [Tasks].
    ///
    /// [Tasks]: homestar_core::workflow::Task
    pub(crate) run: Schedule<'a>,

    /// Step/batch to resume from.
    pub(crate) resume_step: Option<usize>,

    /// Resources that tasks within a [Workflow] rely on, retrieved
    /// through over the network, ahead-of-time.
    ///
    /// This is transferred from the [ExecutionGraph] for executing the
    /// schedule by a worker.
    pub(crate) resources: IndexMap<Resource, Vec<u8>>,
}

/// Scheduler context containing the a schedule for executing tasks
/// and a map of [IndexedResources].
pub(crate) struct SchedulerContext<'a> {
    /// Scheduler for a series of tasks, including what's run.
    pub(crate) scheduler: TaskScheduler<'a>,
    /// Map of instructions => resources, for a [Workflow].
    pub(crate) indexed_resources: IndexedResources,
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
        workflow: Workflow<'a, Arg>,
        workflow_cid: Cid,
        settings: Arc<workflow::Settings>,
        event_sender: Arc<mpsc::Sender<Event>>,
        conn: &mut Connection,
        fetch_fn: F,
    ) -> Result<SchedulerContext<'a>>
    where
        F: FnOnce(Vec<Resource>) -> LocalBoxFuture<'a, Result<IndexMap<Resource, Vec<u8>>>>,
    {
        let builder = Builder::new(workflow);
        let graph = builder.graph()?;
        let mut schedule = graph.schedule;
        let schedule_length = schedule.len();
        let mut resources_to_fetch: Vec<Resource> = vec![];
        let resume = schedule
            .iter()
            .enumerate()
            .rev()
            .try_for_each(|(idx, vec)| {
                let folded_pointers = vec.iter().try_fold(vec![], |mut ptrs, node| {
                    let cid = Cid::from_str(node.name())?;
                    graph
                        .indexed_resources
                        .get(&cid)
                        .map(|resource| {
                            resources_to_fetch.push(resource.to_owned());
                            ptrs.push(Pointer::new(cid));
                        })
                        .ok_or_else(|| anyhow!("resource not found for instruction {cid}"))?;
                    Ok::<_, anyhow::Error>(ptrs)
                });

                if let Ok(pointers) = folded_pointers {
                    let pointers_len = pointers.len();
                    match Db::find_instruction_pointers(&pointers, conn) {
                        Ok(found) => {
                            let linkmap = found.iter().fold(
                                LinkMap::<InstructionResult<Arg>>::new(),
                                |mut map, receipt| {
                                    if let Ok(cid) = receipt.instruction().try_into() {
                                        let _ = map.insert(cid, receipt.output_as_arg());
                                    }
                                    map
                                },
                            );

                            if found.len() == vec.len() {
                                ControlFlow::Break((idx + 1, linkmap))
                            } else if !found.is_empty() && found.len() < vec.len() {
                                ControlFlow::Break((idx, linkmap))
                            } else {
                                ControlFlow::Continue(())
                            }
                        }
                        Err(_) => {
                            info!("receipt not available in the database");
                            let (tx, rx) = BoundedChannel::with(pointers_len);
                            for ptr in &pointers {
                                let _ = event_sender.try_send(Event::FindRecord(
                                    QueryRecord::with(ptr.cid(), CapsuleTag::Receipt, tx.clone()),
                                ));
                            }

                            let mut linkmap = LinkMap::<InstructionResult<Arg>>::new();
                            let mut counter = 0;
                            while let Ok(ResponseEvent::Found(Ok(FoundEvent::Receipt(found)))) =
                                rx.recv_deadline(Instant::now() + settings.p2p_check_timeout)
                            {
                                if pointers.contains(&Pointer::new(found.cid())) {
                                    if let Ok(cid) = found.instruction().try_into() {
                                        let stored_receipt =
                                            Db::commit_receipt(workflow_cid, found.clone(), conn)
                                                .unwrap_or(found);

                                        let _ = linkmap.insert(cid, stored_receipt.output_as_arg());
                                        counter += 1;
                                    }
                                }
                            }

                            if counter == pointers_len {
                                ControlFlow::Break((idx + 1, linkmap))
                            } else if counter > 0 && counter < pointers_len {
                                ControlFlow::Break((idx, linkmap))
                            } else {
                                ControlFlow::Continue(())
                            }
                        }
                    }
                } else {
                    ControlFlow::Continue(())
                }
            });

        let fetched = fetch_fn(resources_to_fetch).await?;

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
                        ran: Some(schedule),
                        run: pivot,
                        resume_step: step,
                        resources: fetched,
                    },
                    indexed_resources: graph.indexed_resources,
                })
            }
            _ => Ok(SchedulerContext {
                scheduler: Self {
                    linkmap: Arc::new(LinkMap::<InstructionResult<Arg>>::new().into()),
                    ran: None,
                    run: schedule,
                    resume_step: None,
                    resources: fetched,
                },
                indexed_resources: graph.indexed_resources,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        db::Database,
        test_utils::{self, db::MemoryDb},
        workflow as wf, Receipt,
    };
    use futures::FutureExt;
    use homestar_core::{
        ipld::DagCbor,
        test_utils::workflow as workflow_test_utils,
        workflow::{
            config::Resources, instruction::RunInstruction, prf::UcanPrf, Invocation,
            Receipt as InvocationReceipt, Task,
        },
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

        let db = MemoryDb::setup_connection_pool(&settings.node).unwrap();
        let mut conn = db.conn().unwrap();
        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let workflow_cid = workflow.clone().to_cid().unwrap();
        let workflow_settings = wf::Settings::default();
        let fetch_fn = |_rscs: Vec<Resource>| {
            {
                async {
                    let mut index_map = IndexMap::new();
                    index_map.insert(Resource::Url(instruction1.resource().to_owned()), vec![]);
                    index_map.insert(Resource::Url(instruction2.resource().to_owned()), vec![]);

                    Ok(index_map)
                }
            }
            .boxed_local()
        };

        let (tx, mut _rx) = test_utils::event::setup_event_channel(settings.node);

        let scheduler_ctx = TaskScheduler::init(
            workflow,
            workflow_cid,
            workflow_settings.into(),
            tx.into(),
            &mut conn,
            fetch_fn,
        )
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

        let db = MemoryDb::setup_connection_pool(&settings.node).unwrap();
        let mut conn = db.conn().unwrap();
        let stored_receipt = MemoryDb::store_receipt(receipt.clone(), &mut conn).unwrap();

        assert_eq!(receipt, stored_receipt);

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let workflow_cid = workflow.clone().to_cid().unwrap();
        let workflow_settings = wf::Settings::default();
        let fetch_fn = |_rscs: Vec<Resource>| {
            {
                async {
                    let mut index_map = IndexMap::new();
                    index_map.insert(Resource::Url(instruction1.resource().to_owned()), vec![]);
                    index_map.insert(Resource::Url(instruction2.resource().to_owned()), vec![]);

                    Ok(index_map)
                }
            }
            .boxed_local()
        };

        let (tx, mut _rx) = test_utils::event::setup_event_channel(settings.node);

        let scheduler_ctx = TaskScheduler::init(
            workflow,
            workflow_cid,
            workflow_settings.into(),
            tx.into(),
            &mut conn,
            fetch_fn,
        )
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

        let db = MemoryDb::setup_connection_pool(&settings.node).unwrap();
        let mut conn = db.conn().unwrap();
        let rows_inserted = MemoryDb::store_receipts(vec![receipt1, receipt2], &mut conn).unwrap();
        assert_eq!(2, rows_inserted);

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let workflow_cid = workflow.clone().to_cid().unwrap();
        let workflow_settings = wf::Settings::default();
        let fetch_fn = |_rscs: Vec<Resource>| {
            async {
                let mut index_map = IndexMap::new();
                index_map.insert(Resource::Url(instruction1.resource().to_owned()), vec![]);
                index_map.insert(Resource::Url(instruction2.resource().to_owned()), vec![]);
                Ok(index_map)
            }
            .boxed_local()
        };

        let (tx, mut _rx) = test_utils::event::setup_event_channel(settings.node);

        let scheduler_ctx = TaskScheduler::init(
            workflow,
            workflow_cid,
            workflow_settings.into(),
            tx.into(),
            &mut conn,
            fetch_fn,
        )
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
