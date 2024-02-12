//! Notification types for [swarm] request_reponse events.
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
const NAME_KEY: &str = "name";
const NUM_TASKS_KEY: &str = "num_tasks";
const PROGRESS_KEY: &str = "progress";
const PROGRESS_COUNT_KEY: &str = "progress_count";
const PROVIDER_KEY: &str = "provider";
const REQUESTOR_KEY: &str = "requestor";
const TIMESTAMP_KEY: &str = "timestamp";

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "sent_workflow_info")]
pub struct SentWorkflowInfo {
    timestamp: i64,
    #[schemars(description = "Peer that requested workflow info")]
    requestor: String,
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

impl SentWorkflowInfo {
    pub(crate) fn new(
        requestor: PeerId,
        cid: Cid,
        name: Option<FastStr>,
        num_tasks: u32,
        progress: Vec<Cid>,
        progress_count: u32,
    ) -> SentWorkflowInfo {
        SentWorkflowInfo {
            requestor: requestor.to_string(),
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            name: name.map(|n| n.into()),
            num_tasks,
            progress: progress.iter().map(|cid| cid.to_string()).collect(),
            progress_count,
        }
    }
}

impl DagJson for SentWorkflowInfo {}

impl From<SentWorkflowInfo> for Ipld {
    fn from(notification: SentWorkflowInfo) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (REQUESTOR_KEY.into(), notification.requestor.into()),
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

impl TryFrom<Ipld> for SentWorkflowInfo {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let requestor = from_ipld(
            map.get(REQUESTOR_KEY)
                .ok_or_else(|| anyhow!("missing {REQUESTOR_KEY}"))?
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

        Ok(SentWorkflowInfo {
            timestamp,
            requestor,
            cid,
            name,
            num_tasks,
            progress,
            progress_count,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "received_workflow_info")]
pub struct ReceivedWorkflowInfo {
    timestamp: i64,
    #[schemars(description = "Workflow info provider peer ID")]
    provider: Option<String>,
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

impl ReceivedWorkflowInfo {
    pub(crate) fn new(
        provider: Option<PeerId>,
        cid: Cid,
        name: Option<FastStr>,
        num_tasks: u32,
        progress: Vec<Cid>,
        progress_count: u32,
    ) -> ReceivedWorkflowInfo {
        ReceivedWorkflowInfo {
            timestamp: Utc::now().timestamp_millis(),
            provider: provider.map(|p| p.to_string()),
            cid: cid.to_string(),
            name: name.map(|n| n.into()),
            num_tasks,
            progress: progress.iter().map(|cid| cid.to_string()).collect(),
            progress_count,
        }
    }
}

impl DagJson for ReceivedWorkflowInfo {}

impl From<ReceivedWorkflowInfo> for Ipld {
    fn from(notification: ReceivedWorkflowInfo) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (
                PROVIDER_KEY.into(),
                notification
                    .provider
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

impl TryFrom<Ipld> for ReceivedWorkflowInfo {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let provider = map
            .get(PROVIDER_KEY)
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

        Ok(ReceivedWorkflowInfo {
            timestamp,
            provider,
            cid,
            name,
            num_tasks,
            progress,
            progress_count,
        })
    }
}
