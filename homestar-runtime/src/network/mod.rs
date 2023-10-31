//! [libp2p], multi-use [http] and [websocket] server, and [ipfs] networking
//! interfaces.
//!
//! [libp2p]: libp2p
//! [http]: jsonrpsee::server
//! [websocket]: jsonrpsee::server
//! [ipfs]: ipfs_api

pub(crate) mod error;
#[cfg(feature = "ipfs")]
pub(crate) mod ipfs;
pub(crate) mod pubsub;
pub mod rpc;
pub(crate) mod swarm;
pub(crate) mod webserver;

#[allow(unused_imports)]
pub(crate) use error::Error;
#[cfg(feature = "ipfs")]
pub(crate) use ipfs::IpfsCli;
