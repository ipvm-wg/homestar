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
pub mod daemon;
pub mod db;
mod event_handler;
mod logger;
pub mod network;
mod receipt;
pub mod runner;
mod scheduler;
mod settings;
mod tasks;
mod worker;
pub mod workflow;

pub use db::Db;
pub use event_handler::channel;
pub use logger::*;
pub use receipt::{Receipt, RECEIPT_TAG, VERSION_KEY};
pub use runner::Runner;
pub use settings::Settings;
pub use workflow::WORKFLOW_TAG;

/// Test utilities.
#[cfg(any(test, feature = "test_utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test_utils")))]
pub mod test_utils;
