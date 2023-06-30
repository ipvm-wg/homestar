//! Settings / Configuration.

use config::{Config, ConfigError, Environment, File};
use http::Uri;
use libp2p::{identity::{self, DecodingError}};
use serde::Deserialize;
use std::{path::{PathBuf, Path}, io::Read};
use sha2::{Sha256, Digest};

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

#[derive(Default, Clone, Debug, Deserialize)]
/// Configure how the Network keypair is generated or using an existing one
pub(crate) enum PubkeyConfig {
    #[default]
    #[serde(rename = "random")]
    Random,
    /// Seed string is hashed with SHA-256 to produce the ed25519 secret key.
    #[serde(rename = "seed")]
    GenerateFromSeed(String),
    /// File path to a PEM encoded ed25519 key
    #[serde(rename = "path")]
    Existing(String),
}


impl PubkeyConfig {
    /// Produce a Keypair using the given configuration. Consumes `self` to avoid keeping secrets laying around.
    fn generate_keypair(self) -> Result<identity::Keypair, DecodingError> {
        match self {
            PubkeyConfig::Random => Ok(identity::Keypair::generate_ed25519()),
            PubkeyConfig::GenerateFromSeed(seed) => {
                let mut hasher = Sha256::default();
                hasher.update(&seed);
                identity::Keypair::ed25519_from_bytes(hasher.finalize())
            },
            PubkeyConfig::Existing(path) => {
                let path = Path::new(&path);
                println!("{:?}", path);
                let pem_file =
                {
                    let mut s = String::new();
                    // TODO convert err
                    std::fs::File::open(path).unwrap().read_to_string(&mut s).unwrap();
                    s
                };
                // TODO convert err
                let pem = pem::parse(pem_file).unwrap();
                // we only take ed25519
                if pem.tag() != "PRIVATE KEY" {
                    // TODO custom error at this point...
                    panic!("Not a private key pem file")
                }
                println!("{}", pem.tag());
                identity::Keypair::ed25519_from_bytes(&mut pem.contents().to_vec())
            },
        }
    }
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
#[serde(default)]
pub(crate) struct Network {
    ///
    pub(crate) events_buffer_len: usize,
    /// Address for [Swarm] to listen on.
    ///
    /// [Swarm]: libp2p::swarm::Swarm
    #[serde(with = "http_serde::uri")]
    pub(crate) listen_address: Uri,
    /// Pub/sub duplicate cache time.
    pub(crate) pubsub_duplication_cache_secs: u64,
    /// Pub/sub hearbeat interval for mesh configuration.
    pub(crate) pubsub_heartbeat_secs: u64,
    /// Pub/sub idle timeout
    pub(crate) pubsub_idle_timeout_secs: u64,
    /// Quorum for receipt records on the DHT.
    pub(crate) receipt_quorum: usize,
    /// Transport connection timeout.
    pub(crate) transport_connection_timeout_secs: u64,
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
}

/// Database-related settings for a homestar node.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Database {
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
            pubsub_duplication_cache_secs: 1,
            pubsub_heartbeat_secs: 60,
            pubsub_idle_timeout_secs: 60 * 60 * 24,
            receipt_quorum: 2,
            transport_connection_timeout_secs: 20,
            websocket_host: Uri::from_static("127.0.0.1"),
            websocket_port: 1337,
            websocket_capacity: 100,
            workflow_quorum: 3,
            keypair_config: PubkeyConfig::Random,
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

#[cfg(test)]
mod test {
    use crate::Settings;

    #[test]
    fn load_settings() {
        let settings = Settings::load().unwrap();

        println!("{:?}", settings.node.network.keypair_config.generate_keypair())

    }
}
