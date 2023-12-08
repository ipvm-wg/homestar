//! [libp2p] configuration.

use http::Uri;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds, DurationSeconds};
use std::time::Duration;

/// libp2p settings.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Libp2p {
    /// Multiaddrs of the external addresses this node will announce to the
    /// network.
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub(crate) announce_addresses: Vec<libp2p::Multiaddr>,
    /// Kademlia DHT Settings
    pub(crate) dht: Dht,
    /// Address for [Swarm] to listen on.
    ///
    /// [Swarm]: libp2p::swarm::Swarm
    #[serde(with = "http_serde::uri")]
    pub(crate) listen_address: Uri,
    /// Maximum number of peers we will dial.
    pub(crate) max_connected_peers: u32,
    /// Limit on the number of external addresses we announce to other peers.
    pub(crate) max_announce_addresses: u32,
    /// Multiaddrs of the trusted nodes to connect to on startup.
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub(crate) node_addresses: Vec<libp2p::Multiaddr>,
    /// mDNS Settings.
    pub(crate) mdns: Mdns,
    /// Pubsub Settings.
    pub(crate) pubsub: Pubsub,
    /// Rendezvous Settings.
    pub(crate) rendezvous: Rendezvous,
    /// Transport connection timeout.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) transport_connection_timeout: Duration,
}

/// DHT settings.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Dht {
    /// Timeout for p2p requests for a provided record.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) p2p_provider_timeout: Duration,
    /// Timeout for p2p receipt record lookups in milliseconds.
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub(crate) p2p_receipt_timeout: Duration,
    /// Timeout for p2p workflow info lookups in milliseconds.
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub(crate) p2p_workflow_info_timeout: Duration,
    /// Quorum for receipt records on the DHT.
    pub(crate) receipt_quorum: usize,
    /// Quorum for [workflow::Info] records on the DHT.
    ///
    /// [workflow::Info]: crate::workflow::Info
    pub(crate) workflow_quorum: usize,
}

/// mDNS settings.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Mdns {
    /// Enable mDNS.
    pub(crate) enable: bool,
    /// mDNS IPv6 enable flag
    pub(crate) enable_ipv6: bool,
    /// mDNS query interval.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) query_interval: Duration,
    /// mDNS TTL.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) ttl: Duration,
}

/// Pubsub settings.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Pubsub {
    /// Enable pub/sub.
    pub(crate) enable: bool,
    /// Pub/sub duplicate cache time.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) duplication_cache_time: Duration,
    /// Pub/sub hearbeat interval for mesh configuration.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) heartbeat: Duration,
    /// Pub/sub idle timeout
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) idle_timeout: Duration,
    /// Maximum byte size of pub/sub messages.
    pub(crate) max_transmit_size: usize,
    /// Minimum number of pub/sub peers.
    pub(crate) mesh_n_low: usize,
    /// Maximum number of pub/sub peers.
    pub(crate) mesh_n_high: usize,
    /// Target number of pub/sub peers.
    pub(crate) mesh_n: usize,
    /// Minimum outbound pub/sub peers before adding more peers.
    pub(crate) mesh_outbound_min: usize,
}

/// Rendezvous settings.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Rendezvous {
    /// Enable Rendezvous protocol client.
    pub(crate) enable_client: bool,
    /// Enable Rendezvous protocol server.
    pub(crate) enable_server: bool,
    /// Rendezvous registration TTL.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) registration_ttl: Duration,
    /// Rendezvous discovery interval.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) discovery_interval: Duration,
}

impl Default for Libp2p {
    fn default() -> Self {
        Self {
            announce_addresses: Vec::new(),
            dht: Dht::default(),
            listen_address: Uri::from_static("/ip4/0.0.0.0/tcp/0"),
            max_connected_peers: 32,
            max_announce_addresses: 10,
            mdns: Mdns::default(),
            node_addresses: Vec::new(),
            pubsub: Pubsub::default(),
            rendezvous: Rendezvous::default(),
            transport_connection_timeout: Duration::new(60, 0),
        }
    }
}

impl Libp2p {
    /// DHT settings getter.
    pub(crate) fn dht(&self) -> &Dht {
        &self.dht
    }

    /// Pub/sub settings getter.
    pub(crate) fn pubsub(&self) -> &Pubsub {
        &self.pubsub
    }
}

impl Default for Dht {
    fn default() -> Self {
        Self {
            p2p_provider_timeout: Duration::new(30, 0),
            p2p_receipt_timeout: Duration::from_millis(500),
            p2p_workflow_info_timeout: Duration::from_millis(500),
            receipt_quorum: 2,
            workflow_quorum: 3,
        }
    }
}

impl Default for Mdns {
    fn default() -> Self {
        Self {
            enable: true,
            enable_ipv6: false,
            query_interval: Duration::from_secs(5 * 60),
            ttl: Duration::from_secs(60 * 9),
        }
    }
}

impl Default for Pubsub {
    fn default() -> Self {
        Self {
            enable: true,
            duplication_cache_time: Duration::new(1, 0),
            heartbeat: Duration::new(60, 0),
            idle_timeout: Duration::new(60 * 60 * 24, 0),
            max_transmit_size: 10 * 1024 * 1024,
            mesh_n_low: 1,
            mesh_n_high: 10,
            mesh_n: 2,
            mesh_outbound_min: 1,
        }
    }
}

impl Default for Rendezvous {
    fn default() -> Self {
        Self {
            enable_client: true,
            enable_server: false,
            registration_ttl: Duration::from_secs(2 * 60 * 60),
            discovery_interval: Duration::from_secs(10 * 60),
        }
    }
}
