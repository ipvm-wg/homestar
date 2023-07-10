//! # Error types involving event handling.

use crate::network::swarm::RequestResponseKey;
use anyhow::Result;
use bincode::{Decode, Encode};

/// Error type for messages related to [libp2p::request_response].
#[derive(thiserror::Error, Debug, Encode, Decode)]
pub(crate) enum RequestResponseError {
    /// Return a timeout error when attempting to retrieve data keyed by [Cid].
    ///
    /// [Cid]: libipld::Cid
    #[error("failed to retrieve data keyed by cid {}, tagged with {:?}, within timeout", .0.cid, .0.capsule_tag.tag())]
    Timeout(RequestResponseKey),
    /// Error when attempting to wrap a [Cid] into a [Ipld] capsule.
    ///
    /// [Cid]: libipld::Cid
    /// [Ipld]: libipld::Ipld
    #[error("failed to wrap {} into a Ipld capsule, tagged with {:?}", .0.cid, .0.capsule_tag.tag())]
    InvalidCapsule(RequestResponseKey),
    /// Unsupported message request based on the capsule tag.
    #[error("unsupported message request for tag {:?}, with cid {}", .0.capsule_tag.tag(), .0.cid)]
    Unsupported(RequestResponseKey),
}

impl RequestResponseError {
    /// Encode the error into a byte vector via [bincode].
    pub(crate) fn encode(&self) -> Result<Vec<u8>> {
        bincode::encode_to_vec(self, bincode::config::standard()).map_err(anyhow::Error::msg)
    }

    /// Decode the error from a byte vector via [bincode].
    pub(crate) fn decode(bytes: &[u8]) -> Result<(Self, usize)> {
        bincode::decode_from_slice(bytes, bincode::config::standard()).map_err(anyhow::Error::msg)
    }
}
