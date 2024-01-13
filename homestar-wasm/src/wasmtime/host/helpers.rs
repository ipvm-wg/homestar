//! Helper functions that can be used in guest Wasm components.

use crate::wasmtime::{
    world::{homestar::host::helpers, wasi},
    State,
};
use std::time::Instant;

#[async_trait::async_trait]
impl helpers::Host for State {
    /// Get the current time.
    async fn get_current_time(&mut self) -> wasmtime::Result<helpers::Time> {
        let now = Instant::now();
        let duration = now.duration_since(self.start_time());
        Ok(helpers::Time {
            seconds: duration.as_secs(),
            milliseconds: duration.subsec_millis(),
            nanoseconds: duration.subsec_nanos(),
        })
    }

    /// Print a message.
    async fn print(&mut self, from_wasm: String) -> wasmtime::Result<()> {
        println!("{from_wasm}");
        Ok(())
    }
}

#[async_trait::async_trait]
impl wasi::logging::logging::Host for State {
    /// Log a message, formatted by the runtime subscriber.
    async fn log(
        &mut self,
        level: wasi::logging::logging::Level,
        context: String,
        message: String,
    ) -> wasmtime::Result<()> {
        match level {
            wasi::logging::logging::Level::Trace => {
                tracing::trace!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Debug => {
                tracing::debug!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Info => {
                tracing::info!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Warn => {
                tracing::warn!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Error => {
                tracing::error!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Critical => {
                tracing::error!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
        }
        Ok(())
    }
}
