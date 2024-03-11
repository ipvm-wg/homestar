#![allow(missing_docs)]
use super::IndexedResources;
use crate::{
    channel::{AsyncChannel, AsyncChannelSender},
    db::{Connection, Database},
    event_handler::{
        event::QueryRecord,
        swarm_event::{FoundEvent, ResponseEvent},
        Event,
    },
    network::swarm::CapsuleTag,
    settings, Db, Receipt,
};
use anyhow::{anyhow, bail, Result};
use chrono::{NaiveDateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use faststr::FastStr;
use homestar_invocation::{ipld::DagJson, Pointer};
use libipld::{cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Cid, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, sync::Arc, time::Duration};
use tokio::{
    runtime::Handle,
    time::{timeout_at, Instant},
};
use tracing::info;

/// [Workflow] header tag, for sharing workflow information over libp2p.
///
/// [Workflow]: homestar_workflow::Workflow
pub const WORKFLOW_TAG: &str = "ipvm/workflow";

const CID_KEY: &str = "cid";
const NAME_KEY: &str = "name";
const NUM_TASKS_KEY: &str = "num_tasks";
const PROGRESS_KEY: &str = "progress";
const PROGRESS_COUNT_KEY: &str = "progress_count";
const RESOURCES_KEY: &str = "resources";

/// Status of a [Workflow].
///
/// [Workflow]: homestar_workflow::Workflow
#[derive(Debug, Clone, PartialEq, diesel_derive_enum::DbEnum)]
pub enum Status {
    /// Workflow is pending - default case.
    Pending,
    /// Workflow is currently running.
    Running,
    /// Workflow has been completed.
    Completed,
    /// Workflow is stuck, awaiting CIDs we can't find on the network.
    Stuck,
}

/// [Workflow] information stored in the database.
///
/// [Workflow]: homestar_workflow::Workflow
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, Identifiable, Selectable)]
#[diesel(table_name = crate::db::schema::workflows, primary_key(cid))]
pub struct Stored {
    /// Wrapped-Cid of [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) cid: Pointer,
    /// Local name of [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) name: Option<String>,
    /// Number of tasks in [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) num_tasks: i32,
    /// Map of [Instruction] Cids to resources.
    ///
    /// [Instruction]: homestar_invocation::task::Instruction
    pub(crate) resources: IndexedResources,
    /// Local timestamp of [Workflow] creation.
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) created_at: NaiveDateTime,
    /// Local timestamp of [Workflow] completion.
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) completed_at: Option<NaiveDateTime>,
    /// Status of [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) status: Status,
    /// Retries of [Workflow] when checking for provider.
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) retries: i32,
}

impl Stored {
    /// Create a new [Stored] workflow for the [db].
    ///
    /// [db]: Database
    pub fn new(
        cid: Pointer,
        name: Option<String>,
        num_tasks: i32,
        resources: IndexedResources,
        created_at: NaiveDateTime,
    ) -> Self {
        Self {
            cid,
            name,
            num_tasks,
            resources,
            created_at,
            completed_at: None,
            status: Status::Pending,
            retries: 0,
        }
    }

    /// Create a new [Stored] workflow for the [db] with a default timestamp.
    ///
    /// [db]: Database
    pub fn new_with_resources(
        cid: Pointer,
        name: Option<String>,
        num_tasks: i32,
        resources: IndexedResources,
    ) -> Self {
        Self {
            cid,
            name,
            num_tasks,
            resources,
            created_at: Utc::now().naive_utc(),
            completed_at: None,
            status: Status::Pending,
            retries: 0,
        }
    }

    /// Create a default [Stored] workflow for the [db].
    ///
    /// [db]: Database
    pub fn default(cid: Pointer, num_tasks: i32) -> Self {
        let name = cid.to_string();
        Self {
            cid,
            name: Some(name),
            num_tasks,
            resources: IndexedResources::default(),
            created_at: Utc::now().naive_utc(),
            completed_at: None,
            status: Status::Pending,
            retries: 0,
        }
    }
}

/// [Workflow] information stored in the database, tied to [receipts].
///
/// [Workflow]: homestar_workflow::Workflow
/// [receipts]: crate::Receipt
#[derive(
    Debug, Clone, PartialEq, Queryable, Insertable, Identifiable, Selectable, Associations, Hash,
)]
#[diesel(belongs_to(Receipt, foreign_key = receipt_cid))]
#[diesel(belongs_to(Stored, foreign_key = workflow_cid))]
#[diesel(table_name = crate::db::schema::workflows_receipts, primary_key(workflow_cid, receipt_cid))]
pub(crate) struct StoredReceipt {
    pub(crate) workflow_cid: Pointer,
    pub(crate) receipt_cid: Pointer,
}

