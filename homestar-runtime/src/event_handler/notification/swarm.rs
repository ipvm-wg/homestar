// Notification types for [swarm] events.
//
// [swarm]: libp2p_swarm::Swarm

use anyhow::anyhow;
use chrono::prelude::Utc;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Cid, Ipld};
use libp2p::{
    swarm::{DialError, ListenError},
    Multiaddr, PeerId,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, str::FromStr};

const ADDRESS_KEY: &str = "address";
const ADDRESSES_KEY: &str = "addresses";
const CID_KEY: &str = "cid";
const ENQUIRER_KEY: &str = "enquirer";
const ERROR_KEY: &str = "error";
const PEER_KEY: &str = "peer_id";
const PEERS_KEY: &str = "peers";
const PUBLISHER_KEY: &str = "publisher";
const RAN_KEY: &str = "ran";
const SERVER_KEY: &str = "server";
const TIMESTAMP_KEY: &str = "timestamp";

// Swarm notification types sent to clients
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum SwarmNotification {
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

// TODO Fill these in for NetworkNotification
impl fmt::Display for SwarmNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
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
#[derive(Debug, Clone, JsonSchema)]
#[schemars(rename = "network")]
pub enum NetworkNotification {
    /// Listening on new address notification.
    #[schemars(rename = "new_listen_addr")]
    NewListenAddr(NewListenAddr),
    /// Connection established notification.
    #[schemars(rename = "connection_established")]
    ConnnectionEstablished(ConnectionEstablished),
    /// Connection closed notification.
    #[schemars(rename = "connection_closed")]
    ConnnectionClosed(ConnectionClosed),
    /// Outgoing conenction error notification.
    #[schemars(rename = "outgoing_connection_error")]
    OutgoingConnectionError(OutgoingConnectionError),
    /// Incoming conenction error notification.
    #[schemars(rename = "incoming_connection_error")]
    IncomingConnectionError(IncomingConnectionError),
    /// mDNS discovered notification.
    #[schemars(rename = "discovered_mdns")]
    DiscoveredMdns(DiscoveredMdns),
    /// Rendezvous client discovered notification.
    #[schemars(rename = "discovered_rendezvous")]
    DiscoveredRendezvous(DiscoveredRendezvous),
    /// Rendezvous client discovered notification.
    #[schemars(rename = "registered_rendezvous")]
    RegisteredRendezvous(RegisteredRendezvous),
    /// Rendezvous discover served notification.
    #[schemars(rename = "discover_served_rendezvous")]
    DiscoverServedRendezvous(DiscoverServedRendezvous),
    /// Rendezvous peer registered notification.
    #[schemars(rename = "peer_registered_rendezvous")]
    PeerRegisteredRendezvous(PeerRegisteredRendezvous),
    /// Published receipt pubsub notification.
    #[schemars(rename = "published_receipt_pubsub")]
    PublishedReceiptPubsub(PublishedReceiptPubsub),
    /// Received receipt pubsub notification.
    #[schemars(rename = "received_receipt_pubsub")]
    ReceivedReceiptPubsub(ReceivedReceiptPubsub),
}

impl fmt::Display for NetworkNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            NetworkNotification::NewListenAddr(_) => write!(f, "new_listen_addr"),
            NetworkNotification::ConnnectionEstablished(_) => write!(f, "connection_established"),
            NetworkNotification::ConnnectionClosed(_) => write!(f, "connection_closed"),
            NetworkNotification::OutgoingConnectionError(_) => {
                write!(f, "outgoing_connection_error")
            }
            NetworkNotification::IncomingConnectionError(_) => {
                write!(f, "incoming_connection_error")
            }
            NetworkNotification::DiscoveredMdns(_) => write!(f, "discovered_mdns"),
            NetworkNotification::DiscoveredRendezvous(_) => write!(f, "discovered_rendezvous"),
            NetworkNotification::RegisteredRendezvous(_) => write!(f, "registered_rendezvous"),
            NetworkNotification::DiscoverServedRendezvous(_) => {
                write!(f, "discover_served_rendezvous")
            }
            NetworkNotification::PeerRegisteredRendezvous(_) => {
                write!(f, "peer_registered_rendezvous")
            }
            NetworkNotification::PublishedReceiptPubsub(_) => write!(f, "published_receipt_pubsub"),
            NetworkNotification::ReceivedReceiptPubsub(_) => write!(f, "received_receipt_pubsub"),
        }
    }
}

impl DagJson for NetworkNotification {}

