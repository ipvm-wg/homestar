//! Settings / Configuration.

use anyhow::{anyhow, Context};
use config::{Config, ConfigError, Environment, File};
use http::Uri;
use libp2p::identity;
use rand::{Rng, SeedableRng};
use serde::Deserialize;
use serde_with::{base64::Base64, serde_as};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

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

#[derive(Clone, Debug, Deserialize)]
/// Configure how the Network keypair is generated or using an existing one
pub(crate) enum PubkeyConfig {
    #[serde(rename = "random")]
    Random,
    /// Seed string should be a base64 encoded 32 bytes. This is used as the RNG seed to generate a ed25519 key.
    #[serde(rename = "random_seed")]
    GenerateFromSeed(PupkeyRNGSeed),
    /// File path to a PEM encoded ed25519 key
    #[serde(rename = "path")]
    Existing(String),
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PupkeyRNGSeed {
    #[serde_as(as = "Base64")]
    seed: [u8; 32],
}

impl PubkeyConfig {
    /// Produce a Keypair using the given configuration.
    pub(crate) fn generate_keypair(&self) -> anyhow::Result<identity::Keypair> {
        match self {
            PubkeyConfig::Random => Ok(identity::Keypair::generate_ed25519()),
            PubkeyConfig::GenerateFromSeed(rng_seed) => {
                // seed RNG with supplied seed
                let mut r = rand::prelude::StdRng::from_seed(rng_seed.seed);
                let new_key: [u8; 32] = r.gen();

                identity::Keypair::ed25519_from_bytes(new_key)
                    .map_err(|e| anyhow!("Failed to generate secret key from seed: {:?}", e))
            }
            PubkeyConfig::Existing(path) => {
                let path = Path::new(&path);
                let pem_file = {
                    let mut s = String::new();
                    std::fs::File::open(path)
                        .context("Unable to read key file")?
                        .read_to_string(&mut s)
                        .context("Failed to read file into a string, is it corrupted?")?;
                    s
                };
                // parse key from PEM file
                let pem = pem::parse(pem_file).with_context(|| "Key file must be PEM formatted")?;
                // we only take ed25519
                if pem.tag() != "PRIVATE KEY" {
                    return Err(anyhow!("Imported key file must be a private key"));
                }
                identity::Keypair::ed25519_from_bytes(&mut pem.contents().to_vec())
                    .with_context(|| "Imported key was not parsable into an ed25519 secret key")
            }
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
    fn import_existing_key() {
        let settings = Settings::load().unwrap();

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
            .generate_keypair()
            .unwrap()
            .public()
            .verify(msg, &signature));
    }
}
