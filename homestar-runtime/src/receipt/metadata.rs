//! Metadata related to [receipts].
//!
//! [receipts]: crate::Receipt

/// Metadata attributed to a boolean true/false value on whether
/// the computation was executed from scratch or not.
pub(crate) const REPLAYED_KEY: &str = "replayed";

/// Metadata key for a workflow Cid.
pub(crate) const WORKFLOW_KEY: &str = "workflow";

/// Associated metadata key for a workflow name, which
/// will either be some identifier, or the Cid of the workflow.
pub(crate) const WORKFLOW_NAME_KEY: &str = "name";
