// Notification types for [swarm] events.
//
// [swarm]: libp2p_swarm::Swarm

use anyhow::anyhow;
use chrono::prelude::Utc;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Ipld};
use libp2p::{Multiaddr, PeerId};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, fmt, str::FromStr};

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
    /// mDNS discovered notification.
    #[schemars(rename = "discovered_mdns")]
    DiscoveredMdns(DiscoveredMdns),
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
            NetworkNotification::DiscoveredMdns(n) => {
                Ipld::Map(BTreeMap::from([("discovered_mdns".into(), n.into())]))
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
                "discovered_mdns" => Ok(NetworkNotification::DiscoveredMdns(
                    DiscoveredMdns::try_from(val.to_owned())?,
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

#[derive(Debug, Clone)]
pub struct DiscoveredMdns {
    timestamp: i64,
    peers: Vec<(String, String)>,
}

impl DiscoveredMdns {
    pub(crate) fn new(peers: Vec<(PeerId, Multiaddr)>) -> DiscoveredMdns {
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
            ("timestamp".into(), notification.timestamp.into()),
            ("peers".into(), peers.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for DiscoveredMdns {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let peers_key: &str = "peers";
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let peers_map = from_ipld::<BTreeMap<String, Ipld>>(
            map.get(peers_key)
                .ok_or_else(|| anyhow!("missing {peers_key}"))?
                .to_owned(),
        )?;

        let mut peers: Vec<(String, String)> = vec![];
        for peer in peers_map.iter() {
            peers.push((peer.0.to_string(), from_ipld(peer.1.to_owned())?))
        }

        Ok(DiscoveredMdns { timestamp, peers })
    }
}

impl JsonSchema for DiscoveredMdns {
    fn schema_name() -> String {
        "discovered_mdns".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-runtime::event_handler::notification::swarm::DiscoveredMdns")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([
                    (
                        "timestamp".to_string(),
                        Schema::Object(SchemaObject {
                            instance_type: Some(SingleOrVec::Single(InstanceType::Number.into())),
                            ..Default::default()
                        }),
                    ),
                    (
                        "peers".to_string(),
                        Schema::Object(SchemaObject {
                            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
                            metadata: Some(Box::new(Metadata {
                                description: Some("Peers and their addresses".to_string()),
                                ..Default::default()
                            })),
                            object: Some(Box::new(ObjectValidation {
                                additional_properties: Some(Box::new(<String>::json_schema(gen))),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }),
                    ),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        };
        schema.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug)]
    struct Fixtures {
        peer_id: PeerId,
        address: Multiaddr,
        peers: Vec<(PeerId, Multiaddr)>,
    }

    fn generate_notifications(fixtures: Fixtures) -> Vec<(i64, NetworkNotification)> {
        let Fixtures {
            peer_id,
            address,
            peers,
        } = fixtures;
        let connection_established = ConnectionEstablished::new(peer_id, address.clone());
        let connection_closed = ConnectionClosed::new(peer_id, address.clone());
        let discovered_mdns = DiscoveredMdns::new(peers);

        vec![
            (
                connection_established.timestamp,
                NetworkNotification::ConnnectionEstablished(connection_established.clone()),
            ),
            (
                connection_closed.timestamp,
                NetworkNotification::ConnnectionClosed(connection_closed.clone()),
            ),
            (
                discovered_mdns.timestamp,
                NetworkNotification::DiscoveredMdns(discovered_mdns.clone()),
            ),
        ]
    }

    fn check_notification(timestamp: i64, notification: NetworkNotification, fixtures: Fixtures) {
        let Fixtures {
            peer_id,
            address,
            peers,
        } = fixtures;

        match notification {
            NetworkNotification::ConnnectionEstablished(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.peer_id).unwrap(), peer_id);
                assert_eq!(Multiaddr::from_str(&n.address).unwrap(), address);
            }
            NetworkNotification::ConnnectionClosed(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.peer_id).unwrap(), peer_id);
                assert_eq!(Multiaddr::from_str(&n.address).unwrap(), address);
            }
            NetworkNotification::DiscoveredMdns(n) => {
                assert_eq!(n.timestamp, timestamp);

                for peer in n.peers {
                    assert!(peers.contains(&(
                        PeerId::from_str(&peer.0).unwrap(),
                        Multiaddr::from_str(&peer.1).unwrap()
                    )))
                }
            }
        }
    }

    #[test]
    fn notification_bytes_rountrip() {
        let fixtures = Fixtures {
            peer_id: PeerId::random(),
            address: Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
            peers: vec![
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
                ),
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7001").unwrap(),
                ),
            ],
        };

        // Generate notifications and convert them to bytes
        let notifications: Vec<(i64, Vec<u8>)> = generate_notifications(fixtures.clone())
            .into_iter()
            .map(|(timestamp, notification)| (timestamp, notification.to_json().unwrap()))
            .collect();

        // Convert notifications back and check them
        for (timestamp, bytes) in notifications {
            check_notification(
                timestamp,
                NetworkNotification::from_json(bytes.as_ref()).unwrap(),
                fixtures.clone(),
            )
        }
    }

    #[test]
    fn notification_json_string_rountrip() {
        let fixtures = Fixtures {
            peer_id: PeerId::random(),
            address: Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
            peers: vec![
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
                ),
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7001").unwrap(),
                ),
            ],
        };

        // Generate notifications and convert them to JSON strings
        let notifications: Vec<(i64, String)> = generate_notifications(fixtures.clone())
            .into_iter()
            .map(|(timestamp, notification)| (timestamp, notification.to_json_string().unwrap()))
            .collect();

        // Convert notifications back and check them
        for (timestamp, json) in notifications {
            check_notification(
                timestamp,
                NetworkNotification::from_json_string(json).unwrap(),
                fixtures.clone(),
            )
        }
    }
}
