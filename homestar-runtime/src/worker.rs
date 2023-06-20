#[cfg(feature = "ipfs")]
use crate::workflow::settings::BackoffStrategy;
#[cfg(feature = "ipfs")]
use crate::IpfsCli;
use crate::{
    db::{Connection, Database},
    network::eventloop::{Event, FoundEvent},
    scheduler::TaskScheduler,
    tasks::{RegisteredTasks, WasmContext},
    workflow::{self, Resource},
    Db, Receipt,
};
use anyhow::{anyhow, Result};
use crossbeam::channel;
use futures::FutureExt;
#[cfg(feature = "ipfs")]
use futures::StreamExt;
use homestar_core::{
    workflow::{
        error::ResolveError,
        prf::UcanPrf,
        receipt::metadata::{OP_KEY, WORKFLOW_KEY},
        InstructionResult, Pointer, Receipt as InvocationReceipt,
    },
    Workflow,
};
use homestar_wasm::{io::Arg, wasmtime::State};
use indexmap::IndexMap;
use libipld::{Cid, Ipld};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::mpsc, task::JoinSet};
#[cfg(feature = "ipfs")]
use tryhard::RetryFutureConfig;

/// Worker that operates over a given [TaskScheduler].
#[derive(Debug)]
pub struct Worker<'a> {
    pub(crate) scheduler: TaskScheduler<'a>,
    pub(crate) event_sender: Arc<mpsc::Sender<Event>>,
    pub(crate) workflow_info: &'a mut workflow::Info,
    pub(crate) workflow_settings: &'a workflow::Settings,
}

impl<'a> Worker<'a> {
    /// Instantiate a new [Worker] for a [Workflow].
    #[cfg(not(feature = "ipfs"))]
    pub async fn new(
        workflow: Workflow<'a, Arg>,
        workflow_info: &'a mut workflow::Info,
        workflow_settings: &'a workflow::Settings,
        event_sender: Arc<mpsc::Sender<Event>>,
        mut conn: Connection,
    ) -> Result<Worker<'a>> {
        let fetch_fn = |rscs: Vec<Resource>| {
            async { Self::get_resources(rscs, workflow_settings).await }.boxed()
        };

        let scheduler = TaskScheduler::init(
            workflow.clone(),
            workflow_settings,
            event_sender.clone(),
            &mut conn,
            fetch_fn,
        )
        .await?;

