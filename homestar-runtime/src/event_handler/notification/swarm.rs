// Notification types for [swarm] events.
//
// [swarm]: libp2p_swarm::Swarm

use anyhow::anyhow;
use chrono::prelude::Utc;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Ipld};
use libp2p::{Multiaddr, PeerId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, str::FromStr};

const TIMESTAMP_KEY: &str = "timestamp";

// Swarm notification types sent to clients
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum SwarmNotification {
    ConnnectionEstablished,
    ConnnectionClosed,
    ListeningOn,
    OutgoingConnectionError,
    IncomingConnectionError,
    PublishedReceiptPubsub,
    ReceivedReceiptPubsub,
    GotReceiptDht,
    PutReceiptDht,
    GotWorkflowInfoDht,
    PutWorkflowInfoDht,
    ReceiptQuorumSuccess,
    ReceiptQuorumFailure,
    WorkflowInfoQuorumSuccess,
    WorkflowInfoQuorumFailure,
    SentWorkflowInfo,
    ReceivedWorkflowInfo,
}

impl fmt::Display for SwarmNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SwarmNotification::ConnnectionEstablished => write!(f, "connectionEstablished"),
            SwarmNotification::ConnnectionClosed => write!(f, "connectionClosed"),
            SwarmNotification::ListeningOn => write!(f, "listeningOn"),
            SwarmNotification::OutgoingConnectionError => {
                write!(f, "outgoingConnectionError")
            }
            SwarmNotification::IncomingConnectionError => {
                write!(f, "incomingConnectionError")
            }
            SwarmNotification::ReceivedReceiptPubsub => {
                write!(f, "receivedReceiptPubsub")
            }
            SwarmNotification::PublishedReceiptPubsub => {
                write!(f, "publishedReceiptPubsub")
            }
            SwarmNotification::PutReceiptDht => {
                write!(f, "putReceiptDht")
            }
            SwarmNotification::GotReceiptDht => {
                write!(f, "gotReceiptDht")
            }
            SwarmNotification::PutWorkflowInfoDht => {
                write!(f, "putWorkflowInfoDht")
            }
            SwarmNotification::GotWorkflowInfoDht => {
                write!(f, "gotWorkflowInfoDht")
            }
            SwarmNotification::ReceiptQuorumSuccess => {
                write!(f, "receiptQuorumSuccess")
            }
            SwarmNotification::ReceiptQuorumFailure => {
                write!(f, "receiptQuorumFailure")
            }
            SwarmNotification::WorkflowInfoQuorumSuccess => {
                write!(f, "workflowInfoQuorumSuccess")
            }
            SwarmNotification::WorkflowInfoQuorumFailure => {
                write!(f, "workflowInfoQuorumFailure")
            }
            SwarmNotification::SentWorkflowInfo => {
                write!(f, "sentWorkflowInfo")
            }
            SwarmNotification::ReceivedWorkflowInfo => {
                write!(f, "receivedWorkflowInfo")
            }
        }
    }
}

impl FromStr for SwarmNotification {
    type Err = anyhow::Error;

    fn from_str(ty: &str) -> Result<Self, Self::Err> {
        match ty {
            "connectionEstablished" => Ok(Self::ConnnectionEstablished),
            "connectionClosed" => Ok(Self::ConnnectionClosed),
            "listeningOn" => Ok(Self::ListeningOn),
            "outgoingConnectionError" => Ok(Self::OutgoingConnectionError),
            "incomingConnectionError" => Ok(Self::IncomingConnectionError),
            "receivedReceiptPubsub" => Ok(Self::ReceivedReceiptPubsub),
            "publishedReceiptPubsub" => Ok(Self::PublishedReceiptPubsub),
            "putReciptDht" => Ok(Self::PutReceiptDht),
            "gotReceiptDht" => Ok(Self::GotReceiptDht),
            "putWorkflowInfoDht" => Ok(Self::PutWorkflowInfoDht),
            "gotWorkflowInfoDht" => Ok(Self::GotWorkflowInfoDht),
            "receiptQuorumSuccess" => Ok(Self::ReceiptQuorumSuccess),
            "receiptQuorumFailure" => Ok(Self::ReceiptQuorumFailure),
            "workflowInfoQuorumSuccess" => Ok(Self::WorkflowInfoQuorumSuccess),
            "workflowInfoQuorumFailure" => Ok(Self::WorkflowInfoQuorumFailure),
            "sentWorkflowInfo" => Ok(Self::SentWorkflowInfo),
            "receivedWorkflowInfo" => Ok(Self::ReceivedWorkflowInfo),
            _ => Err(anyhow!("Missing swarm notification type: {}", ty)),
        }
    }
}

/// Network notification type.
#[derive(Clone, JsonSchema, Debug)]
#[schemars(rename = "network")]
pub enum NetworkNotification {
    /// Connection established notification.
    #[schemars(rename = "connection_established")]
    ConnnectionEstablished(ConnectionEstablished),
    /// Connection closed notification.
    #[schemars(rename = "connection_closed")]
    ConnnectionClosed(ConnectionClosed),
}

impl DagJson for NetworkNotification {}

