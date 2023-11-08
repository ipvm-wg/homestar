use libp2p::PeerId;
use serde::{Deserialize, Serialize};

/// TODO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NodeInfo {
    pub(crate) peer_id: PeerId,
}

impl NodeInfo {
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