        Ok(Self {
            scheduler,
            workflow_info,
            event_sender,
            workflow_settings,
        })
    }

    /// Instantiate a new [Worker] for a [Workflow].
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    pub async fn new(
        workflow: Workflow<'a, Arg>,
        workflow_info: &'a mut workflow::Info,
        workflow_settings: &'a workflow::Settings,
        event_sender: Arc<mpsc::Sender<Event>>,
        mut conn: Connection,
        ipfs: &'a IpfsCli,
    ) -> Result<Worker<'a>> {
        let fetch_fn = |rscs: Vec<Resource>| {
            async { Self::get_resources(rscs, workflow_settings, ipfs).await }.boxed()
        };

        let scheduler = TaskScheduler::init(
            workflow,
            workflow_settings,
            event_sender.clone(),
            &mut conn,
            fetch_fn,
        )
        .await?;

        Ok(Self {
            scheduler,
            event_sender,
            workflow_info,
            workflow_settings,
        })
    }

    /// Run [Worker]'s tasks in task-queue with access to the [Db] object
    /// to use a connection from the Database pool per run.
    pub async fn run(self, db: impl Database) -> Result<()> {
        self.run_queue(db).await
    }

    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    async fn get_resources(
        resources: Vec<Resource>,
        settings: &'a workflow::Settings,
        ipfs: &'a IpfsCli,
    ) -> Result<IndexMap<Resource, Vec<u8>>> {
        /// TODO: http(s) calls
        async fn fetch(rsc: Resource, client: IpfsCli) -> Result<(Resource, Result<Vec<u8>>)> {
            match rsc {
                Resource::Url(url) => {
                    let bytes = match (url.scheme(), url.domain(), url.path()) {
                        ("ipfs", Some(cid), _) => {
                            let cid = Cid::try_from(cid)?;
                            client.get_cid(cid).await
                        }
                        (_, Some("ipfs.io"), _) => client.get_resource(&url).await,
                        (_, _, path) if path.contains("/ipfs/") || path.contains("/ipns/") => {
                            client.get_resource(&url).await
                        }
                        (_, Some(domain), _) => {
                            let split: Vec<&str> = domain.splitn(3, '.').collect();
                            // subdomain-gateway case:
                            // <https://bafybeiemxf5abjwjbikoz4mc3a3dla6ual3jsgpdr4cjr3oz3evfyavhwq.ipfs.dweb.link/wiki/>
                            if let (Ok(_cid), "ipfs") = (Cid::try_from(split[0]), split[1]) {
                                client.get_resource(&url).await
                            } else {
                                // TODO: reqwest call
                                todo!()
                            }
                        }
                        // TODO: reqwest call
                        (_, _, _) => todo!(),
                    };
                    Ok((Resource::Url(url), bytes))
                }

                Resource::Cid(cid) => {
                    let bytes = client.get_cid(cid).await;
                    Ok((Resource::Cid(cid), bytes))
                }
            }
        }
        let num_requests = resources.len();
        futures::stream::iter(resources.into_iter().map(|rsc| async move {
            // Have to enumerate configs here, as type variants are different
            // and cannot be matched on.
            match settings.retry_backoff_strategy {
                BackoffStrategy::Exponential => {
                    tryhard::retry_fn(|| {
                        let rsc = rsc.clone();
                        let client = ipfs.clone();
                        tokio::spawn(async move { fetch(rsc, client).await })
                    })
                    .with_config(
                        RetryFutureConfig::new(settings.retries)
                            .exponential_backoff(Duration::from_millis(
                                settings.retry_initial_delay_ms,
                            ))
                            .max_delay(Duration::from_secs(settings.retry_max_delay_secs)),
                    )
                    .await
                }
                BackoffStrategy::Fixed => {
                    tryhard::retry_fn(|| {
                        let rsc = rsc.clone();
                        let client = ipfs.clone();
                        tokio::spawn(async move { fetch(rsc, client).await })
                    })
                    .with_config(
                        RetryFutureConfig::new(settings.retries)
                            .fixed_backoff(Duration::from_millis(settings.retry_initial_delay_ms))
                            .max_delay(Duration::from_secs(settings.retry_max_delay_secs)),
                    )
                    .await
                }
                BackoffStrategy::Linear => {
                    tryhard::retry_fn(|| {
                        let rsc = rsc.clone();
                        let client = ipfs.clone();
                        tokio::spawn(async move { fetch(rsc, client).await })
                    })
                    .with_config(
                        RetryFutureConfig::new(settings.retries)
                            .linear_backoff(Duration::from_millis(settings.retry_initial_delay_ms))
                            .max_delay(Duration::from_secs(settings.retry_max_delay_secs)),
                    )
                    .await
                }
                BackoffStrategy::None => {
                    tryhard::retry_fn(|| {
                        let rsc = rsc.clone();
                        let client = ipfs.clone();
                        tokio::spawn(async move { fetch(rsc, client).await })
                    })
                    .with_config(RetryFutureConfig::new(settings.retries).no_backoff())
                    .await
                }
            }
        }))
        .buffer_unordered(num_requests)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .try_fold(IndexMap::default(), |mut acc, res| {
            let inner = res??;
            let answer = inner.1?;
            acc.insert(inner.0, answer);
            Ok::<_, anyhow::Error>(acc)
        })
    }

    /// TODO: Client calls (only) over http(s).
    #[cfg(not(feature = "ipfs"))]
    async fn get_resources<T>(
        _resources: Vec<Resource>,
        _settings: &'a workflow::Settings,
    ) -> Result<IndexMap<Resource, T>> {
        Ok(IndexMap::default())
    }

    async fn run_queue(mut self, db: impl Database) -> Result<()> {
        for batch in self.scheduler.run.into_iter() {
            let (mut set, _handles) = batch.into_iter().try_fold(
                (JoinSet::new(), vec![]),
                |(mut set, mut handles), node| {
                    let vertice = node.into_inner();
                    let invocation_ptr = vertice.invocation;
                    let instruction = vertice.instruction;
                    let rsc = instruction.resource();
                    let parsed = vertice.parsed;
                    let fun = parsed.fun().ok_or_else(|| anyhow!("no function defined"))?;

                    let args = parsed.into_args();
                    let meta = Ipld::Map(BTreeMap::from([
                        (OP_KEY.into(), fun.to_string().into()),
                        (WORKFLOW_KEY.into(), self.workflow_info.cid().into())
                    ]));

                    match RegisteredTasks::ability(&instruction.op().to_string()) {
                        Some(RegisteredTasks::WasmRun) => {
                            let wasm = self
                                .scheduler
                                .resources
                                .get(&Resource::Url(rsc.to_owned()))
                                .ok_or_else(|| anyhow!("resource not available"))?
                                .to_owned();

                            let instruction_ptr = Pointer::try_from(instruction)?;
                            let state = State::default();
                            let mut wasm_ctx = WasmContext::new(state)?;
                            let resolved =
                                args.resolve(|cid| if let Some(result) = self.scheduler.linkmap.get(&cid) {
                                    Ok(result.to_owned())
                                } else {
                                    match Db::find_instruction(
                                        Pointer::new(cid),
                                        &mut db.conn()?,
                                    ) {
                                        Ok(found) => Ok(found.output_as_arg()),
                                        Err(_e) => {
                                            tracing::debug!(
                                                "no related instruction receipt found in the DB"
                                            );
                                            let (sender, receiver) = channel::bounded(1);
                                            self.event_sender.blocking_send(Event::FindReceipt(
                                                cid,
                                                sender,
                                            )).map_err(|err| ResolveError::TransportError(err.to_string()))?;

                                            let found = match receiver.recv_deadline(
                                                Instant::now() + Duration::from_secs(self.workflow_settings.p2p_timeout_secs),
                                            ) {
                                                Ok((found_cid, FoundEvent::Receipt(found))) if found_cid == cid => {
                                                    found
                                                }
                                                Ok(_) =>
                                                    homestar_core::bail!(
                                                        ResolveError::UnresolvedCidError(
                                                            "wrong or unexpected event message received".to_string())
                                                    ),
                                                Err(err) =>
                                                    homestar_core::bail!(ResolveError::UnresolvedCidError(
                                                        format!("timeout deadline reached for invocation receipt @ {cid}: {err}"))
                                                    ),
                                            };

                                            Ok(found.output_as_arg())
                                        }
                                    }
                                })?;

                            let handle = set.spawn(async move {
                                match wasm_ctx.run(wasm, &fun, resolved).await {
                                    Ok(output) => {
                                        Ok((output, instruction_ptr, invocation_ptr, meta))
                                    }
                                    Err(e) => Err(anyhow!("cannot execute wasm module: {e}")),
                                }
                            });
                            handles.push(handle);
                        }
                        None => tracing::error!(
                            "no valid task/instruction-type referenced by operation: {}",
                            instruction.op()
                        ),
                    };

                    Ok::<_, anyhow::Error>((set, handles))
                },
            )?;

            while let Some(res) = set.join_next().await {
                let (executed, instruction_ptr, invocation_ptr, meta) = res??;
                let output_to_store = Ipld::try_from(executed)?;

                let invocation_receipt = InvocationReceipt::new(
                    invocation_ptr,
                    InstructionResult::Ok(output_to_store),
                    Ipld::Null,
                    None,
                    UcanPrf::default(),
                );

                let mut receipt = Receipt::try_with(instruction_ptr, &invocation_receipt)?;
                self.scheduler.linkmap.insert(
                    Cid::try_from(receipt.instruction())?,
                    receipt.output_as_arg(),
                );

                // set receipt metadata
                receipt.set_meta(meta);
                // modify workflow info before progress update, in case
                // that we timed out getting info from the network, but later
                // recovered where we last started from.
                if let Some(step) = self.scheduler.resume_step {
                    self.workflow_info.set_progress_count(std::cmp::max(
                        self.workflow_info.progress_count,
                        step as u32,
                    ))
                };

                let stored_receipt = Db::store_receipt(receipt, &mut db.conn()?)?;

                // send internal event
                self.event_sender
                    .send(Event::CapturedReceipt(
                        stored_receipt,
                        self.workflow_info.clone(),
                    ))
                    .await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(feature = "ipfs")]
    use crate::IpfsCli;

    use crate::{db::Database, network::EventLoop, settings::Settings, test_utils, workflow as wf};
    use homestar_core::{
        ipld::DagCbor,
        test_utils::workflow as workflow_test_utils,
        workflow::{
            config::Resources, instruction::RunInstruction, prf::UcanPrf, Invocation, Task,
        },
    };

    #[tokio::test]
    async fn initialize_worker() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            workflow_test_utils::related_wasm_instructions::<Arg>();
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

        let db = test_utils::db::MemoryDb::setup_connection_pool(Settings::load().unwrap().node())
            .unwrap();
        let mut conn = db.conn().unwrap();

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let workflow_cid = workflow.clone().to_cid().unwrap();
        let workflow_settings = wf::Settings::default();
        let settings = Settings::load().unwrap();

        #[cfg(feature = "ipfs")]
        let (tx, mut rx) = EventLoop::setup_channel(settings.node());
        #[cfg(not(feature = "ipfs"))]
        let (tx, mut _rx) = EventLoop::setup_channel(settings.node());

        #[cfg(feature = "ipfs")]
        let ipfs = IpfsCli::default();

        let mut workflow_info = wf::Info::gather(
            workflow.clone(),
            &workflow_settings,
            &tx.clone().into(),
            &mut conn,
        )
        .await
        .unwrap();

        #[cfg(feature = "ipfs")]
        let worker = Worker::new(
            workflow,
            &mut workflow_info,
            &workflow_settings,
            tx.into(),
            conn,
            &ipfs,
        )
        .await
        .unwrap();
        #[cfg(not(feature = "ipfs"))]
        let worker = Worker::new(
            workflow,
            &mut workflow_info,
            &workflow_settings,
            tx.into(),
            conn,
        )
        .await
        .unwrap();

        assert!(worker.scheduler.linkmap.is_empty());
        assert!(worker.scheduler.ran.is_none());
        assert_eq!(worker.scheduler.run.len(), 2);
        assert_eq!(worker.scheduler.resume_step, None);
        assert_eq!(worker.workflow_info.cid, workflow_cid);
        assert_eq!(worker.workflow_info.num_tasks, 2);

        #[cfg(feature = "ipfs")]
        {
            let worker_workflow_cid = worker.workflow_info.cid;
            worker.run(db.clone()).await.unwrap();

            // first time check DHT for workflow info
            let workflow_info_event = rx.recv().await.unwrap();
            // we should have received 2 receipts
            let next_run_receipt = rx.recv().await.unwrap();
            let next_next_run_receipt = rx.recv().await.unwrap();

            match workflow_info_event {
                Event::FindWorkflow(cid, _) => assert_eq!(cid, worker_workflow_cid),
                _ => panic!("Wrong event type"),
            };

            let (next_receipt, _wf_info) = match next_run_receipt {
                Event::CapturedReceipt(next_receipt, _) => {
                    let mut conn = db.conn().unwrap();
                    let _ = Db::store_workflow_receipt(workflow_cid, next_receipt.cid(), &mut conn);
                    let mut info = workflow::Info::default(workflow_cid, 2);
                    info.increment_progress(next_receipt.cid());

                    (next_receipt, info)
                }
                _ => panic!("Wrong event type"),
            };

            let (_next_next_receipt, wf_info) = match next_next_run_receipt {
                Event::CapturedReceipt(next_next_receipt, _) => {
                    let mut conn = db.conn().unwrap();
                    let _ = Db::store_workflow_receipt(
                        workflow_cid,
                        next_next_receipt.cid(),
                        &mut conn,
                    );
                    let mut info = workflow::Info::default(workflow_cid, 2);
                    info.increment_progress(next_next_receipt.cid());

                    assert_ne!(next_next_receipt, next_receipt);

                    (next_next_receipt, info)
                }
                _ => panic!("Wrong event type"),
            };

            assert!(rx.recv().await.is_none());

            let mut conn = db.conn().unwrap();
            let (stored_info, receipt_cids) =
                test_utils::db::MemoryDb::join_workflow_with_receipts(workflow_cid, &mut conn)
                    .unwrap();

            assert_eq!(stored_info.num_tasks, 2);
            assert_eq!(stored_info.cid.cid(), workflow_cid);
            assert_eq!(receipt_cids.len(), 2);
            assert_eq!(wf_info.progress_count, 2);
        }
    }

    #[tokio::test]
    async fn initialize_worker_with_run_instructions_and_run() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            workflow_test_utils::related_wasm_instructions::<Arg>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1.clone()),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.into(),
            UcanPrf::default(),
        );

        let db = test_utils::db::MemoryDb::setup_connection_pool(Settings::load().unwrap().node())
            .unwrap();
        let mut conn = db.conn().unwrap();

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

        let _ = test_utils::db::MemoryDb::store_receipt(receipt.clone(), &mut conn).unwrap();

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let workflow_cid = workflow.clone().to_cid().unwrap();
        let workflow_settings = wf::Settings::default();
        let settings = Settings::load().unwrap();

        // already have stored workflow information (from a previous run)
        let _ = test_utils::db::MemoryDb::store_workflow(
            workflow::Stored::new(Pointer::new(workflow_cid), workflow.len() as i32),
            &mut conn,
        )
        .unwrap();
        let _ = test_utils::db::MemoryDb::store_workflow_receipt(
            workflow_cid,
            receipt.cid(),
            &mut conn,
        )
        .unwrap();

        #[cfg(feature = "ipfs")]
        let (tx, mut rx) = EventLoop::setup_channel(settings.node());
        #[cfg(not(feature = "ipfs"))]
        let (tx, mut _rx) = EventLoop::setup_channel(settings.node());

        #[cfg(feature = "ipfs")]
        let ipfs = IpfsCli::default();

        let mut workflow_info = wf::Info::gather(
            workflow.clone(),
            &workflow_settings,
            &tx.clone().into(),
            &mut conn,
        )
        .await
        .unwrap();

        #[cfg(feature = "ipfs")]
        let worker = Worker::new(
            workflow,
            &mut workflow_info,
            &workflow_settings,
            tx.into(),
            conn,
            &ipfs,
        )
        .await
        .unwrap();
        #[cfg(not(feature = "ipfs"))]
        let worker = Worker::new(
            workflow,
            &mut workflow_info,
            &workflow_settings,
            tx.into(),
            conn,
        )
        .await
        .unwrap();

        assert_eq!(worker.scheduler.linkmap.len(), 1);
        assert!(worker
            .scheduler
            .linkmap
            .contains_key(&instruction1.to_cid().unwrap()));
        assert_eq!(worker.scheduler.ran.as_ref().unwrap().len(), 1);
        assert_eq!(worker.scheduler.run.len(), 1);
        assert_eq!(worker.scheduler.resume_step, Some(1));
        assert_eq!(worker.workflow_info.cid, workflow_cid);
        assert_eq!(worker.workflow_info.num_tasks, 2);

        #[cfg(feature = "ipfs")]
        {
            worker.run(db.clone()).await.unwrap();

            // we should have received 1 receipt
            let next_run_receipt = rx.recv().await.unwrap();

            let (_next_receipt, wf_info) = match next_run_receipt {
                Event::CapturedReceipt(next_receipt, _) => {
                    let mut conn = db.conn().unwrap();
                    let _ = Db::store_workflow_receipt(workflow_cid, next_receipt.cid(), &mut conn);
                    let mut info = workflow::Info::default(workflow_cid, 2);
                    info.increment_progress(next_receipt.cid());

                    assert_ne!(next_receipt, receipt);

                    (next_receipt, info)
                }
                _ => panic!("Wrong event type"),
            };

            assert!(rx.recv().await.is_none());

            let mut conn = db.conn().unwrap();
            let (stored_info, receipt_cids) =
                test_utils::db::MemoryDb::join_workflow_with_receipts(workflow_cid, &mut conn)
                    .unwrap();

            assert_eq!(stored_info.num_tasks, 2);
            assert_eq!(stored_info.cid.cid(), workflow_cid);
            assert_eq!(receipt_cids.len(), 2);
            assert_eq!(wf_info.progress_count, 2);
        }
    }

    #[tokio::test]
    async fn initialize_wroker_with_all_receipted_instruction() {
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

        let db = test_utils::db::MemoryDb::setup_connection_pool(Settings::load().unwrap().node())
            .unwrap();
        let mut conn = db.conn().unwrap();

        let rows_inserted = test_utils::db::MemoryDb::store_receipts(
            vec![receipt1.clone(), receipt2.clone()],
            &mut conn,
        )
        .unwrap();

        assert_eq!(2, rows_inserted);

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let workflow_cid = workflow.clone().to_cid().unwrap();
        let workflow_settings = wf::Settings::default();
        let settings = Settings::load().unwrap();

        // already have stored workflow information (from a previous run)
        let _ = test_utils::db::MemoryDb::store_workflow(
            workflow::Stored::new(Pointer::new(workflow_cid), workflow.len() as i32),
            &mut conn,
        )
        .unwrap();
        let _ = test_utils::db::MemoryDb::store_workflow_receipt(
            workflow_cid,
            receipt1.cid(),
            &mut conn,
        )
        .unwrap();
        let _ = test_utils::db::MemoryDb::store_workflow_receipt(
            workflow_cid,
            receipt2.cid(),
            &mut conn,
        )
        .unwrap();

        #[cfg(feature = "ipfs")]
        let (tx, mut rx) = EventLoop::setup_channel(settings.node());
        #[cfg(not(feature = "ipfs"))]
        let (tx, mut rx) = EventLoop::setup_channel(settings.node());

        #[cfg(feature = "ipfs")]
        let ipfs = IpfsCli::default();

        let mut workflow_info = wf::Info::gather(
            workflow.clone(),
            &workflow_settings,
            &tx.clone().into(),
            &mut conn,
        )
        .await
        .unwrap();

        #[cfg(feature = "ipfs")]
        let worker = Worker::new(
            workflow,
            &mut workflow_info,
            &workflow_settings,
            tx.into(),
            conn,
            &ipfs,
        )
        .await
        .unwrap();
        #[cfg(not(feature = "ipfs"))]
        let worker = Worker::new(
            workflow,
            &mut workflow_info,
            &workflow_settings,
            tx.into(),
            conn,
        )
        .await
        .unwrap();

        assert_eq!(worker.scheduler.linkmap.len(), 1);
        assert!(!worker
            .scheduler
            .linkmap
            .contains_key(&instruction1.to_cid().unwrap()));
        assert!(worker
            .scheduler
            .linkmap
            .contains_key(&instruction2.to_cid().unwrap()));
        assert_eq!(worker.scheduler.ran.as_ref().unwrap().len(), 2);
        assert!(worker.scheduler.run.is_empty());
        assert_eq!(worker.scheduler.resume_step, None);
        assert_eq!(worker.workflow_info.cid, workflow_cid);
        assert_eq!(worker.workflow_info.num_tasks, 2);

        let mut conn = db.conn().unwrap();
        let (stored_info, receipt_cids) =
            test_utils::db::MemoryDb::join_workflow_with_receipts(workflow_cid, &mut conn).unwrap();

        assert_eq!(stored_info.num_tasks, 2);
        assert_eq!(stored_info.cid.cid(), workflow_cid);
        assert_eq!(receipt_cids.len(), 2);

        assert!(rx.try_recv().is_err())
    }
}
