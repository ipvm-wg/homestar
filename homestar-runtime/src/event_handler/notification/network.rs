//! Notification types for [swarm] events.
//!
//! [swarm]: libp2p::swarm::Swarm

use anyhow::anyhow;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Ipld};
use schemars::JsonSchema;
use std::{collections::BTreeMap, fmt};

pub(crate) mod autonat;
pub(crate) mod connection;
pub(crate) mod dht;
pub(crate) mod mdns;
pub(crate) mod pubsub;
pub(crate) mod rendezvous;
pub(crate) mod req_resp;
pub(crate) use autonat::StatusChangedAutonat;
pub(crate) use connection::{
    ConnectionClosed, ConnectionEstablished, IncomingConnectionError, NewListenAddr,
    OutgoingConnectionError,
};
pub(crate) use dht::{
    GotReceiptDht, GotWorkflowInfoDht, PutReceiptDht, PutWorkflowInfoDht, ReceiptQuorumFailureDht,
    ReceiptQuorumSuccessDht, WorkflowInfoQuorumFailureDht, WorkflowInfoQuorumSuccessDht,
};
pub(crate) use mdns::DiscoveredMdns;
pub(crate) use pubsub::{PublishedReceiptPubsub, ReceivedReceiptPubsub};
pub(crate) use rendezvous::{
    DiscoverServedRendezvous, DiscoveredRendezvous, PeerRegisteredRendezvous, RegisteredRendezvous,
};
pub(crate) use req_resp::{ReceivedWorkflowInfo, SentWorkflowInfo};

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
    /// Outgoing connection error notification.
    #[schemars(rename = "outgoing_connection_error")]
    OutgoingConnectionError(OutgoingConnectionError),
    /// Incoming connection error notification.
    #[schemars(rename = "incoming_connection_error")]
    IncomingConnectionError(IncomingConnectionError),
    /// Autonat status changed notification.
    #[schemars(rename = "status_changed_autonat")]
    StatusChangedAutonat(StatusChangedAutonat),
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
    /// Put receipt DHT notification.
    #[schemars(rename = "put_receipt_dht")]
    PutReceiptDht(PutReceiptDht),
    /// Got receipt DHT notification.
    #[schemars(rename = "got_receipt_dht")]
    GotReceiptDht(GotReceiptDht),
    /// Put workflow info DHT notification.
    #[schemars(rename = "put_workflow_info_dht")]
    PutWorkflowInfoDht(PutWorkflowInfoDht),
    /// Put workflow info DHT notification.
    #[schemars(rename = "got_workflow_info_dht")]
    GotWorkflowInfoDht(GotWorkflowInfoDht),
    /// Receipt quorum success notification.
    #[schemars(rename = "receipt_quorum_success_dht")]
    ReceiptQuorumSuccessDht(ReceiptQuorumSuccessDht),
    /// Receipt quorum failure notification.
    #[schemars(rename = "receipt_quorum_failure_dht")]
    ReceiptQuorumFailureDht(ReceiptQuorumFailureDht),
    /// Workflow info quorum success notification.
    #[schemars(rename = "workflow_info_quorum_success_dht")]
    WorkflowInfoQuorumSuccessDht(WorkflowInfoQuorumSuccessDht),
    /// Workflow info quorum failure notification.
    #[schemars(rename = "workflow_info_quorum_failure_dht")]
    WorkflowInfoQuorumFailureDht(WorkflowInfoQuorumFailureDht),
    /// Sent workflow info notification.
    #[schemars(rename = "sent_workflow_info")]
    SentWorkflowInfo(SentWorkflowInfo),
    /// Received workflow info notification.
    #[schemars(rename = "received_workflow_info")]
    ReceivedWorkflowInfo(ReceivedWorkflowInfo),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum WorkflowInfoSource {
    Dht,
    RequestResponse,
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
            NetworkNotification::StatusChangedAutonat(_) => write!(f, "status_changed_autonat"),
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
            NetworkNotification::PutReceiptDht(_) => write!(f, "put_receipt_dht"),
            NetworkNotification::GotReceiptDht(_) => write!(f, "got_receipt_dht"),
            NetworkNotification::PutWorkflowInfoDht(_) => write!(f, "put_workflow_info_dht"),
            NetworkNotification::GotWorkflowInfoDht(_) => write!(f, "got_workflow_info_dht"),
            NetworkNotification::ReceiptQuorumSuccessDht(_) => {
                write!(f, "receipt_quorum_success_dht")
            }
            NetworkNotification::ReceiptQuorumFailureDht(_) => {
                write!(f, "receipt_quorum_failure_dht")
            }
            NetworkNotification::WorkflowInfoQuorumSuccessDht(_) => {
                write!(f, "workflow_info_quorum_success_dht")
            }
            NetworkNotification::WorkflowInfoQuorumFailureDht(_) => {
                write!(f, "workflow_info_quorum_failure_dht")
            }
            NetworkNotification::SentWorkflowInfo(_) => {
                write!(f, "sent_workflow_info")
            }
            NetworkNotification::ReceivedWorkflowInfo(_) => {
                write!(f, "received_workflow_info")
            }
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
            NetworkNotification::StatusChangedAutonat(n) => Ipld::Map(BTreeMap::from([(
                "status_changed_autonat".into(),
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
            NetworkNotification::PutReceiptDht(n) => {
                Ipld::Map(BTreeMap::from([("put_receipt_dht".into(), n.into())]))
            }
            NetworkNotification::GotReceiptDht(n) => {
                Ipld::Map(BTreeMap::from([("got_receipt_dht".into(), n.into())]))
            }
            NetworkNotification::PutWorkflowInfoDht(n) => {
                Ipld::Map(BTreeMap::from([("put_workflow_info_dht".into(), n.into())]))
            }
            NetworkNotification::GotWorkflowInfoDht(n) => {
                Ipld::Map(BTreeMap::from([("got_workflow_info_dht".into(), n.into())]))
            }
            NetworkNotification::ReceiptQuorumSuccessDht(n) => Ipld::Map(BTreeMap::from([(
                "receipt_quorum_success_dht".into(),
                n.into(),
            )])),
            NetworkNotification::ReceiptQuorumFailureDht(n) => Ipld::Map(BTreeMap::from([(
                "receipt_quorum_failure_dht".into(),
                n.into(),
            )])),
            NetworkNotification::WorkflowInfoQuorumSuccessDht(n) => Ipld::Map(BTreeMap::from([(
                "workflow_info_quorum_success_dht".into(),
                n.into(),
            )])),
            NetworkNotification::WorkflowInfoQuorumFailureDht(n) => Ipld::Map(BTreeMap::from([(
                "workflow_info_quorum_failure_dht".into(),
                n.into(),
            )])),
            NetworkNotification::SentWorkflowInfo(n) => {
                Ipld::Map(BTreeMap::from([("sent_workflow_info".into(), n.into())]))
            }
            NetworkNotification::ReceivedWorkflowInfo(n) => Ipld::Map(BTreeMap::from([(
                "received_workflow_info".into(),
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
                "status_changed_autonat" => Ok(NetworkNotification::StatusChangedAutonat(
                    StatusChangedAutonat::try_from(val.to_owned())?,
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
                "put_receipt_dht" => Ok(NetworkNotification::PutReceiptDht(
                    PutReceiptDht::try_from(val.to_owned())?,
                )),
                "got_receipt_dht" => Ok(NetworkNotification::GotReceiptDht(
                    GotReceiptDht::try_from(val.to_owned())?,
                )),
                "put_workflow_info_dht" => Ok(NetworkNotification::PutWorkflowInfoDht(
                    PutWorkflowInfoDht::try_from(val.to_owned())?,
                )),
                "got_workflow_info_dht" => Ok(NetworkNotification::GotWorkflowInfoDht(
                    GotWorkflowInfoDht::try_from(val.to_owned())?,
                )),
                "receipt_quorum_success_dht" => Ok(NetworkNotification::ReceiptQuorumSuccessDht(
                    ReceiptQuorumSuccessDht::try_from(val.to_owned())?,
                )),
                "receipt_quorum_failure_dht" => Ok(NetworkNotification::ReceiptQuorumFailureDht(
                    ReceiptQuorumFailureDht::try_from(val.to_owned())?,
                )),
                "workflow_info_quorum_success_dht" => {
                    Ok(NetworkNotification::WorkflowInfoQuorumSuccessDht(
                        WorkflowInfoQuorumSuccessDht::try_from(val.to_owned())?,
                    ))
                }
                "workflow_info_quorum_failure_dht" => {
                    Ok(NetworkNotification::WorkflowInfoQuorumFailureDht(
                        WorkflowInfoQuorumFailureDht::try_from(val.to_owned())?,
                    ))
                }
                "sent_workflow_info" => Ok(NetworkNotification::SentWorkflowInfo(
                    SentWorkflowInfo::try_from(val.to_owned())?,
                )),
                "received_workflow_info" => Ok(NetworkNotification::ReceivedWorkflowInfo(
                    ReceivedWorkflowInfo::try_from(val.to_owned())?,
                )),
                _ => Err(anyhow!("Unknown network notification tag type")),
            }
        } else {
            Err(anyhow!("Network notification was an empty map"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::libp2p::nat_status::NatStatusExt;
    use faststr::FastStr;
    use homestar_invocation::test_utils::cid::generate_cid;
    use libipld::Cid;
    use libp2p::{
        autonat::NatStatus,
        swarm::{DialError, ListenError},
        Multiaddr, PeerId,
    };
    use rand::thread_rng;
    use std::str::FromStr;

    #[derive(Clone, Debug)]
    struct Fixtures {
        address: Multiaddr,
        addresses: Vec<Multiaddr>,
        cid: Cid,
        connected_peer_count: usize,
        name: FastStr,
        nat_status: NatStatus,
        num_tasks: u32,
        peer_id: PeerId,
        peers: Vec<PeerId>,
        peers_map: BTreeMap<PeerId, Multiaddr>,
        peers_map_vec_addr: BTreeMap<PeerId, Vec<Multiaddr>>,
        progress: Vec<Cid>,
        progress_count: u32,
        quorum: usize,
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
            connected_peer_count: 1,
            name: FastStr::new("Strong Bad"),
            nat_status: NatStatus::Public(Multiaddr::from_str("/ip4/127.0.0.1/tcp/7002").unwrap()),
            num_tasks: 1,
            peer_id: PeerId::random(),
            peers: vec![PeerId::random(), PeerId::random()],
            peers_map: BTreeMap::from([
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7000").unwrap(),
                ),
                (
                    PeerId::random(),
                    Multiaddr::from_str("/ip4/127.0.0.1/tcp/7001").unwrap(),
                ),
            ]),
            peers_map_vec_addr: BTreeMap::from([
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
            progress: vec![generate_cid(&mut thread_rng())],
            progress_count: 1,
            quorum: 3,
            ran: generate_cid(&mut thread_rng()),
        }
    }

    fn generate_notifications(fixtures: Fixtures) -> Vec<(i64, NetworkNotification)> {
        let Fixtures {
            address,
            addresses,
            cid,
            connected_peer_count,
            name,
            nat_status,
            num_tasks,
            peer_id,
            peers,
            peers_map,
            peers_map_vec_addr,
            progress,
            progress_count,
            quorum,
            ran,
        } = fixtures;

        let new_listen_addr = NewListenAddr::new(peer_id, address.clone());
        let connection_established = ConnectionEstablished::new(peer_id, address.clone());
        let connection_closed = ConnectionClosed::new(peer_id, address.clone());
        let outgoing_connection_error =
            OutgoingConnectionError::new(Some(peer_id), DialError::NoAddresses);
        let incoming_connection_error = IncomingConnectionError::new(ListenError::Aborted);
        let status_changed_autonat = StatusChangedAutonat::new(nat_status);
        let discovered_mdns = DiscoveredMdns::new(peers_map);
        let discovered_rendezvous = DiscoveredRendezvous::new(peer_id, peers_map_vec_addr);
        let registered_rendezvous = RegisteredRendezvous::new(peer_id);
        let discover_served_rendezvous = DiscoverServedRendezvous::new(peer_id);
        let peer_registered_rendezvous = PeerRegisteredRendezvous::new(peer_id, addresses);
        let published_receipt_pubsub = PublishedReceiptPubsub::new(cid, ran.to_string());
        let received_receipt_pubsub = ReceivedReceiptPubsub::new(peer_id, cid, ran.to_string());
        let put_receipt_dht = PutReceiptDht::new(cid, ran.to_string());
        let got_receipt_dht = GotReceiptDht::new(Some(peer_id), cid, ran.to_string());
        let put_workflow_info_dht = PutWorkflowInfoDht::new(
            cid,
            Some(name.clone()),
            num_tasks,
            progress.clone(),
            progress_count,
        );
        let got_workflow_info_dht = GotWorkflowInfoDht::new(
            Some(peer_id),
            cid,
            Some(name.clone()),
            num_tasks,
            progress.clone(),
            progress_count,
        );
        let receipt_quorum_success_dht =
            ReceiptQuorumSuccessDht::new(FastStr::new(cid.to_string()), quorum);
        let receipt_quorum_failure_dht = ReceiptQuorumFailureDht::new(
            FastStr::new(cid.to_string()),
            quorum,
            connected_peer_count,
            peers.clone(),
        );
        let workflow_info_quorum_success_dht =
            WorkflowInfoQuorumSuccessDht::new(FastStr::new(cid.to_string()), quorum);
        let workflow_info_quorum_failure_dht = WorkflowInfoQuorumFailureDht::new(
            FastStr::new(cid.to_string()),
            quorum,
            connected_peer_count,
            peers,
        );
        let sent_workflow_info = SentWorkflowInfo::new(
            peer_id,
            cid,
            Some(name.clone()),
            num_tasks,
            progress.clone(),
            progress_count,
        );
        let received_workflow_info = ReceivedWorkflowInfo::new(
            Some(peer_id),
            cid,
            Some(name),
            num_tasks,
            progress,
            progress_count,
        );

        vec![
            (
                new_listen_addr.timestamp().to_owned(),
                NetworkNotification::NewListenAddr(new_listen_addr),
            ),
            (
                connection_established.timestamp().to_owned(),
                NetworkNotification::ConnnectionEstablished(connection_established),
            ),
            (
                connection_closed.timestamp().to_owned(),
                NetworkNotification::ConnnectionClosed(connection_closed),
            ),
            (
                outgoing_connection_error.timestamp().to_owned(),
                NetworkNotification::OutgoingConnectionError(outgoing_connection_error),
            ),
            (
                incoming_connection_error.timestamp().to_owned(),
                NetworkNotification::IncomingConnectionError(incoming_connection_error),
            ),
            (
                status_changed_autonat.timestamp().to_owned(),
                NetworkNotification::StatusChangedAutonat(status_changed_autonat),
            ),
            (
                discovered_mdns.timestamp().to_owned(),
                NetworkNotification::DiscoveredMdns(discovered_mdns),
            ),
            (
                discovered_rendezvous.timestamp().to_owned(),
                NetworkNotification::DiscoveredRendezvous(discovered_rendezvous),
            ),
            (
                registered_rendezvous.timestamp().to_owned(),
                NetworkNotification::RegisteredRendezvous(registered_rendezvous),
            ),
            (
                discover_served_rendezvous.timestamp().to_owned(),
                NetworkNotification::DiscoverServedRendezvous(discover_served_rendezvous),
            ),
            (
                peer_registered_rendezvous.timestamp().to_owned(),
                NetworkNotification::PeerRegisteredRendezvous(peer_registered_rendezvous),
            ),
            (
                published_receipt_pubsub.timestamp().to_owned(),
                NetworkNotification::PublishedReceiptPubsub(published_receipt_pubsub),
            ),
            (
                received_receipt_pubsub.timestamp().to_owned(),
                NetworkNotification::ReceivedReceiptPubsub(received_receipt_pubsub),
            ),
            (
                put_receipt_dht.timestamp().to_owned(),
                NetworkNotification::PutReceiptDht(put_receipt_dht),
            ),
            (
                got_receipt_dht.timestamp().to_owned(),
                NetworkNotification::GotReceiptDht(got_receipt_dht),
            ),
            (
                put_workflow_info_dht.timestamp().to_owned(),
                NetworkNotification::PutWorkflowInfoDht(put_workflow_info_dht),
            ),
            (
                got_workflow_info_dht.timestamp().to_owned(),
                NetworkNotification::GotWorkflowInfoDht(got_workflow_info_dht),
            ),
            (
                receipt_quorum_success_dht.timestamp().to_owned(),
                NetworkNotification::ReceiptQuorumSuccessDht(receipt_quorum_success_dht),
            ),
            (
                receipt_quorum_failure_dht.timestamp().to_owned(),
                NetworkNotification::ReceiptQuorumFailureDht(receipt_quorum_failure_dht),
            ),
            (
                workflow_info_quorum_success_dht.timestamp().to_owned(),
                NetworkNotification::WorkflowInfoQuorumSuccessDht(workflow_info_quorum_success_dht),
            ),
            (
                workflow_info_quorum_failure_dht.timestamp().to_owned(),
                NetworkNotification::WorkflowInfoQuorumFailureDht(workflow_info_quorum_failure_dht),
            ),
            (
                sent_workflow_info.timestamp().to_owned(),
                NetworkNotification::SentWorkflowInfo(sent_workflow_info),
            ),
            (
                received_workflow_info.timestamp().to_owned(),
                NetworkNotification::ReceivedWorkflowInfo(received_workflow_info),
            ),
        ]
    }

    fn check_notification(timestamp: &i64, notification: NetworkNotification, fixtures: Fixtures) {
        let Fixtures {
            address,
            addresses,
            cid,
            connected_peer_count,
            name,
            nat_status,
            num_tasks,
            peer_id,
            peers,
            peers_map,
            peers_map_vec_addr,
            progress,
            progress_count,
            quorum,
            ran,
        } = fixtures;

        match notification {
            NetworkNotification::NewListenAddr(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.peer_id()).unwrap(), peer_id);
                assert_eq!(Multiaddr::from_str(n.address()).unwrap(), address);
            }
            NetworkNotification::ConnnectionEstablished(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.peer_id()).unwrap(), peer_id);
                assert_eq!(Multiaddr::from_str(n.address()).unwrap(), address);
            }
            NetworkNotification::ConnnectionClosed(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.peer_id()).unwrap(), peer_id);
                assert_eq!(Multiaddr::from_str(n.address()).unwrap(), address);
            }
            NetworkNotification::OutgoingConnectionError(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(
                    n.peer_id().as_ref().map(|p| PeerId::from_str(&p).unwrap()),
                    Some(peer_id)
                );
                assert_eq!(n.error().to_string(), DialError::NoAddresses.to_string());
            }
            NetworkNotification::IncomingConnectionError(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(n.error().to_string(), ListenError::Aborted.to_string());
            }
            NetworkNotification::StatusChangedAutonat(n) => {
                let (status, address) = nat_status.to_tuple();

                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(n.status(), &status);
                assert_eq!(
                    n.address()
                        .as_ref()
                        .map(|a| Multiaddr::from_str(&a).unwrap()),
                    address
                );
            }
            NetworkNotification::DiscoveredMdns(n) => {
                assert_eq!(n.timestamp(), timestamp);

                for peer in n.peers() {
                    assert_eq!(
                        Multiaddr::from_str(&peer.1).unwrap(),
                        peers_map[&PeerId::from_str(&peer.0).unwrap()]
                    )
                }
            }
            NetworkNotification::DiscoveredRendezvous(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.server()).unwrap(), peer_id);

                for peer in n.peers() {
                    assert_eq!(
                        peer.1
                            .iter()
                            .map(|address| Multiaddr::from_str(address).unwrap())
                            .collect::<Vec<Multiaddr>>(),
                        peers_map_vec_addr[&PeerId::from_str(&peer.0).unwrap()]
                    )
                }
            }
            NetworkNotification::RegisteredRendezvous(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.server()).unwrap(), peer_id);
            }
            NetworkNotification::DiscoverServedRendezvous(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.enquirer()).unwrap(), peer_id);
            }
            NetworkNotification::PeerRegisteredRendezvous(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.peer_id()).unwrap(), peer_id);
                assert_eq!(
                    n.addresses()
                        .iter()
                        .map(|address| Multiaddr::from_str(address).unwrap())
                        .collect::<Vec<Multiaddr>>(),
                    addresses
                );
            }
            NetworkNotification::PublishedReceiptPubsub(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(Cid::from_str(n.cid()).unwrap(), cid);
                assert_eq!(Cid::from_str(n.ran()).unwrap(), ran);
            }
            NetworkNotification::ReceivedReceiptPubsub(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(n.publisher()).unwrap(), peer_id);
                assert_eq!(Cid::from_str(n.cid()).unwrap(), cid);
                assert_eq!(Cid::from_str(n.ran()).unwrap(), ran);
            }
            NetworkNotification::PutReceiptDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(Cid::from_str(n.cid()).unwrap(), cid);
                assert_eq!(Cid::from_str(n.ran()).unwrap(), ran);
            }
            NetworkNotification::GotReceiptDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(
                    n.publisher()
                        .as_ref()
                        .map(|p| PeerId::from_str(&p).unwrap()),
                    Some(peer_id)
                );
                assert_eq!(Cid::from_str(n.cid()).unwrap(), cid);
                assert_eq!(Cid::from_str(n.ran()).unwrap(), ran);
            }
            NetworkNotification::PutWorkflowInfoDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(Cid::from_str(n.cid()).unwrap(), cid);
                assert_eq!(n.name().as_ref().map(|name| FastStr::new(name)), Some(name));
                assert_eq!(n.num_tasks(), &num_tasks);
                assert_eq!(
                    n.progress()
                        .iter()
                        .map(|cid| Cid::from_str(&cid).unwrap())
                        .collect::<Vec<Cid>>(),
                    progress
                );
                assert_eq!(n.progress_count(), &progress_count);
            }
            NetworkNotification::GotWorkflowInfoDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(
                    n.publisher()
                        .as_ref()
                        .map(|p| PeerId::from_str(&p).unwrap()),
                    Some(peer_id)
                );
                assert_eq!(Cid::from_str(&n.cid()).unwrap(), cid);
                assert_eq!(n.name().as_ref().map(|name| FastStr::new(name)), Some(name));
                assert_eq!(n.num_tasks(), &num_tasks);
                assert_eq!(
                    n.progress()
                        .iter()
                        .map(|cid| Cid::from_str(&cid).unwrap())
                        .collect::<Vec<Cid>>(),
                    progress
                );
                assert_eq!(n.progress_count(), &progress_count);
            }
            NetworkNotification::ReceiptQuorumSuccessDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(FastStr::new(n.cid()), FastStr::new(cid.to_string()));
                assert_eq!(n.quorum(), &quorum);
            }
            NetworkNotification::ReceiptQuorumFailureDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(FastStr::new(n.cid()), FastStr::new(cid.to_string()));
                assert_eq!(n.quorum(), &quorum);
                assert_eq!(n.connected_peer_count(), &connected_peer_count);
                assert_eq!(
                    n.stored_to_peers()
                        .iter()
                        .map(|p| PeerId::from_str(p).unwrap())
                        .collect::<Vec<PeerId>>(),
                    peers
                );
            }
            NetworkNotification::WorkflowInfoQuorumSuccessDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(FastStr::new(n.cid()), FastStr::new(cid.to_string()));
                assert_eq!(n.quorum(), &quorum);
            }
            NetworkNotification::WorkflowInfoQuorumFailureDht(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(FastStr::new(n.cid()), FastStr::new(cid.to_string()));
                assert_eq!(n.quorum(), &quorum);
                assert_eq!(n.connected_peer_count(), &connected_peer_count);
                assert_eq!(
                    n.stored_to_peers()
                        .iter()
                        .map(|p| PeerId::from_str(p).unwrap())
                        .collect::<Vec<PeerId>>(),
                    peers
                );
            }
            NetworkNotification::SentWorkflowInfo(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(PeerId::from_str(&n.requestor()).unwrap(), peer_id);
                assert_eq!(Cid::from_str(n.cid()).unwrap(), cid);
                assert_eq!(n.name().as_ref().map(|name| FastStr::new(name)), Some(name));
                assert_eq!(n.num_tasks(), &num_tasks);
                assert_eq!(
                    n.progress()
                        .iter()
                        .map(|cid| Cid::from_str(&cid).unwrap())
                        .collect::<Vec<Cid>>(),
                    progress
                );
                assert_eq!(n.progress_count(), &progress_count);
            }
            NetworkNotification::ReceivedWorkflowInfo(n) => {
                assert_eq!(n.timestamp(), timestamp);
                assert_eq!(
                    n.provider().as_ref().map(|p| PeerId::from_str(&p).unwrap()),
                    Some(peer_id)
                );
                assert_eq!(Cid::from_str(n.cid()).unwrap(), cid);
                assert_eq!(n.name().as_ref().map(|name| FastStr::new(name)), Some(name));
                assert_eq!(n.num_tasks(), &num_tasks);
                assert_eq!(
                    n.progress()
                        .iter()
                        .map(|cid| Cid::from_str(&cid).unwrap())
                        .collect::<Vec<Cid>>(),
                    progress
                );
                assert_eq!(n.progress_count(), &progress_count);
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
                &timestamp,
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
                &timestamp,
                NetworkNotification::from_json_string(json).unwrap(),
                fixtures.clone(),
            )
        }
    }
}
