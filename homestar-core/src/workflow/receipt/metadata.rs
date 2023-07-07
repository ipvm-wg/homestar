//! Metadata related to [receipts].
//!
//! [receipts]: crate::workflow::Receipt

/// Metadata key for an operation or function name.
pub const OP_KEY: &str = "op";

/// Metadata key for a workflow [Cid].
///
/// [Cid]: libipld::Cid
pub const WORKFLOW_KEY: &str = "workflow";
