#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! `homestar-workflow` is the underlying foundation for homestar
//! packages, implementing workflow-centric [IPVM]
//! features, among other useful library components.
//!
//! *Note*: To be used in conjunction with [homestar-invocation].
//!
//! Related crates/packages:
//!
//! - [homestar-invocation]
//! - [homestar-runtime]
//! - [homestar-wasm]
//!
//! ## Getting Started
//!
//! For getting started with Homestar in general, please check out our
//! [README] and [Quickstart] guide.
//!
//! [homestar-invocation]: <https://docs.rs/homestar-invocation>
//! [homestar-runtime]: <https://docs.rs/homestar-runtime>
//! [homestar-wasm]: <https://docs.rs/homestar-wasm>
//! [IPVM]: <https://github.com/ipvm-wg/spec>
//! [Quickstart]: https://github.com/ipvm-wg/homestar/blob/main/README.md#quickstart
//! [README]: https://github.com/ipvm-wg/homestar/blob/main/README.md

mod linkmap;
pub mod workflow;

pub use linkmap::LinkMap;
pub use workflow::Workflow;
