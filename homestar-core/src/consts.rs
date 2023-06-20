//! Exported global constants.

/// SemVer-formatted version of the UCAN Invocation Specification.
pub const INVOCATION_VERSION: &str = "0.2.0";
/// DagCbor codec.
pub const DAG_CBOR: u64 = 0x71;
/// 4GiB maximum memory for Wasm.
pub const WASM_MAX_MEMORY: u64 = byte_unit::n_gib_bytes!(4);
