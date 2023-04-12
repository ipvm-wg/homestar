#![allow(missing_docs)]

//! Module for working with task-types and task-specific functionality.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use enum_assoc::Assoc;
use std::path::PathBuf;

mod wasm;

pub(crate) use wasm::*;

const WASM_OP: &str = "wasm/run";

/// First-class registered task-types.
#[allow(missing_debug_implementations)]
#[derive(Assoc)]
#[func(pub fn ability(s: &str) -> Option<Self>)]
pub enum RegisteredTasks {
    /// Basic `wasm/run` task-type.
    #[assoc(ability = WASM_OP)]
    WasmRun,
}

/// Trait for loading files for different task-types directly.
#[async_trait]
pub trait FileLoad {
    /// Load file asynchronously.
    async fn load(file: PathBuf) -> Result<Vec<u8>> {
        tokio::fs::read(file).await.map_err(|e| anyhow!(e))
    }
}
