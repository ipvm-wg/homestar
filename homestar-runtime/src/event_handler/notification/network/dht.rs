//! Notification types for [swarm] DHT events.
//!
//! [swarm]: libp2p::swarm::Swarm

use anyhow::anyhow;
use chrono::prelude::Utc;
use derive_getters::Getters;
use faststr::FastStr;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Cid, Ipld};
use libp2p::PeerId;
use schemars::JsonSchema;
use std::collections::BTreeMap;

const CID_KEY: &str = "cid";
const CONNECTED_PEER_COUNT_KEY: &str = "connected_peer_count";
const NAME_KEY: &str = "name";
const NUM_TASKS_KEY: &str = "num_tasks";
const PROGRESS_KEY: &str = "progress";
const PROGRESS_COUNT_KEY: &str = "progress_count";
const PUBLISHER_KEY: &str = "publisher";
const QUORUM_KEY: &str = "quorum";
const RAN_KEY: &str = "ran";
const STORED_TO_PEERS_KEY: &str = "stored_to_peers";
const TIMESTAMP_KEY: &str = "timestamp";

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "put_receipt_dht")]
pub struct PutReceiptDht {
    timestamp: i64,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Ran receipt CID")]
    ran: String,
}

impl PutReceiptDht {
    pub(crate) fn new(cid: Cid, ran: String) -> PutReceiptDht {
        PutReceiptDht {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            ran,
        }
    }
}

impl DagJson for PutReceiptDht {}

