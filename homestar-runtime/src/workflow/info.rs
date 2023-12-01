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
    Db, Receipt,
};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{NaiveDateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use faststr::FastStr;
use homestar_core::{ipld::DagJson, workflow::Pointer};
use libipld::{cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Cid, Ipld};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use tracing::info;

/// [Workflow] header tag, for sharing workflow information over libp2p.
///
/// [Workflow]: homestar_core::Workflow
pub const WORKFLOW_TAG: &str = "ipvm/workflow";

const CID_KEY: &str = "cid";
const NAME_KEY: &str = "name";
const NUM_TASKS_KEY: &str = "num_tasks";
const PROGRESS_KEY: &str = "progress";
const PROGRESS_COUNT_KEY: &str = "progress_count";
const RESOURCES_KEY: &str = "resources";

/// [Workflow] information stored in the database.
///
/// [Workflow]: homestar_core::Workflow
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, Identifiable, Selectable)]
#[diesel(table_name = crate::db::schema::workflows, primary_key(cid))]
pub struct Stored {
    /// Wrapped-[Cid] of [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) cid: Pointer,
    /// Local name of [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) name: Option<String>,
    /// Number of tasks in [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) num_tasks: i32,
    /// Map of [Instruction] [Cid]s to resources.
    ///
    /// [Instruction]: homestar_core::workflow::Instruction
    pub(crate) resources: IndexedResources,
    /// Local timestamp of [Workflow] creation.
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) created_at: NaiveDateTime,
    /// Local timestamp of [Workflow] completion.
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) completed_at: Option<NaiveDateTime>,
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
        }
    }
}

/// [Workflow] information stored in the database, tied to [receipts].
///
/// [Workflow]: homestar_core::Workflow
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
/// [Workflow]: homestar_core::Workflow
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

    /// Create a default workflow [Info] given a [Cid] and number of tasks.
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

    /// Get unique identifier, [Cid], of [Workflow].
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) fn cid(&self) -> Cid {
        self.cid
    }

    /// Get the [Cid] of a [Workflow] as a [String].
    ///
    /// [Workflow]: homestar_core::Workflow
    #[allow(dead_code)]
    pub(crate) fn cid_as_string(&self) -> String {
        self.cid.to_string()
    }

    /// Get the [Cid] of a [Workflow] as bytes.
    ///
    /// [Workflow]: homestar_core::Workflow
    pub(crate) fn cid_as_bytes(&self) -> Vec<u8> {
        self.cid().to_bytes()
    }

    /// Set map of [Instruction] [Cid]s to resources.
    ///
    /// [Instruction]: homestar_core::workflow::Instruction
    #[allow(dead_code)]
    pub(crate) fn set_resources(&mut self, resources: IndexedResources) {
        self.resources = resources;
    }

    /// Set the progress / step of the [Workflow] completed, which
    /// may not be the same as the `progress` vector of [Cid]s.
    ///
    /// [Workflow]: homestar_core::Workflow
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
    /// [DagCbor] encoded bytes.
    ///
    /// [DagCbor]: DagCborCodec
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

    /// [Gather] available [Info] from the database or [libp2p] given a
    /// [Workflow], or return a default/new version of [Info] if none is found.
    ///
    /// [Gather]: Self::gather
    /// [Workflow]: homestar_core::Workflow
    pub(crate) async fn init(
        workflow_cid: Cid,
        workflow_len: u32,
        name: FastStr,
        resources: IndexedResources,
        p2p_timeout: Duration,
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
                    Some(name.into_string()),
                    workflow_len as i32,
                    resources,
                    timestamp,
                );

                let result = Db::store_workflow(stored.clone(), &mut conn)?;
                let workflow_info = Self::default(result);

                // spawn a task to retrieve the workflow info from the
                // network and store it in the database if it finds it.
                let handle = Handle::current();
                handle.spawn(Self::retrieve_from_query(
                    workflow_cid,
                    p2p_timeout,
                    event_sender,
                    Some(conn),
                    None::<fn(Cid, Option<Connection>) -> Result<Self>>,
                ));

                Ok((workflow_info, timestamp))
            }
        }
    }

    /// Gather available [Info] from the database or [libp2p] given a
    /// workflow [Cid].
    pub(crate) async fn gather<'a>(
        workflow_cid: Cid,
        p2p_timeout: Duration,
        event_sender: Arc<AsyncChannelSender<Event>>,
        mut conn: Option<Connection>,
        handle_timeout_fn: Option<impl FnOnce(Cid, Option<Connection>) -> Result<Self>>,
    ) -> Result<Self> {
        let workflow_info = match conn
            .as_mut()
            .and_then(|conn| Db::get_workflow_info(workflow_cid, conn).ok())
        {
            Some((_name, workflow_info)) => Ok(workflow_info),
            None => {
                info!(
                    subject = "workflow.gather.db.check",
                    category = "workflow",
                    cid = workflow_cid.to_string(),
                    "workflow information not available in the database"
                );

                Self::retrieve_from_query(
                    workflow_cid,
                    p2p_timeout,
                    event_sender,
                    conn,
                    handle_timeout_fn,
                )
                .await
            }
        }?;

        Ok(workflow_info)
    }

    async fn retrieve_from_query<'a>(
        workflow_cid: Cid,
        p2p_timeout: Duration,
        event_sender: Arc<AsyncChannelSender<Event>>,
        conn: Option<Connection>,
        handle_timeout_fn: Option<impl FnOnce(Cid, Option<Connection>) -> Result<Info>>,
    ) -> Result<Info> {
        let (tx, rx) = AsyncChannel::oneshot();
        event_sender
            .send_async(Event::FindRecord(QueryRecord::with(
                workflow_cid,
                CapsuleTag::Workflow,
                Some(tx),
            )))
            .await?;

        match rx.recv_deadline(Instant::now() + p2p_timeout) {
            Ok(ResponseEvent::Found(Ok(FoundEvent::Workflow(workflow_info)))) => {
                // store workflow receipts from info, as we've already stored
                // the static information.
                if let Some(mut conn) = conn {
                    Db::store_workflow_receipts(workflow_cid, &workflow_info.progress, &mut conn)?;
                }

                Ok(workflow_info)
            }
            Ok(ResponseEvent::Found(Err(err))) => {
                bail!("failure in attempting to find event: {err}")
            }
            Ok(event) => {
                bail!("received unexpected event {event:?} for workflow {workflow_cid}")
            }
            Err(err) => handle_timeout_fn
                .map(|f| f(workflow_cid, conn).context(err))
                .unwrap_or(Err(anyhow!(
                    "timeout deadline reached for retrieving workflow info"
                ))),
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
    use homestar_core::{
        ipld::DagCbor,
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
        Workflow,
    };
    use homestar_wasm::io::Arg;

    #[test]
    fn ipld_roundtrip_workflow_info() {
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
        let stored_info = Stored::default(
            Pointer::new(workflow.clone().to_cid().unwrap()),
            workflow.len() as i32,
        );
        let mut workflow_info = Info::default(stored_info);
        workflow_info.increment_progress(task1.to_cid().unwrap());
        workflow_info.increment_progress(task2.to_cid().unwrap());
        let ipld = Ipld::from(workflow_info.clone());
        assert_eq!(workflow_info, ipld.try_into().unwrap());
    }
}