impl StoredReceipt {
    pub(crate) fn new(workflow_cid: Pointer, receipt_cid: Pointer) -> Self {
        Self {
            workflow_cid,
            receipt_cid,
        }
    }
}

/// Associated [Workflow] information, separated from [Workflow] struct in order
/// to relate to it as a key-value relationship of (workflow)
/// cid => [Info].
///
/// [Workflow]: homestar_workflow::Workflow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Info {
    pub(crate) cid: Cid,
    pub(crate) name: Option<FastStr>,
    pub(crate) num_tasks: u32,
    pub(crate) progress: Vec<Cid>,
    pub(crate) progress_count: u32,
    pub(crate) resources: IndexedResources,
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cid: {}, local_name: {}, name, progress: {}/{}",
            self.cid,
            self.name.clone().unwrap_or(self.cid.to_string().into()),
            self.progress_count,
            self.num_tasks
        )
    }
}

impl Info {
    /// Create a workflow information structure from a [Stored] workflow and
    /// `progress` vector.
    pub fn new(stored: Stored, progress: Vec<Cid>) -> Self {
        let progress_count = progress.len() as u32;
        let cid = stored.cid.cid();
        Self {
            cid,
            name: stored.name.map(|name| name.into()),
            num_tasks: stored.num_tasks as u32,
            progress,
            progress_count,
            resources: stored.resources,
        }
    }

    /// Create a default workflow [Info] given a Cid and number of tasks.
    pub fn default(stored: Stored) -> Self {
        let cid = stored.cid.cid();
        Self {
            cid,
            name: stored.name.map(|name| name.into()),
            num_tasks: stored.num_tasks as u32,
            progress: vec![],
            progress_count: 0,
            resources: stored.resources,
        }
    }

    /// Get workflow progress as a vector of Cids.
    pub fn progress(&self) -> &Vec<Cid> {
        &self.progress
    }

    /// Get workflow progress as a number of receipts completed.
    pub fn progress_count(&self) -> u32 {
        self.progress_count
    }

