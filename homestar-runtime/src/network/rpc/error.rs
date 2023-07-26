//! Error types related to the RPC server / client interface(s).

use serde::{Deserialize, Serialize};

/// Error types related to the RPC server interface.
#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum Error {
    /// Error when attempting to send data on a channel.
    #[error("{0}")]
    FailureToSendOnChannel(String),
    /// Error when attempting to receive data on a channel.
    #[error("{0}")]
    FailureToReceiveOnChannel(String),
    /// Error when attempting to run a workflow via the [Runner].
    ///
    /// [Runner]: crate::Runner
    #[error("runtime error: {0}")]
    FromRunner(String),
}
