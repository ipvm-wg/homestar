//! Settings / Configuration.

use config::{Config, ConfigError, Environment, File};
use http::Uri;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, DurationMilliSeconds, DurationSeconds};
#[cfg(feature = "ipfs")]
use std::net::Ipv4Addr;
use std::{
    env,
    net::{IpAddr, Ipv6Addr},
    path::PathBuf,
    time::Duration,
};

mod pubkey_config;
pub(crate) use pubkey_config::PubkeyConfig;

#[cfg(target_os = "windows")]
const HOME_VAR: &str = "USERPROFILE";
#[cfg(not(target_os = "windows"))]
const HOME_VAR: &str = "HOME";

/// Application settings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(default)]
    pub(crate) node: Node,
}

impl Settings {
    /// Node settings getter.
    pub fn node(&self) -> &Node {
        &self.node
    }
}

/// Monitoring settings.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Monitoring {
    /// Tokio console port.
    pub console_subscriber_port: u16,
    /// Monitoring collection interval in milliseconds.
    #[cfg(feature = "monitoring")]
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub process_collector_interval: Duration,
}

/// Server settings.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Node {
    /// Monitoring settings.
    #[serde(default)]
    pub(crate) monitoring: Monitoring,
    /// Network settings.
    #[serde(default)]
    pub(crate) network: Network,
    /// Database settings.
    #[serde(default)]
    pub(crate) db: Database,
    /// Garbage collection interval.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) gc_interval: Duration,
    /// Shutdown timeout.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) shutdown_timeout: Duration,
}

/// Network-related settings for a homestar node.
/// TODO: Split-up and re-arrange.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Network {
    /// Metrics port for prometheus scraping.
    pub metrics_port: u16,
    /// Buffer-length for event(s) / command(s) channels.
    pub(crate) events_buffer_len: usize,
    /// Address for [Swarm] to listen on.
    ///
    /// [Swarm]: libp2p::swarm::Swarm
    #[serde(with = "http_serde::uri")]
    pub(crate) listen_address: Uri,
    /// Enable Rendezvous protocol client.
    pub(crate) enable_rendezvous_client: bool,
    /// Enable Rendezvous protocol server.
    pub(crate) enable_rendezvous_server: bool,
    /// Rendezvous registration TTL.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) rendezvous_registration_ttl: Duration,
    /// Rendezvous discovery interval.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) rendezvous_discovery_interval: Duration,
    /// Enable mDNS.
    pub(crate) enable_mdns: bool,
    /// mDNS IPv6 enable flag
    pub(crate) mdns_enable_ipv6: bool,
    /// mDNS query interval.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) mdns_query_interval: Duration,
    /// mDNS TTL.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) mdns_ttl: Duration,
    /// Timeout for p2p requests for a provided record.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) p2p_provider_timeout: Duration,
    /// Enable pub/sub.
    pub(crate) enable_pubsub: bool,
    /// Pub/sub duplicate cache time.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) pubsub_duplication_cache_time: Duration,
    /// Pub/sub hearbeat interval for mesh configuration.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) pubsub_heartbeat: Duration,
    /// Pub/sub idle timeout
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) pubsub_idle_timeout: Duration,
    /// TODO
    pub(crate) pubsub_max_transmit_size: usize,
    /// TODO
    pub(crate) pubsub_mesh_n_low: usize,
    /// TODO
    pub(crate) pubsub_mesh_n_high: usize,
    /// TODO
    pub(crate) pubsub_mesh_n: usize,
    /// TODO
    pub(crate) pubsub_mesh_outbound_min: usize,
    /// Quorum for receipt records on the DHT.
    pub(crate) receipt_quorum: usize,
    /// RPC-server port.
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) rpc_host: IpAddr,
    /// RPC-server max-concurrent connections.
    pub(crate) rpc_max_connections: usize,
    /// RPC-server port.
    pub(crate) rpc_port: u16,
    #[serde_as(as = "DurationSeconds<u64>")]
    /// RPC-server timeout.
    pub(crate) rpc_server_timeout: Duration,
    /// Transport connection timeout.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) transport_connection_timeout: Duration,
    /// Webserver host address.
    #[serde(with = "http_serde::uri")]
    pub(crate) webserver_host: Uri,
    /// Webserver-server port.
    pub(crate) webserver_port: u16,
    /// TODO
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) webserver_timeout: Duration,
    /// Number of *bounded* clients to send messages to, used for a
    /// [tokio::sync::broadcast::channel]
    pub(crate) websocket_capacity: usize,
    /// Websocket-server timeout for receiving messages from the runner.
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub(crate) websocket_receiver_timeout: Duration,
    /// Quorum for [workflow::Info] records on the DHT.
    ///
    /// [workflow::Info]: crate::workflow::Info
    pub(crate) workflow_quorum: usize,
    /// Pubkey setup configuration.
    pub(crate) keypair_config: PubkeyConfig,
    /// Multiaddrs of the trusted nodes to connect to on startup.
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub(crate) node_addresses: Vec<libp2p::Multiaddr>,
    /// Multiaddrs of the external addresses this node will announce to the
    /// network.
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub(crate) announce_addresses: Vec<libp2p::Multiaddr>,
    /// Maximum number of peers we will dial.
    pub(crate) max_connected_peers: u32,
    /// Limit on the number of external addresses we announce to other peers.
    pub(crate) max_announce_addresses: u32,
    /// Event handler poll cache interval in milliseconds.
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub(crate) poll_cache_interval: Duration,
    /// TODO
    #[cfg(feature = "ipfs")]
    pub(crate) ipfs: Ipfs,
}

