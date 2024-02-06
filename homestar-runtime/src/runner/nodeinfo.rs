//! Node information.

use libp2p::{Multiaddr, PeerId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
use tabled::Tabled;

/// Node information.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "node_info")]
pub struct NodeInfo {
    /// Static node information available at startup.
    #[serde(rename = "static")]
    pub(crate) stat: StaticNodeInfo,
    /// Dynamic node information available through events
    /// at runtime.
    pub(crate) dynamic: DynamicNodeInfo,
}

impl NodeInfo {
    /// Create an instance of [NodeInfo].
    pub(crate) fn new(stat: StaticNodeInfo, dynamic: DynamicNodeInfo) -> Self {
        Self { stat, dynamic }
    }
}

/// Static node information available at startup.
#[derive(Debug, Clone, Serialize, Deserialize, Tabled, JsonSchema)]
#[schemars(rename = "static")]
pub(crate) struct StaticNodeInfo {
    /// The [PeerId] of a node.
    #[schemars(with = "String", description = "The peer ID of the node")]
    pub(crate) peer_id: PeerId,
}

impl fmt::Display for StaticNodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "peer_id: {}", self.peer_id)
    }
}

impl StaticNodeInfo {
    /// Create an instance of [StaticNodeInfo].
    pub(crate) fn new(peer_id: PeerId) -> Self {
        Self { peer_id }
    }

    /// Get a reference to the [PeerId] of a node.
    #[allow(dead_code)]
    pub(crate) fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }
}

/// Dynamic node information available through events
/// at runtime.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "dynamic")]
pub(crate) struct DynamicNodeInfo {
    /// Listeners for the node.
    #[schemars(with = "Vec<String>", description = "Listen addresses for the node")]
    pub(crate) listeners: Vec<Multiaddr>,
    /// Connections for the node.
    #[schemars(
        with = "HashMap<String, String>",
        description = "Peers and their addresses that are connected to the node"
    )]
    pub(crate) connections: HashMap<PeerId, Multiaddr>,
}

impl fmt::Display for DynamicNodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "listeners: {:?}, connections: {:?}",
            self.listeners, self.connections
        )
    }
}

impl DynamicNodeInfo {
    /// Create an instance of [DynamicNodeInfo].
    pub(crate) fn new(listeners: Vec<Multiaddr>, connections: HashMap<PeerId, Multiaddr>) -> Self {
        Self {
            listeners,
            connections,
        }
    }
}
