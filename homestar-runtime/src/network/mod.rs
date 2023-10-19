//! [libp2p], [websocket], and [ipfs] networking interfaces.
//!
//! [libp2p]: libp2p
//! [websocket]: axum::extract::ws
//! [ipfs]: ipfs_api

pub(crate) mod error;
#[cfg(feature = "ipfs")]
pub(crate) mod ipfs;
pub(crate) mod pubsub;
pub mod rpc;
pub(crate) mod swarm;
#[cfg(feature = "websocket-server")]
pub(crate) mod ws;

#[cfg(feature = "ipfs")]
pub(crate) use ipfs::IpfsCli;
