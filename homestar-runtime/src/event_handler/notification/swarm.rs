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
            _ => Err(anyhow!("Missing swarm notification type: {}", ty)),
        }
    }
}
