//! Exported `unit` type for generic type conversions around
//! [Tasks], [Inputs], and [Invocations].
//!
//! [Tasks]: crate::workflow::Task
//! [Inputs]: crate::workflow::Input
//! [Invocations]: crate::workflow::Invocation
//!

use crate::workflow::{
    error::InputParseError,
    input::{self, Args, Parsed},
    Error, Input,
};
use libipld::Ipld;

/// Unit type, which allows only one value (and thusly holds
/// no information). Essentially a wrapper over `()`, but one we control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit;

impl From<Unit> for Ipld {
    fn from(_unit: Unit) -> Self {
        Ipld::Null
    }
}

impl From<Ipld> for Unit {
    fn from(_ipld: Ipld) -> Self {
        Unit
    }
}

// Default implementation.
impl input::Parse<Unit> for Input<Unit> {
    fn parse(&self) -> Result<Parsed<Unit>, InputParseError<Unit>> {
        let args = match Ipld::try_from(self.to_owned())? {
            Ipld::List(v) => Ipld::List(v).try_into()?,
            ipld => Args::new(vec![ipld.try_into()?]),
        };

        Ok(Parsed::with(args))
    }
}

impl From<Error<String>> for InputParseError<Unit> {
    fn from(err: Error<String>) -> Self {
        InputParseError::WorkflowError(err.into())
    }
}
