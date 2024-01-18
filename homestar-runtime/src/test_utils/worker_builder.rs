//! Module for building out [Worker]s for testing purposes.

use super::{db::MemoryDb, event};
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    channel::AsyncChannelSender,
    db::Database,
    event_handler::Event,
    settings,
    tasks::Fetch,
    worker::WorkerMessage,
    workflow::{self, Resource},
    Settings, Worker,
};
use fnv::FnvHashSet;
use futures::{future::BoxFuture, FutureExt};
use homestar_invocation::{
    authority::UcanPrf,
    ipld::DagCbor,
    task::{instruction::RunInstruction, Resources},
    Task,
};
use homestar_wasm::io::Arg;
use homestar_workflow::Workflow;
use indexmap::IndexMap;
use libipld::Cid;

/// Utility structure for building out [Worker]s for testing purposes.
///
/// [Worker]: crate::Worker
#[cfg(feature = "ipfs")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
pub(crate) struct WorkerBuilder<'a> {
    /// In-memory database for testing.
    db: MemoryDb,
    /// Event channel sender.
    event_sender: AsyncChannelSender<Event>,
    /// [IPFS client].
    ///
    /// [IPFS client]: crate::network::IpfsCli
    ipfs: IpfsCli,
    /// Runner channel sender.
    runner_sender: AsyncChannelSender<WorkerMessage>,
    /// Name of the workflow.
    name: Option<String>,
    /// [Workflow] to run.
    workflow: Workflow<'a, Arg>,
    /// [Workflow] settings.
    workflow_settings: workflow::Settings,
    /// Network settings.
    network_settings: settings::Dht,
}

/// Utility structure for building out [Worker]s for testing purposes.
///
/// [Worker]: crate::Worker
#[cfg(not(feature = "ipfs"))]
pub(crate) struct WorkerBuilder<'a> {
    /// In-memory database for testing.
    db: MemoryDb,
    /// Event channel sender.
    event_sender: AsyncChannelSender<Event>,
    /// Runner channel sender.
    runner_sender: AsyncChannelSender<WorkerMessage>,
    /// Name of the workflow.
    name: Option<String>,
    /// [Workflow] to run.
    workflow: Workflow<'a, Arg>,
    /// [Workflow] settings.
    workflow_settings: workflow::Settings,
    /// Network settings.
    network_settings: settings::Dht,
}

impl<'a> WorkerBuilder<'a> {
    /// Create a new, default instance of a builder to generate a test [Worker].
    pub(crate) fn new(settings: settings::Node) -> Self {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            homestar_invocation::test_utils::related_wasm_instructions::<Arg>();
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

        let (evt_tx, _rx) = event::setup_event_channel(settings.clone());
        let (wk_tx, _rx) = event::setup_worker_channel(settings.clone());

        let workflow = Workflow::new(vec![task1, task2]);
        let workflow_cid = workflow.clone().to_cid().unwrap();

        #[cfg(feature = "ipfs")]
        let ipfs = IpfsCli::new(settings.network.ipfs()).unwrap();
        Self {
            #[cfg(feature = "ipfs")]
            ipfs: ipfs.clone(),
            db: MemoryDb::setup_connection_pool(&settings, None).unwrap(),
            event_sender: evt_tx,
            runner_sender: wk_tx,
            name: Some(workflow_cid.to_string()),
            workflow,
            workflow_settings: workflow::Settings::default(),
            network_settings: settings::Dht::default(),
        }
    }

    /// Build a [Worker] from the current state of the builder.
    #[allow(dead_code)]
    pub(crate) async fn build(self) -> Worker<'a, MemoryDb> {
        Worker::new(
            self.workflow,
            self.workflow_settings,
            self.network_settings,
            self.name,
            self.event_sender.into(),
            self.runner_sender,
            self.db,
        )
        .await
        .unwrap()
    }

    /// Fetch-function closure for the [Worker]/[Scheduler] to use.
    ///
    /// [Worker]: crate::Worker
    /// [Scheduler]: crate::TaskScheduler
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    #[allow(dead_code)]
    pub(crate) fn fetch_fn(
        &self,
    ) -> impl FnOnce(FnvHashSet<Resource>) -> BoxFuture<'a, anyhow::Result<IndexMap<Resource, Vec<u8>>>>
    {
        let fetch_settings = self.workflow_settings.clone().into();
        let ipfs = self.ipfs.clone();
        let fetch_fn = move |rscs: FnvHashSet<Resource>| {
            async move { Fetch::get_resources(rscs, fetch_settings, ipfs).await }.boxed()
        };

        fetch_fn
    }

    /// Fetch-function closure for the [Worker]/[Scheduler] to use.
    ///
    /// [Worker]: crate::Worker
    /// [Scheduler]: crate::TaskScheduler
    #[cfg(not(feature = "ipfs"))]
    #[allow(dead_code)]
    pub(crate) fn fetch_fn(
        &self,
    ) -> impl FnOnce(FnvHashSet<Resource>) -> BoxFuture<'a, anyhow::Result<IndexMap<Resource, Vec<u8>>>>
    {
        let fetch_settings = self.workflow_settings.clone().into();
        let fetch_fn = |rscs: FnvHashSet<Resource>| {
            async move { Fetch::get_resources(rscs, fetch_settings).await }.boxed()
        };

        fetch_fn
    }

    /// Get the [Cid] of the workflow from the builder state.
    #[allow(dead_code)]
    pub(crate) fn workflow_cid(&self) -> Cid {
        self.workflow.clone().to_cid().unwrap()
    }

    /// Get the length of the workflow from the builder state.
    #[allow(dead_code)]
    pub(crate) fn workflow_len(&self) -> u32 {
        self.workflow.len()
    }

    /// Get the in-memory [db] from the builder state.
    ///
    /// [db]: MemoryDb
    #[allow(dead_code)]
    pub(crate) fn db(&self) -> MemoryDb {
        self.db.clone()
    }

    /// Build a [Worker] with a specific [Workflow] from a set of tasks.
    ///
    /// [tasks]: Task
    #[allow(dead_code)]
    pub(crate) fn with_tasks(mut self, tasks: Vec<Task<'a, Arg>>) -> Self {
        self.workflow = Workflow::new(tasks);
        self
    }

    /// Build a [Worker] with a specific Event [AsyncChannelSender].
    #[allow(dead_code)]
    pub(crate) fn with_event_sender(mut self, event_sender: AsyncChannelSender<Event>) -> Self {
        self.event_sender = event_sender;
        self
    }

    /// Build a [Worker] with a specific [workflow::Settings].
    #[allow(dead_code)]
    pub(crate) fn with_workflow_settings(mut self, workflow_settings: workflow::Settings) -> Self {
        self.workflow_settings = workflow_settings;
        self
    }
}

impl Default for WorkerBuilder<'_> {
    fn default() -> Self {
        let settings = Settings::load().unwrap();
        Self::new(settings.node)
    }
}
