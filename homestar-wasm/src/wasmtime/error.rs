//! Error types and implementations for Wasm (via [Wasmtime]) execution,
//! instantiation, and runtime interaction(s).
//!
//! [Wasmtime]: <https://docs.rs/wasmtime/latest/wasmtime>

/// Generic error type for Wasm execution, conversions, instantiations, etc.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to convert from Wasm binary into Wasm component.
    #[error("cannot convert from binary structure to Wasm component")]
    IntoWasmComponentError(#[source] anyhow::Error),
    /// Bubble-up [ResolveError]s for [Cid]s still awaiting resolution.
    ///
    /// [ResolveError]: homestar_core::workflow::error::ResolveError
    /// [Cid]: libipld::Cid
    #[error(transparent)]
    PromiseError(#[from] homestar_core::workflow::error::ResolveError),
    /// Generic unknown error.
    #[error("unknown error")]
    UnknownError,
    /// Failure to instantiate Wasm component and its host bindings.
    #[error("bindings not yet instantiated for wasm environment")]
    WasmInstantiationError,
    /// Failure to parse Wasm binary.
    ///
    /// Transparently forwards from [wasmparser::BinaryReaderError]'s `source`
    /// and `Display` methods through to an underlying error.
    #[error(transparent)]
    WasmParserError(#[from] wasmparser::BinaryReaderError),
    /// Generic [wasmtime] runtime error.
    ///
    /// Transparently forwards from [anyhow::Error]'s `source` and
    /// `Display` methods through to an underlying error.
    #[error(transparent)]
    WasmRuntimeError(#[from] anyhow::Error),
    /// Failure to find Wasm function for execution.
    #[error("Wasm function {0} not found")]
    WasmFunctionNotFoundError(String),
    /// [Wat] as Wasm component error.
    ///
    /// [Wat]: wat
    #[error("{0}")]
    WatComponentError(String),
    /// [wat]-related error.
    ///
    /// Transparently forwards from [wat::Error]'s `source`
    /// and `Display` methods through to an underlying error.
    #[error(transparent)]
    WatError(#[from] wat::Error),
}