#[cfg(feature = "ipfs")]
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Ipfs {
    /// TODO
    pub(crate) host: String,
    /// TODO
    pub(crate) port: u16,
}

/// Database-related settings for a homestar node.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Database {
    /// Database Url provided within the configuration file.
    ///
    /// Note: This is not used if the `DATABASE_URL` environment variable
    /// is set.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub(crate) url: Option<String>,
    /// Maximum number of connections managed by the [pool].
    ///
    /// [pool]: crate::db::Pool
    pub(crate) max_pool_size: u32,
}

#[cfg(feature = "monitoring")]
impl Default for Monitoring {
    fn default() -> Self {
        Self {
            process_collector_interval: Duration::from_millis(5000),
            console_subscriber_port: 6669,
        }
    }
}

#[cfg(not(feature = "monitoring"))]
impl Default for Monitoring {
    fn default() -> Self {
        Self {
            console_subscriber_port: 6669,
        }
    }
}

#[cfg(feature = "ipfs")]
impl Default for Ipfs {
    fn default() -> Self {
        Self {
            host: Ipv4Addr::LOCALHOST.to_string(),
            port: 5001,
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            max_pool_size: 100,
            url: None,
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            gc_interval: Duration::from_secs(1800),
            shutdown_timeout: Duration::from_secs(20),
            monitoring: Default::default(),
            network: Default::default(),
            db: Default::default(),
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self {
            metrics_port: 4000,
            events_buffer_len: 1024,
            listen_address: Uri::from_static("/ip4/0.0.0.0/tcp/0"),
            enable_rendezvous_client: true,
            enable_rendezvous_server: false,
            rendezvous_registration_ttl: Duration::from_secs(2 * 60 * 60),
            rendezvous_discovery_interval: Duration::from_secs(10 * 60),
            // TODO: we would like to enable this by default, however this breaks mdns on at least some linux distros. Requires further investigation.
            enable_mdns: true,
            mdns_enable_ipv6: false,
            mdns_query_interval: Duration::from_secs(5 * 60),
            mdns_ttl: Duration::from_secs(60 * 9),
            p2p_provider_timeout: Duration::new(30, 0),
            enable_pubsub: true,
            pubsub_duplication_cache_time: Duration::new(1, 0),
            pubsub_heartbeat: Duration::new(60, 0),
            pubsub_idle_timeout: Duration::new(60 * 60 * 24, 0),
            pubsub_max_transmit_size: 10 * 1024 * 1024,
            pubsub_mesh_n_low: 1,
            pubsub_mesh_n_high: 10,
            pubsub_mesh_n: 2,
            pubsub_mesh_outbound_min: 1,
            receipt_quorum: 2,
            rpc_host: IpAddr::V6(Ipv6Addr::LOCALHOST),
            rpc_max_connections: 10,
            rpc_port: 3030,
            rpc_server_timeout: Duration::new(120, 0),
            transport_connection_timeout: Duration::new(60, 0),
            webserver_host: Uri::from_static("127.0.0.1"),
            webserver_port: 1337,
            webserver_timeout: Duration::new(120, 0),
            websocket_capacity: 2048,
            websocket_receiver_timeout: Duration::from_millis(30_000),
            workflow_quorum: 3,
            keypair_config: PubkeyConfig::Random,
            node_addresses: Vec::new(),
            announce_addresses: Vec::new(),
            max_connected_peers: 32,
            max_announce_addresses: 10,
            poll_cache_interval: Duration::from_millis(1000),
            #[cfg(feature = "ipfs")]
            ipfs: Default::default(),
        }
    }
}

impl Node {
    /// Monitoring settings getter.
    pub fn monitoring(&self) -> &Monitoring {
        &self.monitoring
    }

    /// Network settings.
    pub fn network(&self) -> &Network {
        &self.network
    }

    /// Node shutdown timeout.
    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }
}

impl Network {
    /// TODO
    #[cfg(feature = "ipfs")]
    pub(crate) fn ipfs(&self) -> &Ipfs {
        &self.ipfs
    }
}

