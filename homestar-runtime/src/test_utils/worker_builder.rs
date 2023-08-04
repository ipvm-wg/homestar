//! Module for building out [Worker]s for testing purposes.

use super::{db::MemoryDb, event};
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::Database, event_handler::Event, settings, worker::WorkerMessage, workflow, Settings, Worker,
};
use homestar_core::{
    ipld::DagCbor,
    test_utils::workflow as workflow_test_utils,
    workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
    Workflow,
};
use homestar_wasm::io::Arg;
use libipld::Cid;
use std::sync::Arc;
use tokio::sync::mpsc;

#[cfg(feature = "ipfs")]
pub(crate) struct WorkerBuilder<'a> {
    db: MemoryDb,
    event_sender: Arc<mpsc::Sender<Event>>,
    runner_sender: mpsc::Sender<WorkerMessage>,
    ipfs: IpfsCli,
    workflow: Workflow<'a, Arg>,
    workflow_settings: workflow::Settings,
}

#[cfg(not(feature = "ipfs"))]
pub(crate) struct WorkerBuilder<'a> {
    db: MemoryDb,
    event_sender: Arc<mpsc::Sender<Event>>,
    runner_sender: mpsc::Sender<WorkerMessage>,
    workflow: Workflow<'a, Arg>,
    workflow_settings: workflow::Settings,
}

impl<'a> WorkerBuilder<'a> {
    /// Create a new, default instance of a builder to generate a test [Worker].
    #[cfg(feature = "ipfs")]
    pub(crate) fn new(settings: settings::Node) -> Self {
        let ipfs = IpfsCli::default();

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

        let (evt_tx, _rx) = event::setup_event_channel(settings.clone());
        let (wk_tx, _rx) = event::setup_worker_channel(settings.clone());
        Self {
            db: MemoryDb::setup_connection_pool(&settings).unwrap(),
            event_sender: evt_tx.into(),
            runner_sender: wk_tx,
            ipfs,
            workflow: Workflow::new(vec![task1, task2]),
            workflow_settings: workflow::Settings::default(),
        }
    }

    /// Create a new, default instance of a builder to generate a test [Worker].
    #[cfg(not(feature = "ipfs"))]
    pub(crate) fn new(settings: settings::Node) -> Self {
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

        let (evt_tx, _rx) = event::setup_event_channel(settings.clone());
        let (wk_tx, _rx) = event::setup_worker_channel(settings.clone());
        Self {
            db: MemoryDb::setup_connection_pool(&settings).unwrap(),
            event_sender: evt_tx.into(),
            runner_sender: wk_tx,
            workflow: Workflow::new(vec![task1, task2]),
            workflow_settings: workflow::Settings::default(),
        }
    }

    /// Build a [Worker] from the current state of the builder.
    #[cfg(feature = "ipfs")]
    #[allow(dead_code)]
    pub(crate) async fn build(self) -> Worker<'a, MemoryDb> {
        Worker::new(
            self.workflow,
            self.workflow_settings,
            self.event_sender,
            self.runner_sender,
            self.db,
            self.ipfs,
        )
        .await
        .unwrap()
    }

    /// Build a [Worker] from the current state of the builder.
    #[cfg(not(feature = "ipfs"))]
    #[allow(dead_code)]
    pub(crate) async fn build(self) -> Worker<'a, MemoryDb> {
        Worker::new(
            self.workflow,
            self.workflow_settings,
            self.event_sender,
            self.runner_sender,
            self.db,
        )
        .await
        .unwrap()
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

    /// Build a [Worker] with a specific Event [mpsc::Sender].
    #[allow(dead_code)]
    pub(crate) fn with_event_sender(mut self, event_sender: Arc<mpsc::Sender<Event>>) -> Self {
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
