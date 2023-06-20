//! [libp2p], [websocket], and [ipfs] networking interfaces.
//!
//! [libp2p]: <https://libp2p.io/>
//! [websocket]: ws
//! [ipfs]: ipfs

pub mod eventloop;
#[cfg(feature = "ipfs")]
pub mod ipfs;
pub mod pubsub;
pub mod swarm;
pub mod ws;

pub use eventloop::EventLoop;
