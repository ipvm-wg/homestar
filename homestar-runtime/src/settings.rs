//! General runtime settings / configuration.

use config::{Config, ConfigError, Environment, File};
use derive_builder::Builder;
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

mod libp2p_config;
mod pubkey_config;
pub use libp2p_config::{Autonat, Dht, Libp2p, Mdns, Pubsub, Rendezvous};
pub use pubkey_config::{ExistingKeyPath, KeyType, PubkeyConfig, RNGSeed};

#[cfg(target_os = "windows")]
const HOME_VAR: &str = "USERPROFILE";
#[cfg(not(target_os = "windows"))]
const HOME_VAR: &str = "HOME";

/// Application settings.
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Settings {
    /// Node settings
    #[builder(default)]
    #[serde(default)]
    pub(crate) node: Node,
}

impl Settings {
    /// Node settings getter.
    pub fn node(&self) -> &Node {
        &self.node
    }
}

/// Server settings.
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
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

/// Database-related settings for a homestar node.
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
#[serde(default)]
pub struct Database {
    /// Database Url provided within the configuration file.
    ///
    /// Note: This is not used if the `DATABASE_URL` environment variable
    /// is set.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub(crate) url: Option<String>,
    /// Maximum number of connections managed by the pool.
    ///
    /// [pool]: crate::db::Pool
    pub(crate) max_pool_size: u32,
}

/// Monitoring settings.
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
#[serde(default)]
pub struct Monitoring {
    /// Tokio console port.
    pub console_subscriber_port: u16,
    /// Monitoring collection interval in milliseconds.
    #[cfg(feature = "monitoring")]
    #[cfg_attr(docsrs, doc(cfg(feature = "monitoring")))]
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub process_collector_interval: Duration,
}

/// Network settings for a homestar node.
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
#[serde(default)]
pub struct Network {
    /// libp2p Settings.
    pub(crate) libp2p: Libp2p,
    /// Metrics Settings.
    pub(crate) metrics: Metrics,
    /// Buffer-length for event(s) / command(s) channels.
    pub(crate) events_buffer_len: usize,
    /// RPC server settings.
    pub(crate) rpc: Rpc,
    /// Pubkey setup configuration.
    pub(crate) keypair_config: PubkeyConfig,
    /// Event handler poll cache interval in milliseconds.
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub(crate) poll_cache_interval: Duration,
    /// IPFS settings.
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    pub(crate) ipfs: Ipfs,
    /// Webserver settings
    pub(crate) webserver: Webserver,
}

/// IPFS Settings
#[cfg(feature = "ipfs")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
#[serde(default)]
pub struct Ipfs {
    /// The host where Homestar expects IPFS.
    pub(crate) host: String,
    /// The port where Homestar expects IPFS.
    pub(crate) port: u16,
}

/// Metrics settings.
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
#[serde(default)]
pub struct Metrics {
    /// Metrics port for prometheus scraping.
    pub(crate) port: u16,
}

/// RPC server settings.
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
#[serde(default)]
pub struct Rpc {
    /// RPC-server port.
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) host: IpAddr,
    /// RPC-server max-concurrent connections.
    pub(crate) max_connections: usize,
    /// RPC-server port.
    pub(crate) port: u16,
    #[serde_as(as = "DurationSeconds<u64>")]
    /// RPC-server timeout.
    pub(crate) server_timeout: Duration,
}

/// Webserver settings
#[serde_as]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[builder(default)]
#[serde(default)]
pub struct Webserver {
    /// V4 Webserver host address.
    #[serde(with = "http_serde::uri")]
    pub(crate) v4_host: Uri,
    /// V6 (fallback) Webserver host address.
    #[serde(with = "http_serde::uri")]
    pub(crate) v6_host: Uri,
    /// Webserver-server port.
    pub(crate) port: u16,
    /// Webserver timeout.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub(crate) timeout: Duration,
    /// Message capacity for the websocket-server.
    pub(crate) websocket_capacity: usize,
    /// Websocket-server send timeout.
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub(crate) websocket_sender_timeout: Duration,
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

impl Default for Database {
    fn default() -> Self {
        Self {
            max_pool_size: 100,
            url: None,
        }
    }
}

#[cfg(feature = "monitoring")]
#[cfg_attr(docsrs, doc(cfg(feature = "monitoring")))]
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

impl Default for Network {
    fn default() -> Self {
        Self {
            libp2p: Libp2p::default(),
            metrics: Metrics::default(),
            events_buffer_len: 1024,
            rpc: Rpc::default(),
            keypair_config: PubkeyConfig::Random,
            poll_cache_interval: Duration::from_millis(1000),
            #[cfg(feature = "ipfs")]
            #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
            ipfs: Default::default(),
            webserver: Webserver::default(),
        }
    }
}

impl Network {
    /// IPFS settings.
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    pub(crate) fn ipfs(&self) -> &Ipfs {
        &self.ipfs
    }

    /// libp2p settings.
    pub(crate) fn libp2p(&self) -> &Libp2p {
        &self.libp2p
    }

