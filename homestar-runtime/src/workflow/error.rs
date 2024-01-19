//! Error type for workflow reception and execution.

/// Error types for for workflow reception and execution.
#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    /// Duplicate task error.
    #[error(
        "workflow cannot contain duplicate tasks: use a nonce (nnc field) to ensure uniqueness"
    )]
    DuplicateTask,
    /// Invalid schedule error.
    #[error("Schedule could not be generated from workflow: {0}")]
    InvalidSchedule(String),
    /// Generic ahead-of-time error.
    ///
    /// Transparently forwards from [anyhow::Error]'s `source` and
    /// `Display` methods through to an underlying error.
    #[error(transparent)]
    AoT(#[from] anyhow::Error),
}
