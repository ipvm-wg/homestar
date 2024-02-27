//! Pubkey configuration.

use anyhow::{anyhow, Context};
use clap::ValueEnum;
use libp2p::{identity, identity::secp256k1};
use rand::{Rng, SeedableRng};
use sec1::der::Decode;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use std::{
    fmt::Display,
    io::Read,
    path::{Path, PathBuf},
};
use tracing::info;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Configure how the Network keypair is generated or using an existing one
pub enum PubkeyConfig {
    /// A randomly generated key, intended primarily for testing
    #[serde(rename = "random")]
    Random,
    /// Seed string should be a base64 encoded 32 bytes. This is used as the RNG seed to generate a key.
    #[serde(rename = "random_seed")]
    GenerateFromSeed(RNGSeed),
    /// File path to a PEM encoded key
    #[serde(rename = "existing")]
    Existing(ExistingKeyPath),
}

/// Supported key types of homestar
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, ValueEnum)]
pub enum KeyType {
    /// Ed25519 key
    #[default]
    #[serde(rename = "ed25519")]
    Ed25519,
    /// Secp256k1 key
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

impl Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ed25519 => f.write_str("Ed25519"),
            KeyType::Secp256k1 => f.write_str("Secp256k1"),
        }
    }
}

/// Seed material for RNG generated keys
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RNGSeed {
    #[serde(default)]
    key_type: KeyType,
    #[serde_as(as = "Base64")]
    seed: [u8; 32],
}

impl RNGSeed {
    /// Create a new [RNGSeed]
    pub fn new(key_type: KeyType, seed: [u8; 32]) -> Self {
        Self { key_type, seed }
    }
}

/// Info on where and what the Key file is
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExistingKeyPath {
    #[serde(default)]
    key_type: KeyType,
    path: PathBuf,
}

impl ExistingKeyPath {
    /// Create a new [ExistingKeyPath]
    pub fn new(key_type: KeyType, path: PathBuf) -> Self {
        Self { key_type, path }
    }
}

impl PubkeyConfig {
    /// Produce a Keypair using the given configuration.
    /// Calling this function will access the filesystem if configured to import a key.
    pub(crate) fn keypair(&self) -> anyhow::Result<identity::Keypair> {
        match self {
            PubkeyConfig::Random => {
                info!(
                    subject = "pubkey_config.random",
                    category = "pubkey_config",
                    "generating random ed25519 key"
                );
                Ok(identity::Keypair::generate_ed25519())
            }
            PubkeyConfig::GenerateFromSeed(RNGSeed { key_type, seed }) => {
                // seed RNG with supplied seed
                let mut r = rand::prelude::StdRng::from_seed(*seed);
                let mut new_key: [u8; 32] = r.gen();

                match key_type {
                    KeyType::Ed25519 => {
                        info!(
                            subject = "pubkey_config.random_seed.ed25519",
                            category = "pubkey_config",
                            "generating random ed25519 key from seed"
                        );

                        identity::Keypair::ed25519_from_bytes(new_key).map_err(|e| {
                            anyhow!("failed to generate ed25519 key from random: {:?}", e)
                        })
                    }
                    KeyType::Secp256k1 => {
                        info!(
                            subject = "pubkey_config.random_seed.secp256k1",
                            category = "pubkey_config",
                            "generating random secp256k1 key from seed"
                        );

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

                        info!(
                            subject = "pubkey_config.path.ed25519",
                            category = "pubkey_config",
                            "importing ed25519 key from: {}",
                            path.display()
                        );

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
                        info!(
                            subject = "pubkey_config.path.secp256k1",
                            category = "pubkey_config",
                            "importing secp256k1 key from: {}",
                            path.display()
                        );

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
