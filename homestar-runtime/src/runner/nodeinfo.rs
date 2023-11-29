//! Node information.

use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DynamicNodeInfo {
    /// Listeners for the node.
    pub(crate) listeners: Vec<Multiaddr>,
}

impl DynamicNodeInfo {
    /// Create an instance of [DynamicNodeInfo].
    pub(crate) fn new(listeners: Vec<Multiaddr>) -> Self {
        Self { listeners }
    }
}
