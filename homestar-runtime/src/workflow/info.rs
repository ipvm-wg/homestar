use crate::{
    db::{Connection, Database},
    event_handler::{
        channel::BoundedChannel,
        event::QueryRecord,
        swarm_event::{FoundEvent, ResponseEvent},
        Event,
    },
    network::swarm::CapsuleTag,
    Db, Receipt,
};
use anyhow::{anyhow, bail, Context, Result};
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};
use homestar_core::{ipld::DagCbor, workflow::Pointer, Workflow};
use homestar_wasm::io::Arg;
use libipld::{cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Cid, Ipld};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tracing::info;

/// [Workflow] header tag, for sharing workflow information over libp2p.
///
/// [Workflow]: Workflow
pub const WORKFLOW_TAG: &str = "ipvm/workflow";

const CID_KEY: &str = "cid";
const PROGRESS_KEY: &str = "progress";
const PROGRESS_COUNT_KEY: &str = "progress_count";
const NUM_TASKS_KEY: &str = "num_tasks";

/// [Workflow] information stored in the database.
///
/// [Workflow]: homestar_core::Workflow
#[derive(Debug, Clone, PartialEq, Queryable, Insertable, Identifiable, Selectable, Hash)]
#[diesel(table_name = crate::db::schema::workflows, primary_key(cid))]
pub struct Stored {
    pub(crate) cid: Pointer,
    pub(crate) num_tasks: i32,
}

impl Stored {
    pub fn new(cid: Pointer, num_tasks: i32) -> Self {
        Self { cid, num_tasks }
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
#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub(crate) cid: Cid,
    pub(crate) progress: Vec<Cid>,
    pub(crate) progress_count: u32,
    pub(crate) num_tasks: u32,
}

impl Info {
    /// Create a new [Info] given a [Cid], progress / step, and number
    /// of tasks.
    pub fn new(cid: Cid, progress: Vec<Cid>, num_tasks: u32) -> Self {
        let progress_count = progress.len() as u32;
        Self {
            cid,
            progress,
            progress_count,
            num_tasks,
        }
    }

    /// Create a default [Info] given a [Cid] and number of tasks.
    pub fn default(cid: Cid, num_tasks: u32) -> Self {
        Self {
            cid,
            progress: vec![],
            progress_count: 0,
            num_tasks,
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

    /// Set the progress / step of the [Workflow] completed, which
    /// may not be the same as the `progress` vector of [Cid]s.
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
    #[allow(dead_code)]
    pub(crate) async fn init<'a>(
        workflow: Workflow<'a, Arg>,
        p2p_timeout: Duration,
        event_sender: Arc<mpsc::Sender<Event>>,
        conn: &'a mut Connection,
    ) -> Result<Self> {
        let workflow_len = workflow.len();
        let workflow_cid = workflow.to_cid()?;

        let handle_timeout_fn = |workflow_cid, reused_conn: Option<&'a mut Connection>| {
            let workflow_info = Self::default(workflow_cid, workflow_len);
            // store workflow from info

            match reused_conn.and_then(|conn| {
                Db::store_workflow(
                    Stored::new(
                        Pointer::new(workflow_info.cid),
                        workflow_info.num_tasks as i32,
                    ),
                    conn,
                )
                .ok()
            }) {
                Some(_) => Ok(workflow_info),
                None => bail!("failed to store workflow"),
            }
        };

        Self::gather(
            workflow_cid,
            p2p_timeout,
            event_sender,
            Some(conn),
            handle_timeout_fn,
        )
        .await
    }

    /// Gather available [Info] from the database or [libp2p] given a
    /// workflow [Cid].
    pub(crate) async fn gather<'a>(
        workflow_cid: Cid,
        p2p_timeout: Duration,
        event_sender: Arc<mpsc::Sender<Event>>,
        mut conn: Option<&'a mut Connection>,
        handle_timeout_fn: impl FnOnce(Cid, Option<&'a mut Connection>) -> Result<Info>,
    ) -> Result<Self> {
        async fn retrieve_from_query<'a>(
            workflow_cid: Cid,
            p2p_timeout: Duration,
            event_sender: Arc<mpsc::Sender<Event>>,
            conn: Option<&'a mut Connection>,
            handle_timeout_fn: impl FnOnce(Cid, Option<&'a mut Connection>) -> Result<Info>,
        ) -> Result<Info> {
            let channel = BoundedChannel::oneshot();
            event_sender.try_send(Event::FindRecord(QueryRecord::with(
                workflow_cid,
                CapsuleTag::Workflow,
                channel.tx,
            )))?;

            match channel.rx.recv_deadline(Instant::now() + p2p_timeout) {
                Ok(ResponseEvent::Found(Ok(FoundEvent::Workflow(workflow_info)))) => {
                    // store workflow from info
                    if let Some(conn) = conn {
                        Db::store_workflow(
                            Stored::new(
                                Pointer::new(workflow_info.cid),
                                workflow_info.num_tasks as i32,
                            ),
                            conn,
                        )?;

                        Db::store_workflow_receipts(workflow_cid, &workflow_info.progress, conn)?;
                    }

                    Ok(workflow_info)
                }
                Ok(ResponseEvent::Found(Err(err))) => {
                    bail!("failure in attempting to find event: {err}")
                }
                Ok(event) => {
                    bail!("received unexpected event {event:?} for workflow {workflow_cid}")
                }
                Err(err) => handle_timeout_fn(workflow_cid, conn).context(err),
            }
        }

        let workflow_info = match conn
            .as_mut()
            .and_then(|conn| Db::get_workflow_info(workflow_cid, conn).ok())
        {
            Some(workflow_info) => Ok(workflow_info),
            None => {
                info!(
                    cid = workflow_cid.to_string(),
                    "workflow information not available in the database"
                );

                retrieve_from_query(
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
}

impl From<Info> for Ipld {
    fn from(workflow: Info) -> Self {
        Ipld::Map(BTreeMap::from([
            (CID_KEY.into(), Ipld::Link(workflow.cid)),
            (
                PROGRESS_KEY.into(),
                Ipld::List(workflow.progress.into_iter().map(Ipld::Link).collect()),
            ),
            (
                PROGRESS_COUNT_KEY.into(),
                Ipld::Integer(workflow.progress_count as i128),
            ),
            (
                NUM_TASKS_KEY.into(),
                Ipld::Integer(workflow.num_tasks as i128),
            ),
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
        let num_tasks = from_ipld(
            map.get(NUM_TASKS_KEY)
                .ok_or_else(|| anyhow!("no `num_tasks` set"))?
                .to_owned(),
        )?;

        Ok(Self {
            cid,
            progress,
            progress_count,
            num_tasks,
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
        let mut workflow_info = Info::default(workflow.clone().to_cid().unwrap(), workflow.len());
        workflow_info.increment_progress(task1.to_cid().unwrap());
        workflow_info.increment_progress(task2.to_cid().unwrap());
        let ipld = Ipld::from(workflow_info.clone());
        assert_eq!(workflow_info, ipld.try_into().unwrap());
    }
}
