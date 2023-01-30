#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub, private_in_public)]

//! IPVM is a determistic Wasm runtime and effectful job system intended to embed inside IPFS.
//! You can find a more complete description [here](https://github.com/ipvm-wg/spec).

pub mod cli;
pub mod db;
pub mod network;
pub mod wasm;
pub mod workflow;

/// Test utilities.
#[cfg(any(test, feature = "test_utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test_utils")))]
pub mod test_utils;

#[cfg(test)]
mod tests {}
