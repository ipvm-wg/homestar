//! Configuration for Wasm/wasmtime execution.

use crate::wasmtime::{self, limits::StoreLimitsAsync};
use homestar_core::{consts, workflow::config::Resources};

impl From<Resources> for wasmtime::State {
    fn from(resources: Resources) -> wasmtime::State {
        wasmtime::State::new(
            resources.fuel().unwrap_or(u64::MAX),
            StoreLimitsAsync::new(Some(consts::WASM_MAX_MEMORY as usize), None),
        )
    }
}
