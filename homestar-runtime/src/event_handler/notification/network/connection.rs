//! Notification types for [swarm] connection events.
//!
//! [swarm]: libp2p::swarm::Swarm

use anyhow::anyhow;
use chrono::prelude::Utc;
use derive_getters::Getters;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Ipld};
use libp2p::{
    swarm::{DialError, ListenError},
    Multiaddr, PeerId,
};
use schemars::JsonSchema;
use std::collections::BTreeMap;

const ADDRESS_KEY: &str = "address";
const ERROR_KEY: &str = "error";
const PEER_KEY: &str = "peer_id";
const TIMESTAMP_KEY: &str = "timestamp";

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "new_listen_addr")]
pub struct NewListenAddr {
    timestamp: i64,
    peer_id: String,
    address: String,
}

impl NewListenAddr {
    pub(crate) fn new(peer_id: PeerId, address: Multiaddr) -> NewListenAddr {
        NewListenAddr {
            timestamp: Utc::now().timestamp_millis(),
            peer_id: peer_id.to_string(),
            address: address.to_string(),
        }
    }
}

impl DagJson for NewListenAddr {}

impl From<NewListenAddr> for Ipld {
    fn from(notification: NewListenAddr) -> Self {
        Ipld::Map(BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (PEER_KEY.into(), notification.peer_id.into()),
            (ADDRESS_KEY.into(), notification.address.into()),
        ]))
    }
}

impl TryFrom<Ipld> for NewListenAddr {
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

        let address = from_ipld(
            map.get(ADDRESS_KEY)
                .ok_or_else(|| anyhow!("missing {ADDRESS_KEY}"))?
                .to_owned(),
        )?;

        Ok(NewListenAddr {
            timestamp,
            peer_id,
            address,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "connection_established")]
pub struct ConnectionEstablished {
    timestamp: i64,
    peer_id: String,
    address: String,
}

impl ConnectionEstablished {
    pub(crate) fn new(peer_id: PeerId, address: Multiaddr) -> ConnectionEstablished {
        ConnectionEstablished {
            timestamp: Utc::now().timestamp_millis(),
            peer_id: peer_id.to_string(),
            address: address.to_string(),
        }
    }
}

impl DagJson for ConnectionEstablished {}

impl From<ConnectionEstablished> for Ipld {
    fn from(notification: ConnectionEstablished) -> Self {
        Ipld::Map(BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (PEER_KEY.into(), notification.peer_id.into()),
            (ADDRESS_KEY.into(), notification.address.into()),
        ]))
    }
}

impl TryFrom<Ipld> for ConnectionEstablished {
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

        let address = from_ipld(
            map.get(ADDRESS_KEY)
                .ok_or_else(|| anyhow!("missing {ADDRESS_KEY}"))?
                .to_owned(),
        )?;

        Ok(ConnectionEstablished {
            timestamp,
            peer_id,
            address,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "connection_closed")]
pub struct ConnectionClosed {
    timestamp: i64,
    peer_id: String,
    address: String,
}

impl ConnectionClosed {
    pub(crate) fn new(peer_id: PeerId, address: Multiaddr) -> ConnectionClosed {
        ConnectionClosed {
            timestamp: Utc::now().timestamp_millis(),
            peer_id: peer_id.to_string(),
            address: address.to_string(),
        }
    }
}

impl DagJson for ConnectionClosed {}

impl From<ConnectionClosed> for Ipld {
    fn from(notification: ConnectionClosed) -> Self {
        Ipld::Map(BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (PEER_KEY.into(), notification.peer_id.into()),
            (ADDRESS_KEY.into(), notification.address.into()),
        ]))
    }
}

impl TryFrom<Ipld> for ConnectionClosed {
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

        let address = from_ipld(
            map.get(ADDRESS_KEY)
                .ok_or_else(|| anyhow!("missing {ADDRESS_KEY}"))?
                .to_owned(),
        )?;

        Ok(ConnectionClosed {
            timestamp,
            peer_id,
            address,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "outgoing_connection_error")]
pub struct OutgoingConnectionError {
    timestamp: i64,
    peer_id: Option<String>,
    error: String,
}

impl OutgoingConnectionError {
    pub(crate) fn new(peer_id: Option<PeerId>, error: DialError) -> OutgoingConnectionError {
        OutgoingConnectionError {
            timestamp: Utc::now().timestamp_millis(),
            peer_id: peer_id.map(|p| p.to_string()),
            error: error.to_string(),
        }
    }
}

impl DagJson for OutgoingConnectionError {}

impl From<OutgoingConnectionError> for Ipld {
    fn from(notification: OutgoingConnectionError) -> Self {
        Ipld::Map(BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (
                PEER_KEY.into(),
                notification
                    .peer_id
                    .map(|peer_id| peer_id.into())
                    .unwrap_or(Ipld::Null),
            ),
            (ERROR_KEY.into(), notification.error.into()),
        ]))
    }
}

impl TryFrom<Ipld> for OutgoingConnectionError {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let peer_id = map
            .get(PEER_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok());

        let error = from_ipld(
            map.get(ERROR_KEY)
                .ok_or_else(|| anyhow!("missing {ERROR_KEY}"))?
                .to_owned(),
        )?;

        Ok(OutgoingConnectionError {
            timestamp,
            peer_id,
            error,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "incoming_connection_error")]
pub struct IncomingConnectionError {
    timestamp: i64,
    error: String,
}

impl IncomingConnectionError {
    pub(crate) fn new(error: ListenError) -> IncomingConnectionError {
        IncomingConnectionError {
            timestamp: Utc::now().timestamp_millis(),
            error: error.to_string(),
        }
    }
}

impl DagJson for IncomingConnectionError {}

impl From<IncomingConnectionError> for Ipld {
    fn from(notification: IncomingConnectionError) -> Self {
        Ipld::Map(BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (ERROR_KEY.into(), notification.error.into()),
        ]))
    }
}

impl TryFrom<Ipld> for IncomingConnectionError {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let error = from_ipld(
            map.get(ERROR_KEY)
                .ok_or_else(|| anyhow!("missing {ERROR_KEY}"))?
                .to_owned(),
        )?;

        Ok(IncomingConnectionError { timestamp, error })
    }
}