    /// Webserver settings.
    pub(crate) fn webserver(&self) -> &Webserver {
        &self.webserver
    }
}

#[cfg(feature = "ipfs")]
#[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
impl Default for Ipfs {
    fn default() -> Self {
        Self {
            host: Ipv4Addr::LOCALHOST.to_string(),
            port: 5001,
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self { port: 4000 }
    }
}

impl Default for Rpc {
    fn default() -> Self {
        Self {
            host: IpAddr::V6(Ipv6Addr::LOCALHOST),
            max_connections: 10,
            port: 3030,
            server_timeout: Duration::new(120, 0),
        }
    }
}

impl Default for Webserver {
    fn default() -> Self {
        Self {
            v4_host: Uri::from_static("127.0.0.1"),
            v6_host: Uri::from_static("[::1]"),
            port: 1337,
            timeout: Duration::new(120, 0),
            websocket_capacity: 2048,
            websocket_sender_timeout: Duration::from_millis(30_000),
        }
    }
}

impl Settings {
    /// Settings file path.
    pub fn path() -> PathBuf {
        config_file_with_extension("toml")
    }

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
        let builder = Config::builder();

        #[cfg(not(test))]
        let builder = builder.add_source(File::from(config_file()).required(false));

        let builder = if let Some(p) = path {
            builder.add_source(File::with_name(
                &p.canonicalize()
                    .map_err(|e| ConfigError::NotFound(e.to_string()))?
                    .as_path()
                    .display()
                    .to_string(),
            ))
        } else {
            builder
        };

        let s = builder
            .add_source(Environment::with_prefix("HOMESTAR").separator("__"))
            .build()?;
        s.try_deserialize()
    }
}

fn config_file() -> PathBuf {
    config_dir().join("settings")
}

fn config_file_with_extension(ext: &str) -> PathBuf {
    config_file().with_extension(ext)
}

fn config_dir() -> PathBuf {
    let config_dir =
        env::var("XDG_CONFIG_HOME").map_or_else(|_| home_dir().join(".config"), PathBuf::from);
    config_dir.join("homestar")
}

fn home_dir() -> PathBuf {
    let home = env::var(HOME_VAR).unwrap_or_else(|_| panic!("{} not found", HOME_VAR));
    PathBuf::from(home)
}

#[cfg(test)]
mod test {
    use super::*;

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
        default_modded_settings.network.webserver.port = 9999;
        default_modded_settings.gc_interval = Duration::from_secs(1800);
        default_modded_settings.shutdown_timeout = Duration::from_secs(20);
        default_modded_settings.network.libp2p.node_addresses =
            vec!["/ip4/127.0.0.1/tcp/9998/ws".to_string().try_into().unwrap()];
        assert_eq!(settings.node(), &default_modded_settings);
    }

    #[test]
    #[serial_test::parallel]
    fn default_config() {
        let settings = Settings::load().unwrap();
        let default_config = Settings::default();
        assert_eq!(settings, default_config);
    }

    #[test]
    #[serial_test::file_serial]
    fn overriding_env_serial() {
        std::env::set_var("HOMESTAR__NODE__NETWORK__RPC__PORT", "2046");
        std::env::set_var("HOMESTAR__NODE__DB__MAX_POOL_SIZE", "1");
        let settings = Settings::build(Some("fixtures/settings.toml".into())).unwrap();
        assert_eq!(settings.node.network.rpc.port, 2046);
        assert_eq!(settings.node.db.max_pool_size, 1);
    }

    #[test]
    fn import_existing_key() {
        // Test using a key not containing curve parameters
        let settings = Settings::build(Some("fixtures/settings-import-ed25519.toml".into()))
            .expect("setting file in test fixtures");

        let msg = b"foo bar";
        let key = [0; 32];
        let signature = libp2p::identity::Keypair::ed25519_from_bytes(key)
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

        // Test using a key containing curve parameters
        let settings = Settings::build(Some(
            "fixtures/settings-import-ed25519-with-params.toml".into(),
        ))
        .expect("setting file in test fixtures");

        let msg = b"foo bar";
        let key = [
            255, 211, 202, 168, 61, 181, 166, 62, 247, 234, 100, 3, 193, 51, 5, 251, 20, 1, 62,
            135, 139, 231, 142, 86, 225, 243, 163, 90, 161, 31, 155, 129,
        ];
        let signature = libp2p::identity::Keypair::ed25519_from_bytes(key)
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
    #[serial_test::file_serial]
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
    #[serial_test::file_serial]
    fn test_config_dir() {
        env::set_var("HOME", "/home/user");
        env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(config_dir(), PathBuf::from("/home/user/.config/homestar"));
        env::remove_var("HOME");
    }

    #[cfg(target_os = "windows")]
    #[test]
    #[serial_test::file_serial]
    fn test_config_dir() {
        env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(
            config_dir(),
            PathBuf::from(format!(r"{}\.config\homestar", env!("USERPROFILE")))
        );
    }
}
