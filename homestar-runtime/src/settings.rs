//! Settings / Configuration.

use anyhow::{anyhow, Context};
use config::{Config, ConfigError, Environment, File};
use http::Uri;
use libp2p::{identity, identity::secp256k1};
use rand::{Rng, SeedableRng};
use sec1::der::Decode;
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
    #[serde(rename = "existing")]
    Existing(ExistingKeyPath),
}

/// Supported key types of homestar
#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) enum KeyType {
    #[default]
    #[serde(rename = "ed25519")]
    Ed25519,
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

/// Seed material for RNG generated keys
#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PupkeyRNGSeed {
    #[serde(default)]
    key_type: KeyType,
    #[serde_as(as = "Base64")]
    seed: [u8; 32],
}

/// Info on where and what the Key file is
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ExistingKeyPath {
    #[serde(default)]
    key_type: KeyType,
    path: String,
}

impl PubkeyConfig {
    /// Produce a Keypair using the given configuration.
    /// Calling this function will access the filesystem if configured to import a key.
    pub(crate) fn keypair(&self) -> anyhow::Result<identity::Keypair> {
        match self {
            PubkeyConfig::Random => Ok(identity::Keypair::generate_ed25519()),
            PubkeyConfig::GenerateFromSeed(PupkeyRNGSeed { key_type, seed }) => {
                // seed RNG with supplied seed
                let mut r = rand::prelude::StdRng::from_seed(*seed);
                let mut new_key: [u8; 32] = r.gen();

                match key_type {
                    KeyType::Ed25519 => {
                        identity::Keypair::ed25519_from_bytes(new_key).map_err(|e| {
                            anyhow!("failed to generate ed25519 key from random: {:?}", e)
                        })
                    }
                    KeyType::Secp256k1 => {
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
                        let (tag, mut key) = sec1::der::pem::decode_vec(&buf)
                            .map_err(|e| anyhow!("key file must be PEM formatted: {:?}", e))?;
                        if tag != "PRIVATE KEY" {
                            return Err(anyhow!(
                                "imported key file had a header of '{}', expected 'PRIVATE KEY' for ed25519",
                                tag
                            ));
                        }

                        // raw bytes of ed25519 secret key from PEM file
                        identity::Keypair::ed25519_from_bytes(&mut key)
                            .with_context(|| "imported key material was invalid for ed25519")
                    }
                    KeyType::Secp256k1 => {
                        let sk = match path.extension().and_then(|ext| ext.to_str()) {
                            Some("der") => sec1::EcPrivateKey::from_der(buf.as_slice()).map_err(|e| anyhow!("failed to parse DER encoded secp256k1 key: {e:?}")),
                            Some("pem") => {
                                Err(anyhow!("PEM encoded secp256k1 keys are unsupported at the moment. Please file an issue if you require this."))
                            },
                            _ => Err(anyhow!("please disambiguate file from either PEM or DER with a file extension."))
                        }?;
                        let kp = secp256k1::SecretKey::try_from_bytes(sk.private_key.to_vec())
                            .map(secp256k1::Keypair::from)
                            .map_err(|e| anyhow!("failed to import secp256k1 key: {:?}", e))?;
                        Ok(identity::Keypair::from(kp))
                    }
                }
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
        let settings = Settings::build("fixtures/settings-import-ed25519.toml".into()).unwrap();

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
            .unwrap()
            .public()
            .verify(msg, &signature));
    }

    #[test]
    fn import_secp256k1_key() {
        let settings = Settings::build("fixtures/settings-import-secp256k1.toml".into()).unwrap();

        settings.node.network.keypair_config.keypair().unwrap();
    }

    #[test]
    fn seeded_secp256k1_key() {
        let settings = Settings::build("fixtures/settings-random-secp256k1.toml".into()).unwrap();

        settings.node.network.keypair_config.keypair().unwrap();
    }
}
