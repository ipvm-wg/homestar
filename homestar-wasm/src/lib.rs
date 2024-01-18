#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! homestar-wasm wraps and extends a [Wasmtime] runtime and acts as the defacto
//! execution engine for Homestar.
//!
//! Related crates/packages:
//!
//! - [homestar-invocation]
//! - [homestar-runtime]
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
//! - `test-utils`: Enables utilities for unit testing and benchmarking.
//!
//! [homestar-invocation]: <https://docs.rs/homestar-invocation>
//! [homestar-runtime]: <https://docs.rs/homestar-runtime>
//! [homestar-workflow]: <https://docs.rs/homestar-workflow>
//! [Quickstart]: https://github.com/ipvm-wg/homestar/blob/main/README.md#quickstart
//! [README]: https://github.com/ipvm-wg/homestar/blob/main/README.md
//! [Wasmtime]: <https://wasmtime.dev/>

pub mod error;
pub mod io;
#[cfg(any(test, feature = "test-utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-utils")))]
pub mod test_utils;
pub mod wasmtime;