impl From<NetworkNotification> for Ipld {
    fn from(notification: NetworkNotification) -> Self {
        match notification {
            NetworkNotification::NewListenAddr(n) => {
                Ipld::Map(BTreeMap::from([("new_listen_addr".into(), n.into())]))
            }
            NetworkNotification::ConnnectionEstablished(n) => Ipld::Map(BTreeMap::from([(
                "connection_established".into(),
                n.into(),
            )])),
            NetworkNotification::ConnnectionClosed(n) => {
                Ipld::Map(BTreeMap::from([("connection_closed".into(), n.into())]))
            }
            NetworkNotification::OutgoingConnectionError(n) => Ipld::Map(BTreeMap::from([(
                "outgoing_connection_error".into(),
                n.into(),
            )])),
            NetworkNotification::IncomingConnectionError(n) => Ipld::Map(BTreeMap::from([(
                "incoming_connection_error".into(),
                n.into(),
            )])),
            NetworkNotification::DiscoveredMdns(n) => {
                Ipld::Map(BTreeMap::from([("discovered_mdns".into(), n.into())]))
            }
            NetworkNotification::DiscoveredRendezvous(n) => {
                Ipld::Map(BTreeMap::from([("discovered_rendezvous".into(), n.into())]))
            }
            NetworkNotification::RegisteredRendezvous(n) => {
                Ipld::Map(BTreeMap::from([("registered_rendezvous".into(), n.into())]))
            }
            NetworkNotification::DiscoverServedRendezvous(n) => Ipld::Map(BTreeMap::from([(
                "discover_served_rendezvous".into(),
                n.into(),
            )])),
            NetworkNotification::PeerRegisteredRendezvous(n) => Ipld::Map(BTreeMap::from([(
                "peer_registered_rendezvous".into(),
                n.into(),
            )])),
            NetworkNotification::PublishedReceiptPubsub(n) => Ipld::Map(BTreeMap::from([(
                "published_receipt_pubsub".into(),
                n.into(),
            )])),
            NetworkNotification::ReceivedReceiptPubsub(n) => Ipld::Map(BTreeMap::from([(
                "received_receipt_pubsub".into(),
                n.into(),
            )])),
        }
    }
}

impl TryFrom<Ipld> for NetworkNotification {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        if let Some((key, val)) = map.first_key_value() {
            match key.as_str() {
                "new_listen_addr" => Ok(NetworkNotification::NewListenAddr(
                    NewListenAddr::try_from(val.to_owned())?,
                )),
                "connection_established" => Ok(NetworkNotification::ConnnectionEstablished(
                    ConnectionEstablished::try_from(val.to_owned())?,
                )),
                "connection_closed" => Ok(NetworkNotification::ConnnectionClosed(
                    ConnectionClosed::try_from(val.to_owned())?,
                )),
                "outgoing_connection_error" => Ok(NetworkNotification::OutgoingConnectionError(
                    OutgoingConnectionError::try_from(val.to_owned())?,
                )),
                "incoming_connection_error" => Ok(NetworkNotification::IncomingConnectionError(
                    IncomingConnectionError::try_from(val.to_owned())?,
                )),
                "discovered_mdns" => Ok(NetworkNotification::DiscoveredMdns(
                    DiscoveredMdns::try_from(val.to_owned())?,
                )),
                "discovered_rendezvous" => Ok(NetworkNotification::DiscoveredRendezvous(
                    DiscoveredRendezvous::try_from(val.to_owned())?,
                )),
                "registered_rendezvous" => Ok(NetworkNotification::RegisteredRendezvous(
                    RegisteredRendezvous::try_from(val.to_owned())?,
                )),
                "discover_served_rendezvous" => Ok(NetworkNotification::DiscoverServedRendezvous(
                    DiscoverServedRendezvous::try_from(val.to_owned())?,
                )),
                "peer_registered_rendezvous" => Ok(NetworkNotification::PeerRegisteredRendezvous(
                    PeerRegisteredRendezvous::try_from(val.to_owned())?,
                )),
                "published_receipt_pubsub" => Ok(NetworkNotification::PublishedReceiptPubsub(
                    PublishedReceiptPubsub::try_from(val.to_owned())?,
                )),
                "received_receipt_pubsub" => Ok(NetworkNotification::ReceivedReceiptPubsub(
                    ReceivedReceiptPubsub::try_from(val.to_owned())?,
                )),
                _ => Err(anyhow!("Unknown network notification tag type")),
            }
        } else {
            Err(anyhow!("Network notification was an empty map"))
        }
    }
}

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
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

#[derive(Debug, Clone, JsonSchema)]
#[schemars(rename = "published_receipt_pubsub")]
pub struct PublishedReceiptPubsub {
    timestamp: i64,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Ran receipt CID")]
    ran: String,
}

