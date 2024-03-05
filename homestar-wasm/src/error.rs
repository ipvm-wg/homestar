//! Error types and implementations for `Ipld <=> Wasm` interaction(s).

/// Error types related for Ipld to/from [Wasm value] interpretation.
///
/// [Wasm value]: wasmtime::component::Val
#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum InterpreterError {
    /// Error encoding structure to a Cid.
    #[error("failed to encode CID: {0}")]
    CidEncode(#[from] libipld::cid::Error),
    /// Error converting from [Decimal] precision to [f64].
    ///
    /// [Decimal]: rust_decimal::Decimal
    /// [f64]: f64
    #[error("Failed to convert from decimal to f64 float {0}")]
    DecimalToFloat(rust_decimal::Decimal),
    /// Error converting from from [f32] to [Decimal].
    ///
    /// [Decimal]: rust_decimal::Decimal
    #[error("Failed to convert from f32 float {0} to decimal")]
    FloatToDecimal(f32),
    /// Error converting from Ipld structure.
    #[error("cannot convert from Ipld structure: {0}")]
    FromIpld(#[from] libipld::error::SerdeError),
    /// Error casting from Ipld [i128] structure to a lower precision integer.
    #[error("failed to cast Ipld i128 to integer type: {0}")]
    IpldToInt(#[from] std::num::TryFromIntError),
    /// Error involving mismatches with Ipld mapping(s).
    /// Error converting from Ipld structure to [Wit] structure.
    ///
    /// [Wit]: wasmtime::component::Val
    #[error("incompatible Ipld type to Wit structural conversion: {0:#?}")]
    IpldToWit(String),
    /// Bubble-up [TagsError] errors while executing the interpreter.
    #[error(transparent)]
    Tags(#[from] TagsError),
    /// Type mismatch error between expected and given types.
    #[error("component type mismatch: expected: {expected}, found: {given:#?}")]
    TypeMismatch {
        /// Expected type.
        expected: String,
        /// Given type or lack thereof.
        given: Option<String>,
    },
    /// Failed to parse, handle, or otherwise convert to/from/between
    /// Wit/Wasm types.
    ///
    /// The underlying error is a [anyhow::Error], per the
    /// [wasmtime::component::types::Type] implementation.
    #[error(transparent)]
    WitType(#[from] anyhow::Error),
    /// Error converting from [Wit] structure to Ipld structure.
    ///
    /// [Wit]: wasmtime::component::Val
    #[error("no compatible WIT type for Ipld structure: {0:#?}")]
    WitToIpld(libipld::Ipld),
}

/// Error type for handling [Tags] stack structure.
///
/// [Tags]: crate::wasmtime::ipld::Tags
#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum TagsError {
    /// An error returned by [atomic_refcell::AtomicRefCell::try_borrow].
    #[error("{0}")]
    Borrow(atomic_refcell::BorrowError),
    /// An error returned by [atomic_refcell::AtomicRefCell::try_borrow_mut].
    #[error("{0}")]
    BorrowMut(atomic_refcell::BorrowMutError),
    /// Working with [Tags] stack structure should never be empty.
    ///
    /// [Tags]: crate::wasmtime::ipld::Tags
    #[error("structure must contain at least one element")]
    TagsEmpty,
}

impl From<atomic_refcell::BorrowError> for TagsError {
    fn from(e: atomic_refcell::BorrowError) -> Self {
        TagsError::Borrow(e)
    }
}

impl From<atomic_refcell::BorrowMutError> for TagsError {
    fn from(e: atomic_refcell::BorrowMutError) -> Self {
        TagsError::BorrowMut(e)
    }
}