    /// Get the number of tasks in the [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub fn num_tasks(&self) -> u32 {
        self.num_tasks
    }

    /// Get unique identifier, Cid, of [Workflow].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) fn cid(&self) -> Cid {
        self.cid
    }

    /// Get the Cid of a [Workflow] as a [String].
    ///
    /// [Workflow]: homestar_workflow::Workflow
    #[allow(dead_code)]
    pub(crate) fn cid_as_string(&self) -> String {
        self.cid.to_string()
    }

    /// Get the Cid of a [Workflow] as bytes.
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) fn cid_as_bytes(&self) -> Vec<u8> {
        self.cid().to_bytes()
    }

    /// Set map of [Instruction] Cids to resources.
    ///
    /// [Instruction]: homestar_invocation::task::Instruction
    #[allow(dead_code)]
    pub(crate) fn set_resources(&mut self, resources: IndexedResources) {
        self.resources = resources;
    }

    /// Set the progress / step of the [Workflow] completed, which
    /// may not be the same as the `progress` vector of Cids.
    ///
    /// [Workflow]: homestar_workflow::Workflow
    #[allow(dead_code)]
    pub(crate) fn set_progress_count(&mut self, progress_count: u32) {
        self.progress_count = progress_count;
    }

    /// Set the progress / step of the [Info].
    pub(crate) fn increment_progress(&mut self, new_cid: Cid) {
        self.progress.push(new_cid);
        self.progress_count = self.progress.len() as u32 + 1;
    }

    /// Capsule-wrapper for [Info] to to be shared over libp2p as
    /// DagCbor encoded bytes.
    pub(crate) fn capsule(&self) -> anyhow::Result<Vec<u8>> {
        let info_ipld = Ipld::from(self.to_owned());
        let capsule = if let Ipld::Map(map) = info_ipld {
            Ok(Ipld::Map(BTreeMap::from([(
                WORKFLOW_TAG.into(),
                Ipld::Map(map),
            )])))
        } else {
            Err(anyhow!("workflow info to Ipld conversion is not a map"))
        }?;

        DagCborCodec.encode(&capsule)
    }

    /// Retrieve available [Info] from the database or libp2p given a
    /// [Workflow], or return a default/new version of [Info] if none is found.
    ///
    /// [Workflow]: homestar_workflow::Workflow
    pub(crate) async fn init(
        workflow_cid: Cid,
        workflow_len: u32,
        name: FastStr,
        resources: IndexedResources,
        network_settings: settings::Dht,
        event_sender: Arc<AsyncChannelSender<Event>>,
        mut conn: Connection,
    ) -> Result<(Self, NaiveDateTime)> {
        let timestamp = Utc::now().naive_utc();
        match Db::get_workflow_info(workflow_cid, &mut conn) {
            Ok((Some(stored_name), info)) if stored_name != name => {
                Db::update_local_name(&name, &mut conn)?;
                Ok((info, timestamp))
            }
            Ok((_, info)) => Ok((info, timestamp)),
            Err(_err) => {
                info!(
                    subject = "workflow.init.db.check",
                    category = "workflow",
                    cid = workflow_cid.to_string(),
                    "workflow information not available in the database"
                );

                let stored = Stored::new(
                    Pointer::new(workflow_cid),
                    Some(name.to_string()),
                    workflow_len as i32,
                    resources,
                    timestamp,
                );

                let result = Db::store_workflow(stored.clone(), &mut conn)?;
                let workflow_info = Self::default(result);

                // spawn a separate task to retrieve workflow info from the
                // network and store it in the database if it finds it.
                let handle = Handle::current();
                handle.spawn(async move {
                    match Self::retrieve_from_dht(
                        workflow_cid,
                        event_sender.clone(),
                        network_settings.p2p_workflow_info_timeout,
                    )
                    .await
                    {
                        Ok(workflow_info) => Ok(workflow_info),
                        Err(_) => {
                            Self::retrieve_from_provider(
                                workflow_cid,
                                event_sender,
                                network_settings.p2p_provider_timeout,
                            )
                            .await
                        }
                    }
                });

                Ok((workflow_info, timestamp))
            }
        }
    }

    /// Retrieve available [Info] from the database or libp2p given a
    /// workflow Cid.
    pub(crate) async fn retrieve<'a>(
        workflow_cid: Cid,
        #[allow(unused)] event_sender: Arc<AsyncChannelSender<Event>>,
        mut conn: Option<Connection>,
        #[allow(unused)] p2p_provider_timeout: Duration,
    ) -> Result<Self> {
        let workflow_info = match conn
            .as_mut()
            .and_then(|conn| Db::get_workflow_info(workflow_cid, conn).ok())
        {
            Some((_name, workflow_info)) => Ok(workflow_info),
            None => {
                info!(
                    subject = "workflow.retrieve.db.check",
                    category = "workflow",
                    cid = workflow_cid.to_string(),
                    "workflow information not available in the database"
                );

                Self::retrieve_from_provider(workflow_cid, event_sender, p2p_provider_timeout).await
            }
        }?;

        Ok(workflow_info)
    }

    // Retrieve [Info] from the DHT and send a [FoundEvent::Workflow] event
    // if info is found.
    async fn retrieve_from_dht<'a>(
        workflow_cid: Cid,
        event_sender: Arc<AsyncChannelSender<Event>>,
        p2p_workflow_info_timeout: Duration,
    ) -> Result<Info> {
        let (tx, rx) = AsyncChannel::oneshot();
        event_sender
            .send_async(Event::FindRecord(QueryRecord::with(
                workflow_cid,
                CapsuleTag::Workflow,
                Some(tx),
            )))
            .await?;

        match timeout_at(Instant::now() + p2p_workflow_info_timeout, rx.recv_async()).await {
            Ok(Ok(ResponseEvent::Found(Ok(FoundEvent::Workflow(event))))) => {
                #[cfg(feature = "websocket-notify")]
                let _ = event_sender
                    .send_async(Event::StoredRecord(FoundEvent::Workflow(event.clone())))
                    .await;

                Ok(event.workflow_info)
            }
            Ok(Ok(ResponseEvent::Found(Err(_err)))) => {
                bail!("failed to find workflow info with cid {workflow_cid}")
            }
            Ok(Ok(event)) => {
                bail!("received unexpected event {event:?} for workflow {workflow_cid}")
            }
            Ok(Err(err)) => {
                bail!("unexpected error while retrieving workflow info: {err}")
            }
            Err(_) => {
                bail!(
                    "timeout deadline reached while finding workflow info with cid {workflow_cid}"
                )
            }
        }
    }

    // Retrieve [Info] from a provider and send a [FoundEvent::Workflow] event
    // if info is found.
    async fn retrieve_from_provider<'a>(
        workflow_cid: Cid,
        event_sender: Arc<AsyncChannelSender<Event>>,
        p2p_provider_timeout: Duration,
    ) -> Result<Info> {
        let (tx, rx) = AsyncChannel::oneshot();
        event_sender
            .send_async(Event::GetProviders(QueryRecord::with(
                workflow_cid,
                CapsuleTag::Workflow,
                Some(tx),
            )))
            .await?;

        match timeout_at(Instant::now() + p2p_provider_timeout, rx.recv_async()).await {
            Ok(Ok(ResponseEvent::Found(Ok(FoundEvent::Workflow(event))))) => {
                #[cfg(feature = "websocket-notify")]
                let _ = event_sender
                    .send_async(Event::StoredRecord(FoundEvent::Workflow(event.clone())))
                    .await;

                Ok(event.workflow_info)
            }
            Ok(Ok(ResponseEvent::Found(Err(err)))) => {
                bail!("failure in attempting to find event: {err}")
            }
            Ok(Ok(event)) => {
                bail!("received unexpected event {event:?} for workflow {workflow_cid}")
            }
            Ok(Err(err)) => {
                bail!("unexpected error while retrieving workflow info: {err}")
            }
            Err(_) => {
                bail!(
                    "timeout deadline reached while finding workflow info with cid {workflow_cid}"
                )
            }
        }
    }
}