impl Settings {
    /// Load settings.
    ///
    /// Inject environment variables naming them properly on the settings,
    /// e.g. HOMESTAR__NODE__DB__MAX_POOL_SIZE=10.
    ///
    /// Use two underscores as defined by the separator below
    pub fn load() -> Result<Self, ConfigError> {
        #[cfg(test)]
        {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/settings.toml");
            Self::build(Some(path))
        }
        #[cfg(not(test))]
        Self::build(None)
    }

    /// Load settings from file string that must conform to a [PathBuf].
    pub fn load_from_file(file: PathBuf) -> Result<Self, ConfigError> {
        Self::build(Some(file))
    }

    fn build(path: Option<PathBuf>) -> Result<Self, ConfigError> {
        let builder = if let Some(p) = path {
            Config::builder().add_source(File::with_name(
                &p.canonicalize()
                    .map_err(|e| ConfigError::NotFound(e.to_string()))?
                    .as_path()
                    .display()
                    .to_string(),
            ))
        } else {
            Config::builder()
        };

        let s = builder
            .add_source(Environment::with_prefix("HOMESTAR").separator("__"))
            .build()?;
        s.try_deserialize()
    }
}

#[allow(dead_code)]
fn config_dir() -> PathBuf {
    let config_dir =
        env::var("XDG_CONFIG_HOME").map_or_else(|_| home_dir().join(".config"), PathBuf::from);
    config_dir.join("homestar")
}

#[allow(dead_code)]
fn home_dir() -> PathBuf {
    let home = env::var(HOME_VAR).unwrap_or_else(|_| panic!("{} not found", HOME_VAR));
    PathBuf::from(home)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Settings;

    #[test]
    fn defaults() {
        let settings = Settings::load().unwrap();
        let node_settings = settings.node;

        let default_settings = Node {
            gc_interval: Duration::from_secs(1800),
            shutdown_timeout: Duration::from_secs(20),
            ..Default::default()
        };

        assert_eq!(node_settings, default_settings);
    }

    #[test]
    fn defaults_with_modification() {
        let settings = Settings::build(Some("fixtures/settings.toml".into())).unwrap();

        let mut default_modded_settings = Node::default();
        default_modded_settings.network.events_buffer_len = 1000;
        default_modded_settings.network.webserver_port = 9999;
        default_modded_settings.gc_interval = Duration::from_secs(1800);
        default_modded_settings.shutdown_timeout = Duration::from_secs(20);
        default_modded_settings.network.node_addresses =
            vec!["/ip4/127.0.0.1/tcp/9998/ws".to_string().try_into().unwrap()];
        assert_eq!(settings.node(), &default_modded_settings);
    }

    #[test]
    fn overriding_env() {
        std::env::set_var("HOMESTAR__NODE__NETWORK__RPC_PORT", "2046");
        std::env::set_var("HOMESTAR__NODE__DB__MAX_POOL_SIZE", "1");
        let settings = Settings::build(Some("fixtures/settings.toml".into())).unwrap();
        assert_eq!(settings.node.network.rpc_port, 2046);
        assert_eq!(settings.node.db.max_pool_size, 1);
    }

    #[test]
    fn import_existing_key() {
        let settings = Settings::build(Some("fixtures/settings-import-ed25519.toml".into()))
            .expect("setting file in test fixtures");

        let msg = b"foo bar";
        let signature = libp2p::identity::Keypair::ed25519_from_bytes([0; 32])
            .unwrap()
            .sign(msg)
            .unwrap();
        // round-about way of testing since there is no Eq derive for keypairs
        assert!(settings
            .node
            .network
            .keypair_config
            .keypair()
            .expect("import ed25519 key")
            .public()
            .verify(msg, &signature));
    }

    #[test]
    fn import_secp256k1_key() {
        let settings = Settings::build(Some("fixtures/settings-import-secp256k1.toml".into()))
            .expect("setting file in test fixtures");

        settings
            .node
            .network
            .keypair_config
            .keypair()
            .expect("import secp256k1 key");
    }

    #[test]
    fn seeded_secp256k1_key() {
        let settings = Settings::build(Some("fixtures/settings-random-secp256k1.toml".into()))
            .expect("setting file in test fixtures");

        settings
            .node
            .network
            .keypair_config
            .keypair()
            .expect("generate a seeded secp256k1 key");
    }

    #[test]
    fn test_config_dir_xdg() {
        env::remove_var("HOME");
        env::set_var("XDG_CONFIG_HOME", "/home/user/custom_config");
        assert_eq!(
            config_dir(),
            PathBuf::from("/home/user/custom_config/homestar")
        );
        env::remove_var("XDG_CONFIG_HOME");
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_config_dir() {
        env::set_var("HOME", "/home/user");
        env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(config_dir(), PathBuf::from("/home/user/.config/homestar"));
        env::remove_var("HOME");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_config_dir() {
        env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(
            config_dir(),
            PathBuf::from(format!(r"{}\.config\homestar", env!("USERPROFILE")))
        );
    }
}
