//! Notification types for [swarm] autonat events.
//!
//! [swarm]: libp2p::swarm::Swarm

use crate::libp2p::nat_status::NatStatusExt;
use anyhow::anyhow;
use chrono::prelude::Utc;
use derive_getters::Getters;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Ipld};
use libp2p::autonat::NatStatus;
use schemars::JsonSchema;
use std::collections::BTreeMap;

const ADDRESS_KEY: &str = "address";
const STATUS_KEY: &str = "status";
const TIMESTAMP_KEY: &str = "timestamp";

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "status_changed_autonat")]
pub struct StatusChangedAutonat {
    timestamp: i64,
    status: String,
    address: Option<String>,
}

impl StatusChangedAutonat {
    pub(crate) fn new(status: NatStatus) -> StatusChangedAutonat {
        let (status, address) = status.to_tuple();

        StatusChangedAutonat {
            timestamp: Utc::now().timestamp_millis(),
            status: status.to_string(),
            address: address.map(|a| a.to_string()),
        }
    }
}

impl DagJson for StatusChangedAutonat {}

impl From<StatusChangedAutonat> for Ipld {
    fn from(notification: StatusChangedAutonat) -> Self {
        Ipld::Map(BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (STATUS_KEY.into(), notification.status.into()),
            (
                ADDRESS_KEY.into(),
                notification
                    .address
                    .map(|peer_id| peer_id.into())
                    .unwrap_or(Ipld::Null),
            ),
        ]))
    }
}

impl TryFrom<Ipld> for StatusChangedAutonat {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let status = from_ipld(
            map.get(STATUS_KEY)
                .ok_or_else(|| anyhow!("missing {STATUS_KEY}"))?
                .to_owned(),
        )?;

        let address = map
            .get(ADDRESS_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok());

        Ok(StatusChangedAutonat {
            timestamp,
            status,
            address,
        })
    }
}
