//! Worker that runs a [Workflow]'s tasks scheduled by the [TaskScheduler] and
//! sends [Event]'s to the [EventHandler].
//!
//! [Workflow]: homestar_core::Workflow
//! [EventHandler]: crate::event_handler::EventHandler

#[cfg(feature = "websocket-notify")]
use crate::event_handler::event::Replay;
use crate::{
    channel::{AsyncBoundedChannel, AsyncBoundedChannelSender},
    db::Database,
    event_handler::{
        event::{Captured, QueryRecord},
        swarm_event::{FoundEvent, ResponseEvent},
        Event,
    },
    network::swarm::CapsuleTag,
    runner::{ModifiedSet, RunningTaskSet},
    scheduler::{ExecutionGraph, TaskScheduler},
    tasks::{RegisteredTasks, WasmContext},
    workflow::{self, Resource},
    Db, Receipt,
};
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use faststr::FastStr;
use fnv::FnvHashSet;
use futures::{future::BoxFuture, FutureExt};
use homestar_core::{
    bail,
    ipld::DagCbor,
    workflow::{
        error::ResolveError,
        prf::UcanPrf,
        receipt::metadata::{OP_KEY, REPLAYED_KEY, WORKFLOW_KEY, WORKFLOW_NAME_KEY},
        InstructionResult, LinkMap, Pointer, Receipt as InvocationReceipt,
    },
    Workflow,
};
use homestar_wasm::{
    io::{Arg, Output},
    wasmtime::State,
};
use indexmap::IndexMap;
use libipld::{Cid, Ipld};
use std::{collections::BTreeMap, sync::Arc};
use tokio::{
    sync::RwLock,
    task::JoinSet,
    time::{self, Instant},
};
use tracing::{debug, error, info};

/// [JoinSet] of tasks run by a [Worker].
#[allow(dead_code)]
pub(crate) type TaskSet = JoinSet<anyhow::Result<(Output, Pointer, Pointer, Ipld, Ipld)>>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WorkerMessage {
    Dropped(Cid),
}

/// Worker that operates over a given [TaskScheduler].
#[allow(dead_code)]
#[allow(missing_debug_implementations)]
pub(crate) struct Worker<'a, DB: Database> {
    pub(crate) graph: Arc<ExecutionGraph<'a>>,
    pub(crate) event_sender: Arc<AsyncBoundedChannelSender<Event>>,
    pub(crate) runner_sender: AsyncBoundedChannelSender<WorkerMessage>,
    pub(crate) db: DB,
    pub(crate) workflow_name: FastStr,
    pub(crate) workflow_info: Arc<workflow::Info>,
    pub(crate) workflow_settings: Arc<workflow::Settings>,
    pub(crate) workflow_started: NaiveDateTime,
}

