//! Notification types for [swarm] rendezvous events.
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

const ADDRESSES_KEY: &str = "addresses";
const ENQUIRER_KEY: &str = "enquirer";
const PEER_KEY: &str = "peer_id";
const PEERS_KEY: &str = "peers";
const SERVER_KEY: &str = "server";
const TIMESTAMP_KEY: &str = "timestamp";

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "discovered_rendezvous")]
pub struct DiscoveredRendezvous {
    timestamp: i64,
    #[schemars(description = "Server that fulfilled the discovery request")]
    server: String,
    #[schemars(description = "Peers discovered by peer ID and multiaddresses")]
    peers: BTreeMap<String, Vec<String>>,
}

impl DiscoveredRendezvous {
    pub(crate) fn new(
        server: PeerId,
        peers: BTreeMap<PeerId, Vec<Multiaddr>>,
    ) -> DiscoveredRendezvous {
        DiscoveredRendezvous {
            timestamp: Utc::now().timestamp_millis(),
            server: server.to_string(),
            peers: peers
                .iter()
                .map(|(peer_id, addresses)| {
                    (
                        peer_id.to_string(),
                        addresses
                            .iter()
                            .map(|address| address.to_string())
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}

impl DagJson for DiscoveredRendezvous {}

impl From<DiscoveredRendezvous> for Ipld {
    fn from(notification: DiscoveredRendezvous) -> Self {
        let peers: BTreeMap<String, Ipld> = notification
            .peers
            .into_iter()
            .map(|(peer_id, addresses)| {
                (
                    peer_id,
                    Ipld::List(
                        addresses
                            .iter()
                            .map(|address| Ipld::String(address.to_owned()))
                            .collect(),
                    ),
                )
            })
            .collect();

        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (SERVER_KEY.into(), notification.server.into()),
            (PEERS_KEY.into(), peers.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for DiscoveredRendezvous {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let server = from_ipld(
            map.get(SERVER_KEY)
                .ok_or_else(|| anyhow!("missing {SERVER_KEY}"))?
                .to_owned(),
        )?;

        let peers = from_ipld::<BTreeMap<String, Vec<String>>>(
            map.get(PEERS_KEY)
                .ok_or_else(|| anyhow!("missing {PEERS_KEY}"))?
                .to_owned(),
        )?;

        Ok(DiscoveredRendezvous {
            timestamp,
            server,
            peers,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "registered_rendezvous")]
pub struct RegisteredRendezvous {
    timestamp: i64,
    #[schemars(description = "Server that accepted registration")]
    server: String,
}

impl RegisteredRendezvous {
    pub(crate) fn new(server: PeerId) -> RegisteredRendezvous {
        RegisteredRendezvous {
            timestamp: Utc::now().timestamp_millis(),
            server: server.to_string(),
        }
    }
}

impl DagJson for RegisteredRendezvous {}

impl From<RegisteredRendezvous> for Ipld {
    fn from(notification: RegisteredRendezvous) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (SERVER_KEY.into(), notification.server.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for RegisteredRendezvous {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let server = from_ipld(
            map.get(SERVER_KEY)
                .ok_or_else(|| anyhow!("missing {SERVER_KEY}"))?
                .to_owned(),
        )?;

        Ok(RegisteredRendezvous { timestamp, server })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "registered_rendezvous")]
pub struct DiscoverServedRendezvous {
    timestamp: i64,
    #[schemars(description = "Peer that requested discovery")]
    enquirer: String,
}

impl DiscoverServedRendezvous {
    pub(crate) fn new(enquirer: PeerId) -> DiscoverServedRendezvous {
        DiscoverServedRendezvous {
            timestamp: Utc::now().timestamp_millis(),
            enquirer: enquirer.to_string(),
        }
    }
}

impl DagJson for DiscoverServedRendezvous {}

impl From<DiscoverServedRendezvous> for Ipld {
    fn from(notification: DiscoverServedRendezvous) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (ENQUIRER_KEY.into(), notification.enquirer.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for DiscoverServedRendezvous {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let enquirer = from_ipld(
            map.get(ENQUIRER_KEY)
                .ok_or_else(|| anyhow!("missing {ENQUIRER_KEY}"))?
                .to_owned(),
        )?;

        Ok(DiscoverServedRendezvous {
            timestamp,
            enquirer,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "peer_registered_rendezvous")]
pub struct PeerRegisteredRendezvous {
    timestamp: i64,
    #[schemars(description = "Peer registered")]
    peer_id: String,
    #[schemars(description = "Multiaddresses for peer")]
    addresses: Vec<String>,
}

impl PeerRegisteredRendezvous {
    pub(crate) fn new(peer_id: PeerId, addresses: Vec<Multiaddr>) -> PeerRegisteredRendezvous {
        PeerRegisteredRendezvous {
            timestamp: Utc::now().timestamp_millis(),
            peer_id: peer_id.to_string(),
            addresses: addresses
                .iter()
                .map(|address| address.to_string())
                .collect(),
        }
    }
}

impl DagJson for PeerRegisteredRendezvous {}

impl From<PeerRegisteredRendezvous> for Ipld {
    fn from(notification: PeerRegisteredRendezvous) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (PEER_KEY.into(), notification.peer_id.into()),
            (
                ADDRESSES_KEY.into(),
                Ipld::List(
                    notification
                        .addresses
                        .iter()
                        .map(|address| Ipld::String(address.to_owned()))
                        .collect(),
                ),
            ),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for PeerRegisteredRendezvous {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let peer_id = from_ipld(
            map.get(PEER_KEY)
                .ok_or_else(|| anyhow!("missing {PEER_KEY}"))?
                .to_owned(),
        )?;

        let addresses = from_ipld(
            map.get(ADDRESSES_KEY)
                .ok_or_else(|| anyhow!("missing {ADDRESSES_KEY}"))?
                .to_owned(),
        )?;

        Ok(PeerRegisteredRendezvous {
            timestamp,
            peer_id,
            addresses,
        })
    }
}
