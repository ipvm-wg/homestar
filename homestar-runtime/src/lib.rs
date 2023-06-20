#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub, private_in_public)]

//! homestar-runtime is a determistic Wasm runtime and effectful workflow/job
//! system intended to be embedded inside or run alongside IPFS.
//!
//! You can find a more complete description [here].
//!
//!
//! Related crates/packages:
//!
//! - [homestar-core]
//! - [homestar-wasm]
//!
//! [here]: <https://github.com/ipvm-wg/spec>
//! [homestar-core]: homestar_core
//! [homestar-wasm]: homestar_wasm

pub mod cli;
pub mod db;
pub mod logger;
pub mod network;
mod receipt;
mod runtime;
pub mod scheduler;
mod settings;
pub mod tasks;
mod worker;
pub mod workflow;

pub use db::Db;
#[cfg(feature = "ipfs")]
pub use network::ipfs::IpfsCli;
pub use receipt::{Receipt, RECEIPT_TAG, VERSION_KEY};
pub use runtime::*;
pub use settings::Settings;
pub use worker::Worker;

/// Test utilities.
#[cfg(any(test, feature = "test_utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test_utils")))]
pub mod test_utils;
