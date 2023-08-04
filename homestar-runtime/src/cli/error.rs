//! Error type for CLI / CLI-interaction.

use crate::network::rpc;
use miette::{miette, Diagnostic};
use std::io;
use tarpc::client::RpcError;

/// Error types for CLI / CLI-interaction.
#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum Error {
    /// Generic CLI error.
    #[error("{error_message}")]
    Cli {
        /// Error message.
        error_message: String,
    },
    /// Propagated RPC error related to [tarpc::client::RpcError].
    #[error(transparent)]
    Rpc(#[from] RpcError),
    /// Propagated error related to an .
    #[error(transparent)]
    RpcMessage(#[from] rpc::Error),
    /// Propagated IO error.
    #[error("error writing data to console: {0}")]
    Io(#[from] io::Error),
}

impl Error {
    /// Create a new [Error].
    pub fn new(err: miette::ErrReport) -> Self {
        Error::Cli {
            error_message: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::new(miette!(e.to_string()))
    }
}