impl From<Info> for Ipld {
    fn from(workflow: Info) -> Self {
        Ipld::Map(BTreeMap::from([
            (CID_KEY.into(), Ipld::Link(workflow.cid)),
            (
                NAME_KEY.into(),
                workflow
                    .name
                    .as_ref()
                    .map(|name| name.to_string().into())
                    .unwrap_or(Ipld::Null),
            ),
            (
                NUM_TASKS_KEY.into(),
                Ipld::Integer(workflow.num_tasks as i128),
            ),
            (
                PROGRESS_KEY.into(),
                Ipld::List(workflow.progress.into_iter().map(Ipld::Link).collect()),
            ),
            (
                PROGRESS_COUNT_KEY.into(),
                Ipld::Integer(workflow.progress_count as i128),
            ),
            (RESOURCES_KEY.into(), Ipld::from(workflow.resources)),
        ]))
    }
}

impl TryFrom<Ipld> for Info {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("no `cid` set"))?
                .to_owned(),
        )?;
        let name = map
            .get(NAME_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok());
        let num_tasks = from_ipld(
            map.get(NUM_TASKS_KEY)
                .ok_or_else(|| anyhow!("no `num_tasks` set"))?
                .to_owned(),
        )?;
        let progress = from_ipld(
            map.get(PROGRESS_KEY)
                .ok_or_else(|| anyhow!("no `progress` set"))?
                .to_owned(),
        )?;
        let progress_count = from_ipld(
            map.get(PROGRESS_COUNT_KEY)
                .ok_or_else(|| anyhow!("no `progress_count` set"))?
                .to_owned(),
        )?;
        let resources = IndexedResources::try_from(
            map.get(RESOURCES_KEY)
                .ok_or_else(|| anyhow!("no `resources` set"))?
                .to_owned(),
        )?;

        Ok(Self {
            cid,
            name,
            num_tasks,
            progress,
            progress_count,
            resources,
        })
    }
}

impl TryFrom<Info> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(workflow_info: Info) -> Result<Self, Self::Error> {
        let info_ipld = Ipld::from(workflow_info);
        DagCborCodec.encode(&info_ipld)
    }
}

impl TryFrom<Vec<u8>> for Info {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let ipld: Ipld = DagCborCodec.decode(&bytes)?;
        ipld.try_into()
    }
}

impl DagJson for Info where Ipld: From<Info> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::workflow::Resource;
    use homestar_invocation::{
        authority::UcanPrf,
        ipld::DagCbor,
        task::{instruction::RunInstruction, Resources},
        test_utils, Task,
    };
    use homestar_wasm::io::Arg;
    use homestar_workflow::Workflow;
    use indexmap::IndexMap;

    #[test]
    fn ipld_roundtrip_workflow_info() {
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

        let mut index_map = IndexMap::new();
        index_map.insert(
            instruction1.clone().to_cid().unwrap(),
            vec![Resource::Url(instruction1.resource().to_owned())],
        );
        index_map.insert(
            instruction2.clone().to_cid().unwrap(),
            vec![Resource::Url(instruction2.resource().to_owned())],
        );

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let stored_info = Stored::new_with_resources(
            Pointer::new(workflow.clone().to_cid().unwrap()),
            None,
            workflow.len() as i32,
            IndexedResources::new(index_map),
        );

        let mut workflow_info = Info::default(stored_info);
        workflow_info.increment_progress(task1.to_cid().unwrap());
        workflow_info.increment_progress(task2.to_cid().unwrap());
        let ipld = Ipld::from(workflow_info.clone());
        assert_eq!(workflow_info, ipld.try_into().unwrap());
    }
}
