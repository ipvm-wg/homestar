#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! `homestar-invocation` is an underlying foundation for Homestar
//! packages, implementing much of the [Ucan Invocation]
//! specification, among other useful library features.
//!
//! ## Related crates/packages:
//!
//! - [homestar-runtime]
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
//! - `diesel`: Enables diesel-sqlite implementations of data structures.
//! - `test-utils`: Enables utilities for unit testing and benchmarking.
//!
//! [homestar-runtime]: https://docs.rs/homestar-runtime
//! [homestar-wasm]: https://docs.rs/homestar-wasm
//! [homestar-workflow]: https://docs.rs/homestar-workflow
//! [IPVM]: https://github.com/ipvm-wg
//! [Quickstart]: https://github.com/ipvm-wg/homestar/blob/main/README.md#quickstart
//! [README]: https://github.com/ipvm-wg/homestar/blob/main/README.md
//! [Ucan invocation]: https://github.com/ucan-wg/invocation

pub mod authority;
pub mod consts;
pub mod error;
mod invocation;
pub mod ipld;
pub mod macros;
pub mod pointer;
pub mod receipt;
pub mod task;
#[cfg(any(test, feature = "test-utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-utils")))]
pub mod test_utils;
pub mod unit;

pub use consts::*;
pub use error::Error;
pub use invocation::Invocation;
pub use pointer::Pointer;
pub use receipt::Receipt;
pub use task::Task;
pub use unit::*;
