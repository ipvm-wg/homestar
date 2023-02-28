//! [Wasmtime]-specific modules for [Ipld] conversion and component setup
//! and execution.
//!
//! [Wasmtime]: <https://wasmtime.dev/>
//! [Ipld]: libipld::Ipld

/// [Ipld] <=> [wasmtime::component::Val] IR.
///
/// [Ipld]: libipld::Ipld
pub mod ipld;

/// Wasmtime component initialzation and execution of Wasm function(s).
mod world;
pub use world::*;
