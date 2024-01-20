//! libp2p, multi-use [HTTP] and [WebSocket] server, and [ipfs] networking
//! interfaces.
//!
//! [HTTP]: jsonrpsee::server
//! [WebSocket]: jsonrpsee::server
//! [ipfs]: ipfs_api

pub(crate) mod error;
#[cfg(feature = "ipfs")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
pub(crate) mod ipfs;
pub(crate) mod pubsub;
pub mod rpc;
pub(crate) mod swarm;
pub(crate) mod webserver;

#[allow(unused_imports)]
pub(crate) use error::Error;
#[cfg(feature = "ipfs")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
pub(crate) use ipfs::IpfsCli;
