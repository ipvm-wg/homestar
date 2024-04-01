#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! homestar-runtime is a determistic Wasm runtime and effectful workflow/job
//! system intended to be embedded inside or run alongside IPFS or another
//! content-addressable store.
//!
//! You can find a more complete description [here].
//!
//! ## Related crates/packages:
//!
//! - [homestar-invocation]
//! - [homestar-wasm]
//! - [homestar-workflow]
//!
//! ## Getting Started
//!
//! For getting started with Homestar in general, please check out our
//! [README] and [Quickstart] guide.
//!
//! ## Feature flags
//!
//! - `default`: Enables the default set of features.
//! - `dev`: Enables a dev-friendly, lighter set of features for development.
//! - `ansi-logs`: Enables ANSI color codes in logs.
//! - `console`: Enables [tokio console] debugging.
//! - `ipfs`: Enables [IPFS]-related integration and settings.
//! - `monitoring`: Enables node monitoring-related features and dependencies.
//! - `profile`: Enables profiling-related features and dependencies.
//! - `test-utils`: Enables utilities for unit testing and benchmarking.
//! - `wasmtime-default`: Enables the default set of features for the embedded
//!                       [Wasmtime] runtime.
//! - `websocket-notify`: Enables websocket notifications.
//!
//! ## Examples
//!
//! Check out our [examples] directory to explore some Homestar
//! scenarios and see the system in action.
//!
//! [examples]: https://github.com/ipvm-wg/homestar/tree/main/examples
//! [here]: https://github.com/ipvm-wg/spec
//! [homestar-invocation]: https://docs.rs/homestar-invocation
//! [homestar-workflow]: https://docs.rs/homestar-workflow
//! [homestar-wasm]: https://docs.rs/homestar-wasm
//! [IPFS]: https://ipfs.tech
//! [Quickstart]: https://github.com/ipvm-wg/homestar/blob/main/README.md#quickstart
//! [README]: https://github.com/ipvm-wg/homestar/blob/main/README.md
//! [tokio console]: https://github.com/tokio-rs/console/tree/main/tokio-console
//! [Wasmtime]: https://github.com/bytecodealliance/wasmtime

pub mod channel;
pub mod cli;
pub mod daemon;
pub mod db;
mod event_handler;
mod ip;
mod logger;
pub mod network;
mod receipt;
pub mod runner;
mod scheduler;
mod settings;
mod tasks;
/// Test utilities.
#[cfg(any(test, feature = "test-utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-utils")))]
pub mod test_utils;
mod worker;
pub mod workflow;

pub use db::{utils::Health, Db};
pub(crate) mod libp2p;
pub use logger::*;
pub(crate) mod metrics;
#[cfg(feature = "websocket-notify")]
pub use event_handler::notification::{network::NetworkNotification, receipt::ReceiptNotification};
#[allow(unused_imports)]
pub(crate) use event_handler::EventHandler;
pub use network::webserver::{listener, PrometheusData};
pub use receipt::{Receipt, RECEIPT_TAG, VERSION_KEY};
pub use runner::{NodeInfo, Runner};
pub(crate) use scheduler::TaskScheduler;
#[cfg(feature = "ipfs")]
pub use settings::IpfsBuilder;
pub use settings::{
    Autonat, DatabaseBuilder, Dht, ExistingKeyPath, KeyType, Libp2p, Mdns, MetricsBuilder,
    MonitoringBuilder, NetworkBuilder, NodeBuilder, PubkeyConfig, Pubsub, RNGSeed, Rendezvous,
    RpcBuilder, Settings, SettingsBuilder, WebserverBuilder,
};
pub(crate) use worker::Worker;
pub use workflow::WORKFLOW_TAG;
