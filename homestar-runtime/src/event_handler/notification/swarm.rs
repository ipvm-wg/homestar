// Notification types for [swarm] events.
//
// [swarm]: libp2p_swarm::Swarm

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

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
            "sentWorkflowInfo" => Ok(Self::SentWorkflowInfo),
            "receivedWorkflowInfo" => Ok(Self::ReceivedWorkflowInfo),
            _ => Err(anyhow!("Missing swarm notification type: {}", ty)),
        }
    }
}