impl PublishedReceiptPubsub {
    pub(crate) fn new(cid: Cid, ran: String) -> PublishedReceiptPubsub {
        PublishedReceiptPubsub {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            ran,
        }
    }
}

impl DagJson for PublishedReceiptPubsub {}

impl From<PublishedReceiptPubsub> for Ipld {
    fn from(notification: PublishedReceiptPubsub) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (RAN_KEY.into(), notification.ran.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for PublishedReceiptPubsub {
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

        Ok(PublishedReceiptPubsub {
            timestamp,
            cid,
            ran,
        })
    }
}

#[derive(Debug, Clone, JsonSchema)]
#[schemars(rename = "received_receipt_pubsub")]
pub struct ReceivedReceiptPubsub {
    timestamp: i64,
    #[schemars(description = "Receipt publisher peer ID")]
    publisher: String,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Ran receipt CID")]
    ran: String,
}

impl ReceivedReceiptPubsub {
    pub(crate) fn new(publisher: PeerId, cid: Cid, ran: String) -> ReceivedReceiptPubsub {
        ReceivedReceiptPubsub {
            timestamp: Utc::now().timestamp_millis(),
            publisher: publisher.to_string(),
            cid: cid.to_string(),
            ran,
        }
    }
}

impl DagJson for ReceivedReceiptPubsub {}

impl From<ReceivedReceiptPubsub> for Ipld {
    fn from(notification: ReceivedReceiptPubsub) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (PUBLISHER_KEY.into(), notification.publisher.into()),
            (CID_KEY.into(), notification.cid.into()),
            (RAN_KEY.into(), notification.ran.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for ReceivedReceiptPubsub {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let publisher = from_ipld(
            map.get(PUBLISHER_KEY)
                .ok_or_else(|| anyhow!("missing {PUBLISHER_KEY}"))?
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

        Ok(ReceivedReceiptPubsub {
            timestamp,
            publisher,
            cid,
            ran,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use homestar_invocation::test_utils::cid::generate_cid;
    use rand::thread_rng;

    #[derive(Clone, Debug)]
    struct Fixtures {
        address: Multiaddr,
        addresses: Vec<Multiaddr>,
        cid: Cid,
        peer_id: PeerId,
        peers: BTreeMap<PeerId, Multiaddr>,
        peers_vec_addr: BTreeMap<PeerId, Vec<Multiaddr>>,
        ran: Cid,
    }

    fn generate_fixtures() -> Fixtures {
        Fixtures {
            address: Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
            addresses: vec![
                Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
                Multiaddr::from_str("/ip4/127.0.0.1/tcp/7001").unwrap(),
            ],
            cid: generate_cid(&mut thread_rng()),
            peer_id: PeerId::random(),
            peers: BTreeMap::from([
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
                ),
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7001").unwrap(),
                ),
            ]),
            peers_vec_addr: BTreeMap::from([
                (
                    PeerId::random(),
                    vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap()],
                ),
                (
                    PeerId::random(),
                    vec![
                        Multiaddr::from_str("/ip4/127.0.0.1/tcp/7001").unwrap(),
                        Multiaddr::from_str("/ip4/127.0.0.1/tcp/7002").unwrap(),
                    ],
                ),
            ]),
            ran: generate_cid(&mut thread_rng()),
        }
    }

    fn generate_notifications(fixtures: Fixtures) -> Vec<(i64, NetworkNotification)> {
        let Fixtures {
            address,
            addresses,
            cid,
            peer_id,
            peers,
            peers_vec_addr,
            ran,
        } = fixtures;

        let new_listen_addr = NewListenAddr::new(peer_id, address.clone());
        let connection_established = ConnectionEstablished::new(peer_id, address.clone());
        let connection_closed = ConnectionClosed::new(peer_id, address.clone());
        let outgoing_connection_error =
            OutgoingConnectionError::new(Some(peer_id), DialError::NoAddresses);
        let incoming_connection_error = IncomingConnectionError::new(ListenError::Aborted);
        let discovered_mdns = DiscoveredMdns::new(peers);
        let discovered_rendezvous = DiscoveredRendezvous::new(peer_id, peers_vec_addr);
        let registered_rendezvous = RegisteredRendezvous::new(peer_id);
        let discover_served_rendezvous = DiscoverServedRendezvous::new(peer_id);
        let peer_registered_rendezvous = PeerRegisteredRendezvous::new(peer_id, addresses);
        let published_receipt_pubsub = PublishedReceiptPubsub::new(cid, ran.to_string());
        let received_receipt_pubsub = ReceivedReceiptPubsub::new(peer_id, cid, ran.to_string());

        vec![
            (
                new_listen_addr.timestamp,
                NetworkNotification::NewListenAddr(new_listen_addr),
            ),
            (
                connection_established.timestamp,
                NetworkNotification::ConnnectionEstablished(connection_established),
            ),
            (
                connection_closed.timestamp,
                NetworkNotification::ConnnectionClosed(connection_closed),
            ),
            (
                outgoing_connection_error.timestamp,
                NetworkNotification::OutgoingConnectionError(outgoing_connection_error),
            ),
            (
                incoming_connection_error.timestamp,
                NetworkNotification::IncomingConnectionError(incoming_connection_error),
            ),
            (
                discovered_mdns.timestamp,
                NetworkNotification::DiscoveredMdns(discovered_mdns),
            ),
            (
                discovered_rendezvous.timestamp,
                NetworkNotification::DiscoveredRendezvous(discovered_rendezvous),
            ),
            (
                registered_rendezvous.timestamp,
                NetworkNotification::RegisteredRendezvous(registered_rendezvous),
            ),
            (
                discover_served_rendezvous.timestamp,
                NetworkNotification::DiscoverServedRendezvous(discover_served_rendezvous),
            ),
            (
                peer_registered_rendezvous.timestamp,
                NetworkNotification::PeerRegisteredRendezvous(peer_registered_rendezvous),
            ),
            (
                published_receipt_pubsub.timestamp,
                NetworkNotification::PublishedReceiptPubsub(published_receipt_pubsub),
            ),
            (
                received_receipt_pubsub.timestamp,
                NetworkNotification::ReceivedReceiptPubsub(received_receipt_pubsub),
            ),
        ]
    }

    fn check_notification(timestamp: i64, notification: NetworkNotification, fixtures: Fixtures) {
        let Fixtures {
            address,
            addresses,
            cid,
            peer_id,
            peers,
            peers_vec_addr,
            ran,
        } = fixtures;

        match notification {
            NetworkNotification::NewListenAddr(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.peer_id).unwrap(), peer_id);
                assert_eq!(Multiaddr::from_str(&n.address).unwrap(), address);
            }
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
            NetworkNotification::OutgoingConnectionError(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(
                    n.peer_id
                        .map_or(None, |p| Some(PeerId::from_str(&p).unwrap())),
                    Some(peer_id)
                );
                assert_eq!(n.error, DialError::NoAddresses.to_string());
            }
            NetworkNotification::IncomingConnectionError(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(n.error, ListenError::Aborted.to_string());
            }
            NetworkNotification::DiscoveredMdns(n) => {
                assert_eq!(n.timestamp, timestamp);

                for peer in n.peers {
                    assert_eq!(
                        Multiaddr::from_str(&peer.1).unwrap(),
                        peers[&PeerId::from_str(&peer.0).unwrap()]
                    )
                }
            }
            NetworkNotification::DiscoveredRendezvous(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.server).unwrap(), peer_id);

                for peer in n.peers {
                    assert_eq!(
                        peer.1
                            .iter()
                            .map(|address| Multiaddr::from_str(address).unwrap())
                            .collect::<Vec<Multiaddr>>(),
                        peers_vec_addr[&PeerId::from_str(&peer.0).unwrap()]
                    )
                }
            }
            NetworkNotification::RegisteredRendezvous(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.server).unwrap(), peer_id);
            }
            NetworkNotification::DiscoverServedRendezvous(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.enquirer).unwrap(), peer_id);
            }
            NetworkNotification::PeerRegisteredRendezvous(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.peer_id).unwrap(), peer_id);
                assert_eq!(
                    n.addresses
                        .iter()
                        .map(|address| Multiaddr::from_str(address).unwrap())
                        .collect::<Vec<Multiaddr>>(),
                    addresses
                );
            }
            NetworkNotification::PublishedReceiptPubsub(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(Cid::from_str(&n.cid).unwrap(), cid);
                assert_eq!(Cid::from_str(&n.ran).unwrap(), ran);
            }
            NetworkNotification::ReceivedReceiptPubsub(n) => {
                assert_eq!(n.timestamp, timestamp);
                assert_eq!(PeerId::from_str(&n.publisher).unwrap(), peer_id);
                assert_eq!(Cid::from_str(&n.cid).unwrap(), cid);
                assert_eq!(Cid::from_str(&n.ran).unwrap(), ran);
            }
        }
    }

    #[test]
    fn notification_bytes_rountrip() {
        let fixtures = generate_fixtures();

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
        let fixtures = generate_fixtures();

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