impl From<PutReceiptDht> for Ipld {
    fn from(notification: PutReceiptDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (RAN_KEY.into(), notification.ran.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for PutReceiptDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let ran = from_ipld(
            map.get(RAN_KEY)
                .ok_or_else(|| anyhow!("missing {RAN_KEY}"))?
                .to_owned(),
        )?;

        Ok(PutReceiptDht {
            timestamp,
            cid,
            ran,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "got_receipt_dht")]
pub struct GotReceiptDht {
    timestamp: i64,
    #[schemars(description = "Receipt publisher peer ID")]
    publisher: Option<String>,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Ran receipt CID")]
    ran: String,
}

impl GotReceiptDht {
    pub(crate) fn new(publisher: Option<PeerId>, cid: Cid, ran: String) -> GotReceiptDht {
        GotReceiptDht {
            timestamp: Utc::now().timestamp_millis(),
            publisher: publisher.map(|p| p.to_string()),
            cid: cid.to_string(),
            ran,
        }
    }
}

impl DagJson for GotReceiptDht {}

impl From<GotReceiptDht> for Ipld {
    fn from(notification: GotReceiptDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (
                PUBLISHER_KEY.into(),
                notification
                    .publisher
                    .map(|peer_id| peer_id.into())
                    .unwrap_or(Ipld::Null),
            ),
            (CID_KEY.into(), notification.cid.into()),
            (RAN_KEY.into(), notification.ran.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for GotReceiptDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let publisher = map
            .get(PUBLISHER_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok());

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let ran = from_ipld(
            map.get(RAN_KEY)
                .ok_or_else(|| anyhow!("missing {RAN_KEY}"))?
                .to_owned(),
        )?;

        Ok(GotReceiptDht {
            timestamp,
            publisher,
            cid,
            ran,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "put_workflow_info_dht")]
pub struct PutWorkflowInfoDht {
    timestamp: i64,
    #[schemars(description = "Workflow info CID")]
    cid: String,
    #[schemars(description = "Optional workflow name")]
    name: Option<String>,
    #[schemars(description = "Number of tasks in workflow")]
    num_tasks: u32,
    #[schemars(description = "Completed task CIDs")]
    progress: Vec<String>,
    #[schemars(description = "Number of workflow tasks completed")]
    progress_count: u32,
}

impl PutWorkflowInfoDht {
    pub(crate) fn new(
        cid: Cid,
        name: Option<FastStr>,
        num_tasks: u32,
        progress: Vec<Cid>,
        progress_count: u32,
    ) -> PutWorkflowInfoDht {
        PutWorkflowInfoDht {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            name: name.map(|n| n.into()),
            num_tasks,
            progress: progress.iter().map(|cid| cid.to_string()).collect(),
            progress_count,
        }
    }
}

impl DagJson for PutWorkflowInfoDht {}

impl From<PutWorkflowInfoDht> for Ipld {
    fn from(notification: PutWorkflowInfoDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (
                NAME_KEY.into(),
                notification
                    .name
                    .map(|peer_id| peer_id.into())
                    .unwrap_or(Ipld::Null),
            ),
            (NUM_TASKS_KEY.into(), notification.num_tasks.into()),
            (
                PROGRESS_KEY.into(),
                Ipld::List(
                    notification
                        .progress
                        .iter()
                        .map(|cid| Ipld::String(cid.to_string()))
                        .collect(),
                ),
            ),
            (
                PROGRESS_COUNT_KEY.into(),
                notification.progress_count.into(),
            ),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for PutWorkflowInfoDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
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
                .ok_or_else(|| anyhow!("missing {NUM_TASKS_KEY}"))?
                .to_owned(),
        )?;

        let progress = from_ipld::<Vec<String>>(
            map.get(PROGRESS_KEY)
                .ok_or_else(|| anyhow!("missing {PROGRESS_KEY}"))?
                .to_owned(),
        )?;

        let progress_count = from_ipld(
            map.get(PROGRESS_COUNT_KEY)
                .ok_or_else(|| anyhow!("missing {PROGRESS_COUNT_KEY}"))?
                .to_owned(),
        )?;

        Ok(PutWorkflowInfoDht {
            timestamp,
            cid,
            name,
            num_tasks,
            progress,
            progress_count,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "got_workflow_info_dht")]
pub struct GotWorkflowInfoDht {
    timestamp: i64,
    #[schemars(description = "Workflow info publisher peer ID")]
    publisher: Option<String>,
    #[schemars(description = "Workflow info CID")]
    cid: String,
    #[schemars(description = "Optional workflow name")]
    name: Option<String>,
    #[schemars(description = "Number of tasks in workflow")]
    num_tasks: u32,
    #[schemars(description = "Completed task CIDs")]
    progress: Vec<String>,
    #[schemars(description = "Number of workflow tasks completed")]
    progress_count: u32,
}

impl GotWorkflowInfoDht {
    pub(crate) fn new(
        publisher: Option<PeerId>,
        cid: Cid,
        name: Option<FastStr>,
        num_tasks: u32,
        progress: Vec<Cid>,
        progress_count: u32,
    ) -> GotWorkflowInfoDht {
        GotWorkflowInfoDht {
            timestamp: Utc::now().timestamp_millis(),
            publisher: publisher.map(|p| p.to_string()),
            cid: cid.to_string(),
            name: name.map(|n| n.into()),
            num_tasks,
            progress: progress.iter().map(|cid| cid.to_string()).collect(),
            progress_count,
        }
    }
}

impl DagJson for GotWorkflowInfoDht {}

impl From<GotWorkflowInfoDht> for Ipld {
    fn from(notification: GotWorkflowInfoDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (
                PUBLISHER_KEY.into(),
                notification
                    .publisher
                    .map(|peer_id| peer_id.into())
                    .unwrap_or(Ipld::Null),
            ),
            (CID_KEY.into(), notification.cid.into()),
            (
                NAME_KEY.into(),
                notification
                    .name
                    .map(|peer_id| peer_id.into())
                    .unwrap_or(Ipld::Null),
            ),
            (NUM_TASKS_KEY.into(), notification.num_tasks.into()),
            (
                PROGRESS_KEY.into(),
                Ipld::List(
                    notification
                        .progress
                        .iter()
                        .map(|cid| Ipld::String(cid.to_string()))
                        .collect(),
                ),
            ),
            (
                PROGRESS_COUNT_KEY.into(),
                notification.progress_count.into(),
            ),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for GotWorkflowInfoDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let publisher = map
            .get(PUBLISHER_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok());

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
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
                .ok_or_else(|| anyhow!("missing {NUM_TASKS_KEY}"))?
                .to_owned(),
        )?;

        let progress = from_ipld::<Vec<String>>(
            map.get(PROGRESS_KEY)
                .ok_or_else(|| anyhow!("missing {PROGRESS_KEY}"))?
                .to_owned(),
        )?;

        let progress_count = from_ipld(
            map.get(PROGRESS_COUNT_KEY)
                .ok_or_else(|| anyhow!("missing {PROGRESS_COUNT_KEY}"))?
                .to_owned(),
        )?;

        Ok(GotWorkflowInfoDht {
            timestamp,
            publisher,
            cid,
            name,
            num_tasks,
            progress,
            progress_count,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "receipt_quorum_success_dht")]
pub struct ReceiptQuorumSuccessDht {
    timestamp: i64,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Number of peers participating in quorum")]
    quorum: usize,
}

impl ReceiptQuorumSuccessDht {
    pub(crate) fn new(cid: FastStr, quorum: usize) -> ReceiptQuorumSuccessDht {
        ReceiptQuorumSuccessDht {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            quorum,
        }
    }
}

impl DagJson for ReceiptQuorumSuccessDht {}

impl From<ReceiptQuorumSuccessDht> for Ipld {
    fn from(notification: ReceiptQuorumSuccessDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (QUORUM_KEY.into(), notification.quorum.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for ReceiptQuorumSuccessDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let quorum = from_ipld(
            map.get(QUORUM_KEY)
                .ok_or_else(|| anyhow!("missing {QUORUM_KEY}"))?
                .to_owned(),
        )?;

        Ok(ReceiptQuorumSuccessDht {
            timestamp,
            cid,
            quorum,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "receipt_quorum_failure_dht")]
pub struct ReceiptQuorumFailureDht {
    timestamp: i64,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Number of peers required for quorum")]
    quorum: usize,
    #[schemars(description = "Number of connected peers")]
    connected_peer_count: usize,
    #[schemars(description = "Peers participating in quorum")]
    stored_to_peers: Vec<String>,
}

impl ReceiptQuorumFailureDht {
    pub(crate) fn new(
        cid: FastStr,
        quorum: usize,
        connected_peer_count: usize,
        stored_to_peers: Vec<PeerId>,
    ) -> ReceiptQuorumFailureDht {
        ReceiptQuorumFailureDht {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            quorum,
            connected_peer_count,
            stored_to_peers: stored_to_peers.iter().map(|p| p.to_string()).collect(),
        }
    }
}

impl DagJson for ReceiptQuorumFailureDht {}

impl From<ReceiptQuorumFailureDht> for Ipld {
    fn from(notification: ReceiptQuorumFailureDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (QUORUM_KEY.into(), notification.quorum.into()),
            (
                CONNECTED_PEER_COUNT_KEY.into(),
                notification.connected_peer_count.into(),
            ),
            (
                STORED_TO_PEERS_KEY.into(),
                Ipld::List(
                    notification
                        .stored_to_peers
                        .iter()
                        .map(|p| Ipld::String(p.to_string()))
                        .collect(),
                ),
            ),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for ReceiptQuorumFailureDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let quorum = from_ipld(
            map.get(QUORUM_KEY)
                .ok_or_else(|| anyhow!("missing {QUORUM_KEY}"))?
                .to_owned(),
        )?;

        let connected_peer_count = from_ipld(
            map.get(CONNECTED_PEER_COUNT_KEY)
                .ok_or_else(|| anyhow!("missing {CONNECTED_PEER_COUNT_KEY}"))?
                .to_owned(),
        )?;

        let stored_to_peers = from_ipld(
            map.get(STORED_TO_PEERS_KEY)
                .ok_or_else(|| anyhow!("missing {STORED_TO_PEERS_KEY}"))?
                .to_owned(),
        )?;

        Ok(ReceiptQuorumFailureDht {
            timestamp,
            cid,
            quorum,
            connected_peer_count,
            stored_to_peers,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "workflow_info_quorum_success_dht")]
pub struct WorkflowInfoQuorumSuccessDht {
    timestamp: i64,
    #[schemars(description = "Workflow info CID")]
    cid: String,
    #[schemars(description = "Number of peers participating in quorum")]
    quorum: usize,
}

impl WorkflowInfoQuorumSuccessDht {
    pub(crate) fn new(cid: FastStr, quorum: usize) -> WorkflowInfoQuorumSuccessDht {
        WorkflowInfoQuorumSuccessDht {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            quorum,
        }
    }
}

impl DagJson for WorkflowInfoQuorumSuccessDht {}

impl From<WorkflowInfoQuorumSuccessDht> for Ipld {
    fn from(notification: WorkflowInfoQuorumSuccessDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (QUORUM_KEY.into(), notification.quorum.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for WorkflowInfoQuorumSuccessDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let quorum = from_ipld(
            map.get(QUORUM_KEY)
                .ok_or_else(|| anyhow!("missing {QUORUM_KEY}"))?
                .to_owned(),
        )?;

        Ok(WorkflowInfoQuorumSuccessDht {
            timestamp,
            cid,
            quorum,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "workflow_info_quorum_failure_dht")]
pub struct WorkflowInfoQuorumFailureDht {
    timestamp: i64,
    #[schemars(description = "Workflow info CID")]
    cid: String,
    #[schemars(description = "Number of peers required for quorum")]
    quorum: usize,
    #[schemars(description = "Number of connected peers")]
    connected_peer_count: usize,
    #[schemars(description = "Peers participating in quorum")]
    stored_to_peers: Vec<String>,
}

impl WorkflowInfoQuorumFailureDht {
    pub(crate) fn new(
        cid: FastStr,
        quorum: usize,
        connected_peer_count: usize,
        stored_to_peers: Vec<PeerId>,
    ) -> WorkflowInfoQuorumFailureDht {
        WorkflowInfoQuorumFailureDht {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            quorum,
            connected_peer_count,
            stored_to_peers: stored_to_peers.iter().map(|p| p.to_string()).collect(),
        }
    }
}

impl DagJson for WorkflowInfoQuorumFailureDht {}

impl From<WorkflowInfoQuorumFailureDht> for Ipld {
    fn from(notification: WorkflowInfoQuorumFailureDht) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (QUORUM_KEY.into(), notification.quorum.into()),
            (
                CONNECTED_PEER_COUNT_KEY.into(),
                notification.connected_peer_count.into(),
            ),
            (
                STORED_TO_PEERS_KEY.into(),
                Ipld::List(
                    notification
                        .stored_to_peers
                        .iter()
                        .map(|p| Ipld::String(p.to_string()))
                        .collect(),
                ),
            ),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for WorkflowInfoQuorumFailureDht {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let quorum = from_ipld(
            map.get(QUORUM_KEY)
                .ok_or_else(|| anyhow!("missing {QUORUM_KEY}"))?
                .to_owned(),
        )?;

        let connected_peer_count = from_ipld(
            map.get(CONNECTED_PEER_COUNT_KEY)
                .ok_or_else(|| anyhow!("missing {CONNECTED_PEER_COUNT_KEY}"))?
                .to_owned(),
        )?;

        let stored_to_peers = from_ipld(
            map.get(STORED_TO_PEERS_KEY)
                .ok_or_else(|| anyhow!("missing {STORED_TO_PEERS_KEY}"))?
                .to_owned(),
        )?;

        Ok(WorkflowInfoQuorumFailureDht {
            timestamp,
            cid,
            quorum,
            connected_peer_count,
            stored_to_peers,
        })
    }
}
