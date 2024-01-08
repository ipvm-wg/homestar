//! [Wasmtime]-specific modules for [Ipld] conversion and component setup
//! and execution.
//!
//! [Wasmtime]: <https://wasmtime.dev/>
//! [Ipld]: libipld::Ipld

pub mod config;
mod error;
mod host;
pub mod ipld;
pub mod limits;
pub mod world;

pub use error::*;
pub use world::{State, World};
