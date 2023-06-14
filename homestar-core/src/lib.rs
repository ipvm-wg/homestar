#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub, private_in_public)]

//! `homestar-core` is the underlying foundation for all homestar
//! packages and implements much of the [Ucan invocation] and [IPVM]
//! specifications, among other useful library features.
//!
//!
//! Related crates/packages:
//!
//! - [homestar-runtime]
//! - [homestar-wasm]
//!
//! [homestar-runtime]: <https://docs.rs/homestar-runtime>
//! [homestar-wasm]: <https://docs.rs/homestar-wasm>
//! [IPVM]: <https://github.com/ipvm-wg/spec>
//! [Ucan invocation]: <https://github.com/ucan-wg/invocation>

pub mod consts;
pub mod macros;
#[cfg(any(test, feature = "test_utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test_utils")))]
pub mod test_utils;
mod unit;

pub mod workflow;
pub use consts::*;
pub use unit::*;
pub use workflow::Workflow;
