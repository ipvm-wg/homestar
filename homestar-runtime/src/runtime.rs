//! General [Runtime] for working across multiple workers
//! and workflows.
//!
//! TODO: Fill this out.

use homestar_wasm::io::Arg;
use tokio::task::JoinSet;

/// Runtime for starting workers on workflows.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Runtime {
    /// The set of [workers] for [workflows]
    ///
    /// [workers]: crate::Worker
    /// [workflows]: crate::Workflow
    pub(crate) workers: JoinSet<anyhow::Result<Arg>>,
}
