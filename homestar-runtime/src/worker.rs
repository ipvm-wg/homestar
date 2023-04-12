#[cfg(feature = "ipfs")]
use crate::workflow::settings::BackoffStrategy;
#[cfg(feature = "ipfs")]
use crate::IpfsCli;
use crate::{
    db::{Connection, Database},
    network::eventloop::Event,
    scheduler::TaskScheduler,
    tasks::{RegisteredTasks, WasmContext},
    workflow::{settings::Settings, Resource, WorkflowInfo},
    Db, Receipt, Workflow,
};
use anyhow::{anyhow, bail, Result};
use crossbeam::channel;
use futures::FutureExt;
#[cfg(feature = "ipfs")]
use futures::StreamExt;
use homestar_core::workflow::{
    prf::UcanPrf, InstructionResult, Pointer, Receipt as InvocationReceipt,
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
    pub(crate) workflow_info: WorkflowInfo,
}

impl<'a> Worker<'a> {
    /// Instantiate a new [Worker] for a [Workflow].
    #[cfg(not(feature = "ipfs"))]
    pub async fn new(
        workflow: Workflow<'a, Arg>,
        workflow_settings: &'a Settings,
        conn: Connection,
    ) -> Result<Worker<'a>> {
        let fetch_fn = |rscs: Vec<Resource>| {
            async { Self::get_resources(rscs, workflow_settings).await }.boxed()
        };

        let scheduler = TaskScheduler::init(workflow.clone(), conn, fetch_fn).await?;
        let workflow_len = workflow.len();
        let workflow_cid = Cid::try_from(workflow)?;
        let workflow_info = WorkflowInfo::new(
            workflow_cid,
            scheduler.resume_step.map_or(0, |step| step),
            workflow_len,
        );
        Ok(Self {
            scheduler,
            workflow_info,
        })
    }

    /// Instantiate a new [Worker] for a [Workflow].
    #[cfg(feature = "ipfs")]
    pub async fn new(
        workflow: Workflow<'a, Arg>,
        workflow_settings: &'a Settings,
        conn: Connection,
        ipfs: &'a IpfsCli,
    ) -> Result<Worker<'a>> {
        let fetch_fn = |rscs: Vec<Resource>| {
            async { Self::get_resources(rscs, workflow_settings, ipfs).await }.boxed()
        };

        let scheduler = TaskScheduler::init(workflow.clone(), conn, fetch_fn).await?;
        let workflow_len = workflow.len();
        let workflow_cid = Cid::try_from(workflow)?;
        let workflow_info = WorkflowInfo::new(
            workflow_cid,
            scheduler.resume_step.map_or(0, |step| step),
            workflow_len,
        );
        Ok(Self {
            scheduler,
            workflow_info,
        })
    }

    /// Run [Worker]'s tasks in task-queue with access to the [Db] object
    /// to use a connection from the Database pool per run.
    pub async fn run(
        self,
        db: Db,
        event_sender: Arc<mpsc::Sender<Event>>,
        settings: Settings,
    ) -> Result<()> {
        self.run_queue(db, event_sender, settings).await
    }

    #[cfg(feature = "ipfs")]
    async fn get_resources(
        resources: Vec<Resource>,
        settings: &'a Settings,
        ipfs: &'a IpfsCli,
    ) -> Result<IndexMap<Resource, Vec<u8>>> {
        async fn fetch(rsc: Resource, client: IpfsCli) -> (Resource, Result<Vec<u8>>) {
            match rsc {
                Resource::Url(url) => {
                    let bytes = match (url.scheme(), url.domain(), url.path()) {
                        ("ipfs", _, _) => client.get_resource(&url).await,
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
                    (Resource::Url(url), bytes)
                }

                Resource::Cid(cid) => {
                    let bytes = client.get_cid(cid).await;
                    (Resource::Cid(cid), bytes)
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
            let inner = res?;
            let answer = inner.1?;
            acc.insert(inner.0, answer);
            Ok::<_, anyhow::Error>(acc)
        })
    }

    /// TODO: Client calls (only) over http(s).
    #[cfg(not(feature = "ipfs"))]
    async fn get_resources<T>(
        _resources: Vec<Resource>,
        _settings: &'a Settings,
    ) -> Result<IndexMap<Resource, T>> {
        Ok(IndexMap::default())
    }

    async fn run_queue(
        mut self,
        db: Db,
        event_sender: Arc<mpsc::Sender<Event>>,
        settings: Settings,
    ) -> Result<()> {
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
                        ("op".into(), fun.to_string().into()),
                        ("workflow".into(), self.workflow_info.cid().into())
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
                                args.resolve(|cid| match self.scheduler.linkmap.get(&cid) {
                                    Some(result) => Ok(result.to_owned()),
                                    None => match Db::find_instruction(
                                        Pointer::new(cid),
                                        &mut db.conn()?,
                                    ) {
                                        Ok(found) => Ok(found.output_as_arg()),
                                        Err(_e) => {
                                            tracing::debug!(
                                                "no related instruction receipt found in the DB"
                                            );
                                            let (sender, receiver) = channel::bounded(1);
                                            event_sender.blocking_send(Event::FindReceipt(
                                                cid,
                                                sender,
                                            ))?;
                                            let found = match receiver.recv_deadline(
                                                Instant::now() + Duration::from_secs(settings.p2p_timeout_secs),
                                            ) {
                                                Ok((found_cid, found)) if found_cid == cid => {
                                                    found
                                                }
                                                Ok(_) => bail!("only one worker channel per worker"),
                                                Err(err) => bail!("error returning invocation receipt for {cid}: {err}"),
                                            };

                                            Ok(found.output_as_arg())
                                        }
                                    },
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

                receipt.set_meta(meta);

                let stored_receipt = Db::store_receipt(receipt, &mut db.conn()?)?;

                // send internal event
                event_sender
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
