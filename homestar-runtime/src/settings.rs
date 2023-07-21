//! Settings / Configuration.

use anyhow::{anyhow, Context};
use config::{Config, ConfigError, Environment, File};
use http::Uri;
use libp2p::{identity, identity::secp256k1};
use rand::{Rng, SeedableRng};
use sec1::der::Decode;
use serde::Deserialize;
use serde_with::{base64::Base64, serde_as, DisplayFromStr, DurationSeconds};
use std::{
    io::Read,
    net::{IpAddr, Ipv6Addr},
    path::{Path, PathBuf},
    time::Duration,
};
use tracing::info;

/// Application settings.
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Monitoring {
    /// Monitoring collection interval.
    #[allow(dead_code)]
    process_collector_interval: u64,
}

/// Server settings.
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Node {
    #[serde(default)]
    pub(crate) network: Network,
    #[serde(default)]
    pub(crate) db: Database,
    #[serde_as(as = "DurationSeconds<u64>")]
    #[serde(default = "default_shutdown_timeout")]
    pub(crate) shutdown_timeout: Duration,
}

/// Network-related settings for a homestar node.
#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq)]
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
}

/// Database-related settings for a homestar node.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Database {
    /// Maximum number of connections managed by the [pool].
    ///
    /// [pool]: crate::db::Pool
    pub(crate) max_pool_size: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
/// Configure how the Network keypair is generated or using an existing one
pub(crate) enum PubkeyConfig {
    #[serde(rename = "random")]
    Random,
    /// Seed string should be a base64 encoded 32 bytes. This is used as the RNG seed to generate a ed25519 key.
    #[serde(rename = "random_seed")]
    GenerateFromSeed(PupkeyRNGSeed),
    /// File path to a PEM encoded ed25519 key
    #[serde(rename = "existing")]
    Existing(ExistingKeyPath),
}

/// Supported key types of homestar
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub(crate) enum KeyType {
    #[default]
    #[serde(rename = "ed25519")]
    Ed25519,
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

/// Seed material for RNG generated keys
#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct PupkeyRNGSeed {
    #[serde(default)]
    key_type: KeyType,
    #[serde_as(as = "Base64")]
    seed: [u8; 32],
}

/// Info on where and what the Key file is
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct ExistingKeyPath {
    #[serde(default)]
    key_type: KeyType,
    path: String,
}

impl Default for Database {
    fn default() -> Self {
        Self { max_pool_size: 100 }
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
            transport_connection_timeout: Duration::new(20, 0),
            websocket_host: Uri::from_static("127.0.0.1"),
            websocket_port: 1337,
            websocket_capacity: 100,
            workflow_quorum: 3,
            keypair_config: PubkeyConfig::Random,
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

impl PubkeyConfig {
    /// Produce a Keypair using the given configuration.
    /// Calling this function will access the filesystem if configured to import a key.
    pub(crate) fn keypair(&self) -> anyhow::Result<identity::Keypair> {
        match self {
            PubkeyConfig::Random => {
                info!("generating random ed25519 key");
                Ok(identity::Keypair::generate_ed25519())
            }
            PubkeyConfig::GenerateFromSeed(PupkeyRNGSeed { key_type, seed }) => {
                // seed RNG with supplied seed
                let mut r = rand::prelude::StdRng::from_seed(*seed);
                let mut new_key: [u8; 32] = r.gen();

                match key_type {
                    KeyType::Ed25519 => {
                        info!("generating radom ed25519 key from seed");

                        identity::Keypair::ed25519_from_bytes(new_key).map_err(|e| {
                            anyhow!("failed to generate ed25519 key from random: {:?}", e)
                        })
                    }
                    KeyType::Secp256k1 => {
                        info!("generating radom secp256k1 key from seed");

                        let sk =
                            secp256k1::SecretKey::try_from_bytes(&mut new_key).map_err(|e| {
                                anyhow!("failed to generate secp256k1 key from random: {:?}", e)
                            })?;
                        let kp = secp256k1::Keypair::from(sk);
                        Ok(identity::Keypair::from(kp))
                    }
                }
            }
            PubkeyConfig::Existing(ExistingKeyPath { key_type, path }) => {
                let path = Path::new(&path);
                let mut file = std::fs::File::open(path).context("unable to read key file")?;

                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .context("unable to read bytes from file, is the file corrupted?")?;

                match key_type {
                    KeyType::Ed25519 => {
                        const PEM_HEADER: &str = "PRIVATE KEY";

                        info!("importing ed25519 key from: {}", path.display());

                        let (tag, mut key) = sec1::der::pem::decode_vec(&buf)
                            .map_err(|e| anyhow!("key file must be PEM formatted: {:#?}", e))?;
                        if tag != PEM_HEADER {
                            return Err(anyhow!("imported key file had a header of '{tag}', expected '{PEM_HEADER}' for ed25519"));
                        }

                        // raw bytes of ed25519 secret key from PEM file
                        identity::Keypair::ed25519_from_bytes(&mut key)
                            .with_context(|| "imported key material was invalid for ed25519")
                    }
                    KeyType::Secp256k1 => {
                        info!("importing secp256k1 key from: {}", path.display());

                        let sk = match path.extension().and_then(|ext| ext.to_str()) {
                            Some("der") => sec1::EcPrivateKey::from_der(buf.as_slice()).map_err(|e| anyhow!("failed to parse DER encoded secp256k1 key: {e:#?}")),
                            Some("pem") => {
                                Err(anyhow!("PEM encoded secp256k1 keys are unsupported at the moment. Please file an issue if you require this."))
                            },
                            _ => Err(anyhow!("please disambiguate file from either PEM or DER with a file extension."))
                        }?;
                        let kp = secp256k1::SecretKey::try_from_bytes(sk.private_key.to_vec())
                            .map(secp256k1::Keypair::from)
                            .map_err(|e| anyhow!("failed to import secp256k1 key: {:#?}", e))?;
                        Ok(identity::Keypair::from(kp))
                    }
                }
            }
        }
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
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/settings.toml");
        #[cfg(not(test))]
        let path = PathBuf::from("config/settings.toml");

        Self::build(path)
    }

    /// Load settings from file string that must conform to a [PathBuf].
    pub fn load_from_file(file: String) -> Result<Self, ConfigError> {
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
        default_modded_settings.shutdown_timeout = Duration::from_secs(20);

        assert_eq!(settings.node, default_modded_settings);
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
