#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub, private_in_public)]

//! homestar-wasm is enables a Wasm runtime and execution engine for Homestar.

/// All interaction with [wasmtime] runtime, types, and values.
pub mod wasmtime;

/// Test utilities.
pub mod test_utils;
