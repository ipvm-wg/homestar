//! [libp2p], [websocket], and [ipfs] networking interfaces.
//!
//! [libp2p]: <https://libp2p.io/>
//! [websocket]: ws
//! [ipfs]: ipfs

#[cfg(feature = "ipfs")]
pub(crate) mod ipfs;
pub(crate) mod pubsub;
pub(crate) mod swarm;
#[cfg(feature = "websocket-server")]
pub mod ws;

#[cfg(feature = "ipfs")]
pub(crate) use ipfs::IpfsCli;
