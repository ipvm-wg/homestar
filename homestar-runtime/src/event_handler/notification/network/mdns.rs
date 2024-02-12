//! Notification types for [swarm] mDNS events.
//!
//! [swarm]: libp2p::swarm::Swarm

use anyhow::anyhow;
use chrono::prelude::Utc;
use derive_getters::Getters;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Ipld};
use libp2p::{Multiaddr, PeerId};
use schemars::JsonSchema;
use std::collections::BTreeMap;

const PEERS_KEY: &str = "peers";
const TIMESTAMP_KEY: &str = "timestamp";

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "discovered_mdns")]
pub struct DiscoveredMdns {
    timestamp: i64,
    #[schemars(description = "Peers discovered by peer ID and multiaddress")]
    peers: BTreeMap<String, String>,
}

impl DiscoveredMdns {
    pub(crate) fn new(peers: BTreeMap<PeerId, Multiaddr>) -> DiscoveredMdns {
        DiscoveredMdns {
            timestamp: Utc::now().timestamp_millis(),
            peers: peers
                .iter()
                .map(|(peer_id, address)| (peer_id.to_string(), address.to_string()))
                .collect(),
        }
    }
}

impl DagJson for DiscoveredMdns {}

impl From<DiscoveredMdns> for Ipld {
    fn from(notification: DiscoveredMdns) -> Self {
        let peers: BTreeMap<String, Ipld> = notification
            .peers
            .into_iter()
            .map(|(peer_id, address)| (peer_id, address.into()))
            .collect();

        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (PEERS_KEY.into(), peers.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for DiscoveredMdns {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let peers = from_ipld::<BTreeMap<String, String>>(
            map.get(PEERS_KEY)
                .ok_or_else(|| anyhow!("missing {PEERS_KEY}"))?
                .to_owned(),
        )?;

        Ok(DiscoveredMdns { timestamp, peers })
    }
}