impl<'a, DB> Worker<'a, DB>
where
    DB: Database + 'static,
{
    /// Instantiate a new [Worker] for a [Workflow].
    ///
    /// TODO: integrate settings within workflow
    #[allow(dead_code)]
    pub(crate) async fn new<S: Into<FastStr>>(
        workflow: Workflow<'a, Arg>,
        settings: workflow::Settings,
        // Name would be runner specific, separated from core workflow spec.
        name: Option<S>,
        event_sender: Arc<AsyncBoundedChannelSender<Event>>,
        runner_sender: AsyncBoundedChannelSender<WorkerMessage>,
        db: DB,
    ) -> Result<Worker<'a, DB>> {
        let p2p_timeout = settings.p2p_timeout;
        let workflow_len = workflow.len();
        // Need to take ownership here to get the cid.
        let workflow_cid = workflow.to_owned().to_cid()?;

        let builder = workflow::Builder::new(workflow);
        let graph = builder.graph()?;
        let name = name
            .map(|n| n.into())
            .unwrap_or(FastStr::from_string(workflow_cid.to_string()));

        let (workflow_info, timestamp) = workflow::Info::init(
            workflow_cid,
            workflow_len,
            name.clone(),
            graph.indexed_resources.clone(),
            p2p_timeout,
            event_sender.clone(),
            db.conn()?,
        )
        .await?;

        Ok(Self {
            graph: graph.into(),
            event_sender,
            runner_sender,
            db,
            workflow_name: name,
            workflow_info: workflow_info.into(),
            workflow_settings: settings.into(),
            workflow_started: timestamp,
        })
    }

    /// Run [Worker]'s tasks in task-queue with access to the [Db] object
    /// to use connections from the Database pool per run.
    ///
    /// This is the main entry point for running a workflow.
    ///
    /// Within this function, the [Worker] executes tasks and resolves
    /// [Instruction] [Cid]s.
    ///
    /// [Instruction] [Cid]s being awaited on are resolved via 3 lookups:
    ///   * a check in the [LinkMap], which is an in-memory cache of resolved
    ///     [InstructionResult]s (this may have been pre-filled out by
    ///     scheduler initialization);
    ///   * a check in the database, which may have been updated at the point of
    ///   execution;
    ///   * a [Swarm]/DHT query to find the [Receipt] in the network.
    ///
    /// [Instruction]: homestar_core::workflow::Instruction
    /// [Swarm]: crate::network::swarm
    pub(crate) async fn run<F>(self, running_tasks: Arc<RunningTaskSet>, fetch_fn: F) -> Result<()>
    where
        F: FnOnce(FnvHashSet<Resource>) -> BoxFuture<'a, Result<IndexMap<Resource, Vec<u8>>>>,
    {
        let scheduler_ctx = TaskScheduler::init(
            self.graph.clone(), // Arc'ed
            &mut self.db.conn()?,
            fetch_fn,
        )
        .await?;

        self.run_queue(scheduler_ctx.scheduler, running_tasks).await
    }

    #[allow(unused_mut)]
    async fn run_queue(
        mut self,
        mut scheduler: TaskScheduler<'a>,
        running_tasks: Arc<RunningTaskSet>,
    ) -> Result<()> {
        async fn insert_into_map<T>(map: Arc<RwLock<LinkMap<T>>>, key: Cid, value: T)
        where
            T: Clone,
        {
            map.write()
                .await
                .entry(key)
                .or_insert_with(|| value.clone());
        }

        async fn resolve_cid(
            cid: Cid,
            workflow_cid: Cid,
            workflow_settings: Arc<workflow::Settings>,
            linkmap: Arc<RwLock<IndexMap<Cid, InstructionResult<Arg>>>>,
            resources: Arc<RwLock<IndexMap<Resource, Vec<u8>>>>,
            db: impl Database,
            event_sender: Arc<AsyncBoundedChannelSender<Event>>,
        ) -> Result<InstructionResult<Arg>, ResolveError> {
            info!(
                workflow_cid = workflow_cid.to_string(),
                cid = cid.to_string(),
                "resolving cid"
            );

            if let Some(result) = linkmap.read().await.get(&cid) {
                debug!(cid = cid.to_string(), "found in in-memory linkmap");
                Ok(result.to_owned())
            } else if let Some(bytes) = resources.read().await.get(&Resource::Cid(cid)) {
                Ok(InstructionResult::Ok(Arg::Ipld(Ipld::Bytes(
                    bytes.to_vec(),
                ))))
            } else {
                let conn = &mut db.conn()?;
                match Db::find_instruction_by_cid(cid, conn) {
                    Ok(found) => Ok(found.output_as_arg()),
                    Err(_) => {
                        debug!("no related instruction receipt found in the DB");
                        let (tx, rx) = AsyncBoundedChannel::oneshot();
                        let _ = event_sender
                            .send_async(Event::FindRecord(QueryRecord::with(
                                cid,
                                CapsuleTag::Receipt,
                                Some(tx),
                            )))
                            .await;

                        let found = match time::timeout_at(
                            Instant::now() + workflow_settings.p2p_timeout,
                            rx.recv_async(),
                        )
                        .await
                        {
                            Ok(Ok(ResponseEvent::Found(Ok(FoundEvent::Receipt(found))))) => found,
                            Ok(Ok(ResponseEvent::Found(Err(err)))) => {
                                bail!(ResolveError::UnresolvedCid(format!(
                                    "failure in attempting to find event: {err}"
                                )))
                            }
                            Ok(Ok(ResponseEvent::NoPeersAvailable)) => {
                                bail!(ResolveError::UnresolvedCid(
                                    "no peers available to communicate with".to_string()
                                ))
                            }
                            Ok(Ok(_)) => bail!(ResolveError::UnresolvedCid(
                                "wrong or unexpected event message received".to_string(),
                            )),
                            Ok(Err(err)) => bail!(ResolveError::UnresolvedCid(format!(
                                "failure in attempting to find receipt: {err}"
                            ))),
                            Err(err) => bail!(ResolveError::UnresolvedCid(format!(
                                "timeout deadline reached for invocation receipt @ {cid}: {err}",
                            ))),
                        };

                        let receipt =
                            Db::commit_receipt(workflow_cid, found.clone(), conn).unwrap_or(found);
                        let found_result = receipt.output_as_arg();

                        // Store the result in the linkmap for use in next iterations.
                        insert_into_map(linkmap.clone(), cid, found_result.clone()).await;
                        Ok(found_result)
                    }
                }
            }
        }

        // Replay previous receipts if subscriptions are on.
        #[cfg(feature = "websocket-notify")]
        {
            if scheduler.ran.as_ref().is_some_and(|ran| !ran.is_empty()) {
                info!(
                    workflow_cid = self.workflow_info.cid.to_string(),
                    "{} tasks left to run, sending last batch for workflow",
                    scheduler.ran.as_ref().unwrap().len()
                );
                let mut pointers = Vec::new();
                for batch in scheduler
                    .ran
                    .as_mut()
                    .ok_or_else(|| anyhow!("empty scheduler information"))?
                    .drain(..)
                {
                    for node in batch.into_iter() {
                        let vertice = node.into_inner();
                        pointers.push(Pointer::new(vertice.instruction.to_cid()?));
                    }
                }

                let additional_meta = Ipld::Map(BTreeMap::from([
                    (REPLAYED_KEY.into(), Ipld::Bool(true)),
                    (WORKFLOW_KEY.into(), self.workflow_info.cid().into()),
                    (
                        WORKFLOW_NAME_KEY.into(),
                        self.workflow_name.to_string().into(),
                    ),
                ]));

                let _ = self
                    .event_sender
                    .send_async(Event::ReplayReceipts(Replay::with(
                        pointers,
                        Some(additional_meta.clone()),
                    )))
                    .await;
            }
        }

        for batch in scheduler.run.into_iter() {
            let mut task_set = TaskSet::new();
            let mut handles = Vec::new();

            for node in batch.into_iter() {
                let vertice = node.into_inner();
                let invocation_ptr = vertice.invocation;
                let instruction = vertice.instruction;
                let rsc = instruction.resource();
                let parsed = vertice.parsed;
                let fun = parsed.fun().ok_or_else(|| anyhow!("no function defined"))?;

                let args = parsed.into_args();
                let receipt_meta =
                    Ipld::Map(BTreeMap::from([(OP_KEY.into(), fun.to_string().into())]));

                let additional_meta = Ipld::Map(BTreeMap::from([
                    (REPLAYED_KEY.into(), Ipld::Bool(false)),
                    (WORKFLOW_KEY.into(), self.workflow_info.cid().into()),
                    (
                        WORKFLOW_NAME_KEY.into(),
                        self.workflow_name.to_string().into(),
                    ),
                ]));

                match RegisteredTasks::ability(&instruction.op().to_string()) {
                    Some(RegisteredTasks::WasmRun) => {
                        let wasm = scheduler
                            .resources
                            .read()
                            .await
                            .get(&Resource::Url(rsc.to_owned()))
                            .ok_or_else(|| anyhow!("resource not available"))?
                            .to_owned();

                        let instruction_ptr = Pointer::try_from(instruction)?;
                        let state = State::default();
                        let mut wasm_ctx = WasmContext::new(state)?;

                        let db = self.db.clone();
                        let settings = self.workflow_settings.clone();
                        let linkmap = scheduler.linkmap.clone();
                        let resources = scheduler.resources.clone();
                        let event_sender = self.event_sender.clone();
                        let workflow_cid = self.workflow_info.cid();

                        let resolved = args.resolve(move |cid| {
                            resolve_cid(
                                cid,
                                workflow_cid,
                                settings.clone(),
                                linkmap.clone(),
                                resources.clone(),
                                db.clone(),
                                event_sender.clone(),
                            )
                            .boxed()
                        });

                        let handle = task_set.spawn(async move {
                            let resolved = match resolved.await {
                                Ok(inst_result) => inst_result,
                                Err(err) => {
                                    error!(err=?err, "error resolving cid");
                                    return Err(anyhow!("error resolving cid: {err}"))
                                        .with_context(|| {
                                            format!("could not spawn task for cid: {workflow_cid}")
                                        });
                                }
                            };
                            match wasm_ctx.run(wasm, &fun, resolved).await {
                                Ok(output) => Ok((
                                    output,
                                    instruction_ptr,
                                    invocation_ptr,
                                    receipt_meta,
                                    additional_meta,
                                )),
                                Err(e) => Err(anyhow!("cannot execute wasm module: {e}")),
                            }
                        });
                        handles.push(handle);
                    }
                    None => error!(
                        "no valid task/instruction-type referenced by operation: {}",
                        instruction.op()
                    ),
                }
            }

            // Concurrently add handles to Runner's running set.
            running_tasks.append_or_insert(self.workflow_info.cid(), handles);

            while let Some(res) = task_set.join_next().await {
                let (executed, instruction_ptr, invocation_ptr, receipt_meta, add_meta) = match res
                {
                    Ok(Ok(data)) => data,
                    Ok(Err(err)) => {
                        error!(err=?err, "error in running task");
                        break;
                    }
                    Err(err) => {
                        error!(err=?err, "error in running task");
                        break;
                    }
                };
                let output_to_store = Ipld::try_from(executed)?;

                let invocation_receipt = InvocationReceipt::new(
                    invocation_ptr,
                    InstructionResult::Ok(output_to_store),
                    receipt_meta,
                    None,
                    UcanPrf::default(),
                );

                let receipt = Receipt::try_with(instruction_ptr, &invocation_receipt)?;

                scheduler
                    .linkmap
                    .write()
                    .await
                    .insert(receipt.instruction().cid(), receipt.output_as_arg());

                // modify workflow info before progress update, in case
                // that we time out getting info from the network, but later
                // recovered where we last started from.
                if let Some(step) = scheduler.resume_step {
                    let current_progress_count = self.workflow_info.progress_count;
                    Arc::make_mut(&mut self.workflow_info)
                        .set_progress_count(std::cmp::max(current_progress_count, step as u32))
                };

                let stored_receipt =
                    Db::commit_receipt(self.workflow_info.cid, receipt, &mut self.db.conn()?)?;
                debug!(
                    cid = self.workflow_info.cid.to_string(),
                    "commited to database"
                );

                let _ = self
                    .event_sender
                    .send_async(Event::CapturedReceipt(Captured::with(
                        stored_receipt.cid(),
                        self.workflow_info.clone(),
                        Some(add_meta),
                    )))
                    .await;
            }
        }
        Ok(())
    }
}

