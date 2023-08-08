//! Settings / Configuration.

use config::{Config, ConfigError, Environment, File};
use http::Uri;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, DurationSeconds};
use std::{
    net::{IpAddr, Ipv6Addr},
    path::PathBuf,
    time::Duration,
};

mod pubkey_config;
pub(crate) use pubkey_config::PubkeyConfig;

/// Application settings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub(crate) monitoring: Monitoring,
    pub(crate) node: Node,
}

impl Settings {
    /// Monitoring settings getter.
    pub fn monitoring(&self) -> &Monitoring {
        &self.monitoring
    }

    /// Node settings getter.
    pub fn node(&self) -> &Node {
        &self.node
    }
}

/// Process monitoring settings.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Monitoring {
    /// Monitoring collection interval.
    #[allow(dead_code)]
    process_collector_interval: u64,
}

/// Server settings.
#[serde_as]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Node {
    /// Network settings.
    #[serde(default)]
    pub(crate) network: Network,
    /// Database settings.
    #[serde(default)]
    pub(crate) db: Database,
    /// Garbage collection interval.
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_gc_interval")]
    pub(crate) gc_interval: Duration,
    /// Shutdown timeout.
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_shutdown_timeout")]
    pub(crate) shutdown_timeout: Duration,
}

/// Network-related settings for a homestar node.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Network {
    /// Buffer-length for event(s) / command(s) channels.
    pub(crate) events_buffer_len: usize,
    /// Address for [Swarm] to listen on.
    ///
    /// [Swarm]: libp2p::swarm::Swarm
    #[serde(with = "http_serde::uri")]
    pub(crate) listen_address: Uri,
    /// Timeout for p2p requests for a provided record.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) p2p_provider_timeout: Duration,
    /// Pub/sub duplicate cache time.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) pubsub_duplication_cache_time: Duration,
    /// Pub/sub hearbeat interval for mesh configuration.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) pubsub_heartbeat: Duration,
    /// Pub/sub idle timeout
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) pubsub_idle_timeout: Duration,
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
    /// Websocket-server host address.
    #[serde(with = "http_serde::uri")]
    pub(crate) websocket_host: Uri,
    /// Websocket-server port.
    pub(crate) websocket_port: u16,
    /// Number of *bounded* clients to send messages to, used for a
    /// [tokio::sync::broadcast::channel]
    pub(crate) websocket_capacity: usize,
    /// Quorum for [workflow::Info] records on the DHT.
    ///
    /// [workflow::Info]: crate::workflow::Info
    pub(crate) workflow_quorum: usize,
    /// Pubkey setup configuration
    pub(crate) keypair_config: PubkeyConfig,
    /// Multiaddrs of the trusted nodes to connect to on startup. These addresses are added as explicit peers for gossipsub.
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub(crate) trusted_node_addresses: Vec<libp2p::Multiaddr>,
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

impl Default for Database {
    fn default() -> Self {
        Self {
            max_pool_size: 100,
            url: None,
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self {
            events_buffer_len: 100,
            listen_address: Uri::from_static("/ip4/0.0.0.0/tcp/0"),
            p2p_provider_timeout: Duration::new(30, 0),
            pubsub_duplication_cache_time: Duration::new(1, 0),
            pubsub_heartbeat: Duration::new(60, 0),
            pubsub_idle_timeout: Duration::new(60 * 60 * 24, 0),
            receipt_quorum: 2,
            rpc_host: IpAddr::V6(Ipv6Addr::LOCALHOST),
            rpc_max_connections: 10,
            rpc_port: 3030,
            rpc_server_timeout: Duration::new(120, 0),
            transport_connection_timeout: Duration::new(20, 0),
            websocket_host: Uri::from_static("127.0.0.1"),
            websocket_port: 1337,
            websocket_capacity: 100,
            workflow_quorum: 3,
            keypair_config: PubkeyConfig::Random,
            trusted_node_addresses: Vec::new(),
        }
    }
}

impl Node {
    /// Network settings.
    pub fn network(&self) -> &Network {
        &self.network
    }
    /// Node shutdown timeout.
    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }
}

fn default_shutdown_timeout() -> Duration {
    Duration::new(20, 0)
}

fn default_gc_interval() -> Duration {
    Duration::new(1800, 0)
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
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/settings.toml");
        #[cfg(not(test))]
        let path = PathBuf::from("config/settings.toml");

        Self::build(path)
    }

    /// Load settings from file string that must conform to a [PathBuf].
    pub fn load_from_file<F>(file: F) -> Result<Self, ConfigError>
    where
        F: AsRef<str>,
        PathBuf: From<F>,
    {
        let path = PathBuf::from(file);
        Self::build(path)
    }

    fn build(path: PathBuf) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(
                &path
                    .canonicalize()
                    .map_err(|e| ConfigError::NotFound(e.to_string()))?
                    .as_path()
                    .display()
                    .to_string(),
            ))
            .add_source(Environment::with_prefix("HOMESTAR").separator("__"))
            .build()?;
        s.try_deserialize()
    }
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
        let settings = Settings::build("fixtures/settings.toml".into()).unwrap();

        let mut default_modded_settings = Node::default();
        default_modded_settings.network.events_buffer_len = 1000;
        default_modded_settings.network.websocket_port = 9999;
        default_modded_settings.gc_interval = Duration::from_secs(1800);
        default_modded_settings.shutdown_timeout = Duration::from_secs(20);
        default_modded_settings.network.trusted_node_addresses =
            vec!["/ip4/127.0.0.1/tcp/9998/ws".to_string().try_into().unwrap()];
        assert_eq!(settings.node(), &default_modded_settings);
    }

    #[test]
    fn overriding_env() {
        std::env::set_var("HOMESTAR__NODE__NETWORK__RPC_PORT", "2046");
        std::env::set_var("HOMESTAR__NODE__DB__MAX_POOL_SIZE", "1");
        let settings = Settings::build("fixtures/settings.toml".into()).unwrap();
        assert_eq!(settings.node.network.rpc_port, 2046);
        assert_eq!(settings.node.db.max_pool_size, 1);
    }

    #[test]
    fn import_existing_key() {
        let settings = Settings::build("fixtures/settings-import-ed25519.toml".into())
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
        let settings = Settings::build("fixtures/settings-import-secp256k1.toml".into())
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
        let settings = Settings::build("fixtures/settings-random-secp256k1.toml".into())
            .expect("setting file in test fixtures");

        settings
            .node
            .network
            .keypair_config
            .keypair()
            .expect("generate a seeded secp256k1 key");
    }
}
