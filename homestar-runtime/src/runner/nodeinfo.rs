//! Node information.

use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node information.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StaticNodeInfo {
    /// The [PeerId] of a node.
    pub(crate) peer_id: PeerId,
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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct DynamicNodeInfo {
    /// Listeners for the node.
    pub(crate) listeners: Vec<Multiaddr>,
    /// Connections for the node.
    pub(crate) connections: HashMap<PeerId, Multiaddr>,
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
