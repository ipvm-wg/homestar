use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

/// TODO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StaticNodeInfo {
    pub(crate) peer_id: PeerId,
}

impl StaticNodeInfo {
    /// TODO
    pub(crate) fn new(peer_id: PeerId) -> Self {
        Self { peer_id }
    }

    /// TODO
    #[allow(dead_code)]
    pub(crate) fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }
}

/// TODO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DynamicNodeInfo {
    pub(crate) listeners: Vec<Multiaddr>,
}

impl DynamicNodeInfo {
    /// TODO
    pub(crate) fn new(listeners: Vec<Multiaddr>) -> Self {
        Self { listeners }
    }
}
