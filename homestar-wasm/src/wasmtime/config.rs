//! Configuration for Wasm/wasmtime execution.

use crate::wasmtime;
use homestar_core::workflow::config::Resources;

impl From<Resources> for wasmtime::State {
    fn from(resources: Resources) -> wasmtime::State {
        wasmtime::State::new(resources.fuel().unwrap_or(u64::MAX))
    }
}