impl<'a, DB> Drop for Worker<'a, DB>
where
    DB: Database,
{
    fn drop(&mut self) {
        let _ = self
            .runner_sender
            .try_send(WorkerMessage::Dropped(self.workflow_info.cid));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        db::Database,
        test_utils::{self, db::MemoryDb, WorkerBuilder},
        workflow::{self, IndexedResources},
    };
    use homestar_core::{
        ipld::DagCbor,
        test_utils::workflow as workflow_test_utils,
        workflow::{
            config::Resources, instruction::RunInstruction, prf::UcanPrf, Invocation, Task,
        },
    };

    #[homestar_runtime_proc_macro::db_async_test]
    fn initialize_worker() {
        let settings = TestSettings::load();

        let (tx, rx) = test_utils::event::setup_event_channel(settings.clone().node);

        let builder = WorkerBuilder::new(settings.node).with_event_sender(tx);
        let fetch_fn = builder.fetch_fn();
        let db = builder.db();
        let worker = builder.build().await;
        let workflow_cid = worker.workflow_info.cid;

        assert_eq!(worker.workflow_info.cid, workflow_cid);
        assert_eq!(worker.workflow_info.num_tasks, 2);
        assert_eq!(worker.workflow_info.resources.len(), 2);
        assert_eq!(
            worker
                .workflow_info
                .resources
                .iter()
                .collect::<Vec<&Resource>>()
                .len(),
            1
        );

        let running_tasks = Arc::new(RunningTaskSet::new());
        let worker_workflow_cid = worker.workflow_info.cid;
        worker.run(running_tasks.clone(), fetch_fn).await.unwrap();
        assert_eq!(running_tasks.len(), 1);
        assert!(running_tasks.contains_key(&worker_workflow_cid));
        assert_eq!(running_tasks.get(&worker_workflow_cid).unwrap().len(), 2);

        // first time check DHT for workflow info
        let workflow_info_event = rx.recv_async().await.unwrap();

        // we should have received 2 receipts
        let next_run_receipt = rx.recv_async().await.unwrap();
        let next_next_run_receipt = rx.recv_async().await.unwrap();

        match workflow_info_event {
            Event::FindRecord(QueryRecord { cid, .. }) => assert_eq!(cid, worker_workflow_cid),
            _ => panic!("Wrong event type"),
        };

        let (next_receipt, _wf_info) = match next_run_receipt {
            Event::CapturedReceipt(Captured {
                receipt: next_receipt,
                ..
            }) => {
                let stored = workflow::Stored::default(Pointer::new(workflow_cid), 2);
                let mut info = workflow::Info::default(stored);
                info.increment_progress(next_receipt);

                (next_receipt, info)
            }
            _ => panic!("Wrong event type"),
        };

        let (_next_next_receipt, wf_info) = match next_next_run_receipt {
            Event::CapturedReceipt(Captured {
                receipt: next_next_receipt,
                ..
            }) => {
                let stored = workflow::Stored::default(Pointer::new(workflow_cid), 2);
                let mut info = workflow::Info::default(stored);
                info.increment_progress(next_next_receipt);

                assert_ne!(next_next_receipt, next_receipt);

                (next_next_receipt, info)
            }
            _ => panic!("Wrong event type"),
        };

        assert!(rx.recv_async().await.is_err());

        let mut conn = db.conn().unwrap();
        let (_, workflow_info) = MemoryDb::get_workflow_info(workflow_cid, &mut conn).unwrap();

        assert_eq!(workflow_info.num_tasks, 2);
        assert_eq!(workflow_info.cid, workflow_cid);
        assert_eq!(workflow_info.progress.len(), 2);
        assert_eq!(workflow_info.resources.len(), 2);
        assert_eq!(wf_info.progress_count, 2);
        assert_eq!(wf_info.progress_count, workflow_info.progress_count);
    }

    #[homestar_runtime_proc_macro::db_async_test]
    async fn initialize_worker_with_run_instructions_and_run() {
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

        let (tx, rx) = test_utils::event::setup_event_channel(settings.node.clone());

        let builder = WorkerBuilder::new(settings.node)
            .with_event_sender(tx)
            .with_tasks(vec![task1, task2]);
        let fetch_fn = builder.fetch_fn();
        let db = builder.db();
        let workflow_cid = builder.workflow_cid();

        let mut index_map = IndexMap::new();
        index_map.insert(
            instruction1.clone().to_cid().unwrap(),
            vec![Resource::Url(instruction1.resource().to_owned())],
        );
        index_map.insert(
            instruction2.clone().to_cid().unwrap(),
            vec![Resource::Url(instruction2.resource().to_owned())],
        );

        let mut conn = db.conn().unwrap();
        let _ = MemoryDb::store_workflow(
            workflow::Stored::new_with_resources(
                Pointer::new(workflow_cid),
                None,
                builder.workflow_len() as i32,
                IndexedResources::new(index_map),
            ),
            &mut conn,
        );
        let _ = MemoryDb::commit_receipt(workflow_cid, receipt.clone(), &mut conn).unwrap();

        let worker = builder.build().await;
        let (_, info) = MemoryDb::get_workflow_info(workflow_cid, &mut conn).unwrap();

        assert_eq!(Arc::new(info), worker.workflow_info);
        assert_eq!(worker.workflow_info.cid, workflow_cid);
        assert_eq!(worker.workflow_info.num_tasks, 2);
        assert_eq!(worker.workflow_info.resources.len(), 2);
        assert_eq!(
            worker
                .workflow_info
                .resources
                .iter()
                .collect::<Vec<&Resource>>()
                .len(),
            1
        );

        let running_tasks = Arc::new(RunningTaskSet::new());
        let worker_workflow_cid = worker.workflow_info.cid;
        worker.run(running_tasks.clone(), fetch_fn).await.unwrap();
        assert_eq!(running_tasks.len(), 1);
        assert!(running_tasks.contains_key(&worker_workflow_cid));
        assert_eq!(running_tasks.get(&worker_workflow_cid).unwrap().len(), 1);

        // First receipt is a replay receipt.
        #[cfg(feature = "websocket-notify")]
        {
            let replay_msg = rx.recv_async().await.unwrap();
            assert!(matches!(replay_msg, Event::ReplayReceipts(_)));
        }

        // we should have received 1 receipt
        let next_run_receipt = rx.recv_async().await.unwrap();

        let (_next_receipt, wf_info) = match next_run_receipt {
            Event::CapturedReceipt(Captured {
                receipt: next_receipt,
                ..
            }) => {
                let next_receipt = MemoryDb::find_receipt_by_cid(next_receipt, &mut conn).unwrap();
                let stored = workflow::Stored::default(Pointer::new(workflow_cid), 2);
                let mut info = workflow::Info::default(stored);
                info.increment_progress(next_receipt.cid());

                assert_ne!(next_receipt, receipt);

                (next_receipt, info)
            }
            _ => panic!("Wrong event type"),
        };

        assert!(rx.recv_async().await.is_err());

        let mut conn = db.conn().unwrap();
        let (_, workflow_info) = MemoryDb::get_workflow_info(workflow_cid, &mut conn).unwrap();

        assert_eq!(workflow_info.num_tasks, 2);
        assert_eq!(workflow_info.cid, workflow_cid);
        assert_eq!(workflow_info.progress.len(), 2);
        assert_eq!(wf_info.progress_count, 2);
        assert_eq!(wf_info.progress_count, workflow_info.progress_count);
    }

    #[homestar_runtime_proc_macro::db_async_test]
    fn initialize_worker_with_all_receipted_instruction() {
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

        let (tx, rx) = test_utils::event::setup_event_channel(settings.node.clone());

        let builder = WorkerBuilder::new(settings.node)
            .with_event_sender(tx.into())
            .with_tasks(vec![task1, task2]);
        let db = builder.db();
        let workflow_cid = builder.workflow_cid();

        let mut index_map = IndexMap::new();
        index_map.insert(
            instruction1.clone().to_cid().unwrap(),
            vec![Resource::Url(instruction1.resource().to_owned())],
        );
        index_map.insert(
            instruction2.clone().to_cid().unwrap(),
            vec![Resource::Url(instruction2.resource().to_owned())],
        );

        let mut conn = db.conn().unwrap();
        let _ = MemoryDb::store_workflow(
            workflow::Stored::new_with_resources(
                Pointer::new(workflow_cid),
                None,
                builder.workflow_len() as i32,
                IndexedResources::new(index_map),
            ),
            &mut conn,
        );

        let rows_inserted =
            MemoryDb::store_receipts(vec![receipt1.clone(), receipt2.clone()], &mut conn).unwrap();
        assert_eq!(2, rows_inserted);

        let _ = MemoryDb::store_workflow_receipt(workflow_cid, receipt1.cid(), &mut conn).unwrap();
        let _ = MemoryDb::store_workflow_receipt(workflow_cid, receipt2.cid(), &mut conn).unwrap();

        let worker = builder.build().await;

        assert_eq!(worker.workflow_info.cid, workflow_cid);
        assert_eq!(worker.workflow_info.num_tasks, 2);
        assert_eq!(worker.workflow_info.resources.len(), 2);
        assert_eq!(
            worker
                .workflow_info
                .resources
                .iter()
                .collect::<Vec<&Resource>>()
                .len(),
            1
        );

        let mut conn = db.conn().unwrap();
        let (_, workflow_info) = MemoryDb::get_workflow_info(workflow_cid, &mut conn).unwrap();

        assert_eq!(workflow_info.num_tasks, 2);
        assert_eq!(workflow_info.cid, workflow_cid);
        assert_eq!(workflow_info.progress.len(), 2);

        assert!(rx.try_recv().is_err())
    }
}
