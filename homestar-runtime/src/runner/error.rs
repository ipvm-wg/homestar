//! Error types related to the Homestar runtime/[Runner].
//!
//! [Runner]: crate::Runner

use std::io;

/// Error types related to running [Workflow]s and other runtime
/// components.
///
/// [Workflow]: homestar_core::Workflow
#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    /// Unsupported workflow type.
    #[error("unsupported workflow file type: {0}")]
    UnsupportedWorkflow(String),
    /// Propagated IO error.
    #[error("error reading data: {0}")]
    Io(#[from] io::Error),
    /// Propagated, general runtime error.
    #[error(transparent)]
    Runtime(#[from] anyhow::Error),
}
