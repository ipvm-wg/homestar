//! Error types and implementations for Wasm (via [Wasmtime]) execution,
//! instantiation, and runtime interaction(s).
//!
//! [Wasmtime]: <https://docs.rs/wasmtime/latest/wasmtime>

/// Generic error type for Wasm execution, conversions, instantiations, etc.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Bubble-up [InterpreterError]s for Wasm <=> [Ipld] execution.
    ///
    /// [InterpreterError]: crate::error::InterpreterError
    /// [Ipld]: libipld::Ipld
    #[error(transparent)]
    InterpreterError(#[from] crate::error::InterpreterError),
    /// Failure to convert from Wasm binary into Wasm component.
    #[error("cannot convert from binary structure to Wasm component")]
    IntoWasmComponent(#[source] anyhow::Error),
    /// Bubble-up [ResolveError]s for [Cid]s still awaiting resolution.
    ///
    /// [ResolveError]: homestar_core::workflow::error::ResolveError
    /// [Cid]: libipld::Cid
    #[error(transparent)]
    ResolvePromise(#[from] homestar_core::workflow::error::ResolveError),
    /// Generic unknown error.
    #[error("unknown error")]
    Unknown,
    /// Failure to instantiate Wasm component and its host bindings.
    #[error("bindings not yet instantiated for wasm environment")]
    WasmInstantiation,
    /// Failure to parse Wasm binary.
    ///
    /// Transparently forwards from [wasmparser::BinaryReaderError]'s `source`
    /// and `Display` methods through to an underlying error.
    #[error(transparent)]
    WasmParser(#[from] wasmparser::BinaryReaderError),
    /// Generic [wasmtime] runtime error.
    ///
    /// Transparently forwards from [anyhow::Error]'s `source` and
    /// `Display` methods through to an underlying error.
    #[error(transparent)]
    WasmRuntime(#[from] anyhow::Error),
    /// Failure to find Wasm function for execution.
    #[error("Wasm function {0} not found in given Wasm component/resource")]
    WasmFunctionNotFound(String),
    /// [Wat] as Wasm component error.
    ///
    /// [Wat]: wat
    #[error("{0}")]
    WatComponent(String),
    /// [wat]-related error.
    ///
    /// Transparently forwards from [wat::Error]'s `source`
    /// and `Display` methods through to an underlying error.
    #[error(transparent)]
    Wat(#[from] wat::Error),
}