impl From<NetworkNotification> for Ipld {
    fn from(notification: NetworkNotification) -> Self {
        match notification {
            NetworkNotification::ConnnectionEstablished(n) => Ipld::Map(BTreeMap::from([(
                "connection_established".into(),
                n.into(),
            )])),
            NetworkNotification::ConnnectionClosed(n) => {
                Ipld::Map(BTreeMap::from([("connection_closed".into(), n.into())]))
            }
        }
    }
}

impl TryFrom<Ipld> for NetworkNotification {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        if let Some((key, val)) = map.first_key_value() {
            match key.as_str() {
                "connection_established" => Ok(NetworkNotification::ConnnectionEstablished(
                    ConnectionEstablished::try_from(val.to_owned())?,
                )),
                "connection_closed" => Ok(NetworkNotification::ConnnectionClosed(
                    ConnectionClosed::try_from(val.to_owned())?,
                )),
                _ => Err(anyhow!("Unknown network notification tag type")),
            }
        } else {
            Err(anyhow!("Network notification was an empty map"))
        }
    }
}

#[derive(JsonSchema, Debug, Clone)]
#[schemars(rename = "connection_established")]
pub struct ConnectionEstablished {
    timestamp: i64,
    peer_id: String,
    address: String,
}

impl ConnectionEstablished {
    pub(crate) fn new(peer_id: PeerId, address: Multiaddr) -> ConnectionEstablished {
        ConnectionEstablished {
            peer_id: peer_id.to_string(),
            timestamp: Utc::now().timestamp_millis(),
            address: address.to_string(),
        }
    }
}

impl DagJson for ConnectionEstablished {}

impl From<ConnectionEstablished> for Ipld {
    fn from(notification: ConnectionEstablished) -> Self {
        Ipld::Map(BTreeMap::from([
            ("timestamp".into(), notification.timestamp.into()),
            ("peer_id".into(), notification.peer_id.into()),
            ("address".into(), notification.address.into()),
        ]))
    }
}

impl TryFrom<Ipld> for ConnectionEstablished {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let peer_key: &str = "peer_id";
        let address_key: &str = "address";

        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let peer_id = from_ipld(
            map.get(peer_key)
                .ok_or_else(|| anyhow!("missing {peer_key}"))?
                .to_owned(),
        )?;

        let address = from_ipld(
            map.get(address_key)
                .ok_or_else(|| anyhow!("missing {address_key}"))?
                .to_owned(),
        )?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        Ok(ConnectionEstablished {
            timestamp,
            peer_id,
            address,
        })
    }
}

#[derive(JsonSchema, Debug, Clone)]
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
            ("timestamp".into(), notification.timestamp.into()),
            ("peer_id".into(), notification.peer_id.into()),
            ("address".into(), notification.address.into()),
        ]))
    }
}

impl TryFrom<Ipld> for ConnectionClosed {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let peer_key: &str = "peer_id";
        let address_key: &str = "address";

        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let peer_id = from_ipld(
            map.get(peer_key)
                .ok_or_else(|| anyhow!("missing {peer_key}"))?
                .to_owned(),
        )?;

        let address = from_ipld(
            map.get(address_key)
                .ok_or_else(|| anyhow!("missing {address_key}"))?
                .to_owned(),
        )?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        Ok(ConnectionClosed {
            timestamp,
            peer_id,
            address,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn notification_bytes_rountrip() {
        let peer_id = PeerId::random();
        let address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap();
        let inner = ConnectionEstablished::new(peer_id, address.clone());

        let notification = NetworkNotification::ConnnectionEstablished(inner.clone());
        let bytes = notification.to_json().unwrap();
        let parsed = NetworkNotification::from_json(bytes.as_ref()).unwrap();

        match parsed {
            NetworkNotification::ConnnectionEstablished(n) => {
                let parsed_peer_id = PeerId::from_str(&n.peer_id).unwrap();
                let parsed_address = Multiaddr::from_str(&n.address).unwrap();

                assert_eq!(parsed_peer_id, peer_id);
                assert_eq!(parsed_address, address);
                assert_eq!(n.timestamp, inner.timestamp);
            }
            _ => panic!("Parsed notification did not matched expected variant"),
        }
    }

    #[test]
    fn notification_json_string_rountrip() {
        let peer_id = PeerId::random();
        let address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap();
        let inner = ConnectionEstablished::new(peer_id, address.clone());

        let notification = NetworkNotification::ConnnectionEstablished(inner.clone());
        let json_string = notification.to_json_string().unwrap();
        let parsed = NetworkNotification::from_json_string(json_string).unwrap();

        match parsed {
            NetworkNotification::ConnnectionEstablished(n) => {
                let parsed_peer_id = PeerId::from_str(&n.peer_id).unwrap();
                let parsed_address = Multiaddr::from_str(&n.address).unwrap();

                assert_eq!(parsed_peer_id, peer_id);
                assert_eq!(parsed_address, address);
                assert_eq!(n.timestamp, inner.timestamp);
            }
            _ => panic!("Parsed notification did not matched expected variant"),
        }
    }
}
