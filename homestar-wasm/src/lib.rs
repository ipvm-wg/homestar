#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! homestar-wasm wraps and extends a [Wasmtime] runtime and acts as the defacto
//! execution engine for Homestar.
//!
//! Related crates/packages:
//!
//! - [homestar-core]
//! - [homestar-runtime]
//!
//! [homestar-core]: homestar_core
//! [homestar-runtime]: <https://docs.rs/homestar-runtime>
//! [Wasmtime]: <https://wasmtime.dev/>

pub mod error;
pub mod io;
pub mod test_utils;
pub mod wasmtime;
