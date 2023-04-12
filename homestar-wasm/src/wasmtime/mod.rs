//! [Wasmtime]-specific modules for [Ipld] conversion and component setup
//! and execution.
//!
//! [Wasmtime]: <https://wasmtime.dev/>
//! [Ipld]: libipld::Ipld

pub mod config;
pub mod ipld;
pub mod world;

pub use world::{State, World};
