//! Settings / Configuration.

use config::{Config, ConfigError, Environment, File};
use http::Uri;
use serde::Deserialize;
use std::path::PathBuf;

/// Server settings.
#[derive(Clone, Debug, Deserialize)]
pub struct Node {
    #[serde(default)]
    pub(crate) network: Network,
    #[serde(default)]
    pub(crate) db: Database,
}

/// Process monitoring settings.
#[derive(Clone, Debug, Deserialize)]
pub struct Monitoring {
    /// Monitoring collection interval.
    #[allow(dead_code)]
    process_collector_interval: u64,
}

#[derive(Debug, Deserialize)]
/// Application settings.
pub struct Settings {
    monitoring: Monitoring,
    node: Node,
}

impl Settings {
    /// Monitoring settings getter.
    pub fn monitoring(&self) -> &Monitoring {
        &self.monitoring
    }

    /// Node
    pub fn node(&self) -> &Node {
        &self.node
    }
}

/// Network-related settings for a homestar node.
#[derive(Clone, Debug, Deserialize)]
pub struct Network {
    ///
    pub(crate) events_buffer_len: usize,
    /// Address for [Swarm] to listen on.
    ///
    /// [Swarm]: libp2p::swarm::Swarm
    #[serde(with = "http_serde::uri")]
    pub(crate) listen_address: Uri,
    /// Pub/sub hearbeat interval for mesh configuration.
    pub(crate) pubsub_heartbeat_secs: u64,
    /// Quorum for receipt records on the DHT.
    pub(crate) receipt_quorum: usize,
    /// Websocket-server host address.
    #[serde(with = "http_serde::uri")]
    pub(crate) websocket_host: Uri,
    /// Websocket-server port.
    pub(crate) websocket_port: u16,
    /// Number of *bounded* clients to send messages to, used for a
    /// [tokio::sync::broadcast::channel]
    pub(crate) websocket_capacity: usize,
}

/// Database-related settings for a homestar node.
#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    /// Maximum number of connections managed by the [pool].
    ///
    /// [pool]: crate::db::Pool
    pub(crate) max_pool_size: u32,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            events_buffer_len: 100,
            listen_address: Uri::from_static("/ip4/0.0.0.0/tcp/0"),
            pubsub_heartbeat_secs: 10,
            receipt_quorum: 2,
            websocket_host: Uri::from_static("127.0.0.1"),
            websocket_port: 1337,
            websocket_capacity: 100,
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self { max_pool_size: 100 }
    }
}

impl Settings {
    /// Load settings.
    pub fn load() -> Result<Self, ConfigError> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/settings.toml");
        // inject environment variables naming them properly on the settings
        // e.g. [database] url="foo"
        // would be injected with environment variable HOMESTAR_DATABASE_URL="foo"
        // use one underscore as defined by the separator below
        Self::build(path)
    }

    /// Load settings from file string that must conform to a [PathBuf].
    pub fn load_from_file(file: String) -> Result<Self, ConfigError> {
        let path = PathBuf::from(file);
        Self::build(path)
    }

    fn build(path: PathBuf) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(&path.as_path().display().to_string()))
            .add_source(Environment::with_prefix("HOMESTAR").separator("__"))
            .build()?;
        s.try_deserialize()
    }
}
