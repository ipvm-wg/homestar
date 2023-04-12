//! Workflow and [Ucan invocation] componets for building Homestar pipelines.
//!
//! [Ucan invocation]: <https://github.com/ucan-wg/invocation>

mod ability;
pub mod config;
pub mod input;
pub mod instruction;
mod instruction_result;
mod invocation;
mod issuer;
mod nonce;
pub mod pointer;
pub mod prf;
pub mod receipt;
pub mod task;

pub use ability::*;
pub use input::Input;
pub use instruction::Instruction;
pub use instruction_result::*;
pub use invocation::*;
pub use issuer::Issuer;
pub use nonce::*;
pub use pointer::Pointer;
pub use receipt::Receipt;
pub use task::Task;

/// Generic link, cid => T [IndexMap] for storing
/// invoked, raw values in-memory and using them to
/// resolve other steps within a runtime's workflow.
///
/// [IndexMap]: indexmap::IndexMap
pub type LinkMap<T> = indexmap::IndexMap<libipld::Cid, T>;
