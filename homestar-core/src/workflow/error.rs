//! Error types and implementations for [Workflow] interaction(s).
//!
//! [Workflow]: crate::Workflow

use crate::{
    workflow::{input::Args, Input},
    Unit,
};
use libipld::Ipld;
use serde::de::Error as DeError;

/// Generic error type for [Workflow] use cases.
///
/// [Workflow]: crate::Workflow
#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    /// Error encoding structure to a [Cid].
    ///
    /// [Cid]: libipld::cid::Cid
    #[error("failed to encode CID: {0}")]
    CidError(#[from] libipld::cid::Error),
    /// Error thrown when condition or dynamic check is not met.
    #[error("condition not met: {0}")]
    ConditionNotMet(String),
    /// Failure to decode/encode from/to DagCbor.
    ///
    /// The underlying error is a [anyhow::Error], per the
    /// [DagCborCodec] implementation.
    ///
    /// [DagCborCodec]: libipld::cbor::DagCborCodec
    #[error("failed to decode/encode DAG-CBOR: {0}")]
    DagCborTranslationError(#[from] anyhow::Error),
    /// Error converting from [Ipld] structure.
    #[error("cannot convert from Ipld structure: {0}")]
    FromIpldError(#[from] libipld::error::SerdeError),
    /// Invalid match discriminant or enumeration.
    #[error("invalid discriminant {0:#?}")]
    InvalidDiscriminant(T),
    /// Error related to a missing a field in a structure or key
    /// in a map.
    #[error("no {0} field set")]
    MissingFieldError(String),
    /// Error during parsing a [Url].
    ///
    /// Transparently forwards from [url::ParseError]'s `source` and
    /// `Display` methods through to an underlying error.
    ///
    /// [Url]: url::Url
    #[error(transparent)]
    ParseResourceError(#[from] url::ParseError),
    /// Generic unknown error.
    #[error("unknown error")]
    UnknownError,
    /// Error when attempting to interpret a sequence of [u8]
    /// as a string.
    ///
    /// Transparently forwards from [std::str::Utf8Error]'s `source` and
    /// `Display` methods through to an underlying error.
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl<T> Error<T> {
    /// Return a [SerdeError] when returning an [Ipld] structure
    /// that's not expected at the call-site.
    ///
    /// [SerdeError]: libipld::error::SerdeError
    pub fn unexpected_ipld(ipld: Ipld) -> Self {
        Error::FromIpldError(libipld::error::SerdeError::custom(format!(
            "unexpected Ipld conversion: {ipld:#?}"
        )))
    }

    /// Return an `invalid type` [SerdeError] when not matching an expected
    /// [Ipld] list/sequence type.
    ///
    /// [SerdeError]: libipld::error::SerdeError
    pub fn not_an_ipld_list() -> Self {
        Error::FromIpldError(libipld::error::SerdeError::invalid_type(
            serde::de::Unexpected::Seq,
            &"an Ipld list / sequence",
        ))
    }
}

impl From<Error<Unit>> for Error<String> {
    fn from(_err: Error<Unit>) -> Self {
        Error::UnknownError
    }
}

impl From<Error<String>> for Error<Unit> {
    fn from(_err: Error<String>) -> Error<Unit> {
        Error::UnknownError
    }
}

impl<T> From<std::convert::Infallible> for Error<T> {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}

/// Error type for parsing [Workflow] [Input]s.
///
/// [Workflow]: crate::Workflow
#[derive(thiserror::Error, Debug)]
pub enum InputParseError<T> {
    /// Error converting from [Ipld] structure.
    #[error("cannot convert from Ipld structure: {0}")]
    FromIpldError(#[from] libipld::error::SerdeError),
    /// Error converting from [Ipld] structure into [Args].
    #[error("cannot convert from Ipld structure into arguments: {0:#?}")]
    IpldToArgsError(Args<T>),
    /// Unexpected [Input] in [Task] structure.
    ///
    /// [Task]: crate::workflow::Task
    #[error("unexpected task input: {0:#?}")]
    UnexpectedTaskInput(Input<T>),
    /// Bubble-up conversion and other general [Workflow errors].
    ///
    /// [Workflow errors]: Error
    #[error(transparent)]
    WorkflowError(#[from] Error<T>),
}

impl<T> From<std::convert::Infallible> for InputParseError<T> {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}

/// Error type for resolving promised [Cid]s within [Workflow] [Input]s.
///
/// [Cid]: libipld::Cid
/// [Workflow]: crate::Workflow
#[derive(thiserror::Error, Debug)]
pub enum ResolveError {
    /// Generic runtime error.
    ///
    /// Transparently forwards from [anyhow::Error]'s `source` and
    /// `Display` methods through to an underlying error.
    #[error(transparent)]
    RuntimeError(#[from] anyhow::Error),
    /// Transport error when attempting to resolve [Workflow] [Input]'s [Cid].
    ///
    /// [Cid]: libipld::Cid
    /// [Workflow]: crate::Workflow
    #[error("transport error during resolve phase of input Cid: {0}")]
    TransportError(String),
    /// Unable to resolve a [Cid] within a [Workflow]'s [Input].
    ///
    /// [Cid]: libipld::Cid
    /// [Workflow]: crate::Workflow
    #[error("error resolving input Cid: {0}")]
    UnresolvedCidError(String),
}

impl From<std::convert::Infallible> for ResolveError {
    fn from(err: std::convert::Infallible) -> Self {
        match err {}
    }
}
