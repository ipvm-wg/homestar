//! IO (input/output) types for the Wasm execution.

use crate::{error::InterpreterError, wasmtime::ipld::RuntimeVal};
use enum_as_inner::EnumAsInner;
use homestar_invocation::{
    error::InputParseError,
    task::instruction::{Args, Input, Parse, Parsed},
    Error as InvocationError,
};
use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::btree_map::BTreeMap, fmt};
use wasmtime;

/// Argument for Wasm execution, which can either be
/// an [Ipld] structure or a [wasmtime::component::Val].
#[derive(Clone, Debug, PartialEq, EnumAsInner, Serialize, Deserialize)]
pub enum Arg {
    /// [Ipld] structure, which can be interpreted into a Wasm [Val].
    ///
    /// [Val]: wasmtime::component::Val
    Ipld(Ipld),
    /// A direct [Wasm value] as argument input.
    ///
    /// [Wasm value]: wasmtime::component::Val
    #[serde(skip)]
    Value(wasmtime::component::Val),
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Ipld(ipld) => write!(f, "{ipld:?}"),
            Arg::Value(v) => write!(f, "{v:?}"),
        }
    }
}

impl From<Ipld> for Arg {
    fn from(ipld: Ipld) -> Self {
        Arg::Ipld(ipld)
    }
}

impl From<Arg> for Ipld {
    fn from(arg: Arg) -> Self {
        match arg {
            Arg::Ipld(ipld) => ipld,
            Arg::Value(v) => {
                if let Ok(ipld) = Ipld::try_from(RuntimeVal::new(v)) {
                    ipld
                } else {
                    Ipld::Null
                }
            }
        }
    }
}

impl Parse<Arg> for Input<Arg> {
    fn parse(&self) -> Result<Parsed<Arg>, InputParseError<Arg>> {
        if let Input::Ipld(ref ipld) = self {
            let map = from_ipld::<BTreeMap<String, Ipld>>(ipld.to_owned())?;

            let func = map.get("func").ok_or_else(|| {
                InputParseError::Invocation(InvocationError::MissingField("func".to_string()))
            })?;

            let wasm_args = map.get("args").ok_or_else(|| {
                InputParseError::Invocation(InvocationError::MissingField("args".to_string()))
            })?;

            let args: Args<Arg> = wasm_args.to_owned().try_into()?;
            Ok(Parsed::with_fn(from_ipld::<String>(func.to_owned())?, args))
        } else {
            Err(InputParseError::UnexpectedTaskInput(self.clone()))
        }
    }
}

/// Enumeration of possible outputs from Wasm execution.
#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    /// A singular [Wasm value] as output.
    ///
    /// [Wasm value]: wasmtime::component::Val
    Value(wasmtime::component::Val),
    /// A list of [Wasm values] as output.
    ///
    /// [Wasm value]: wasmtime::component::Val
    Values(Vec<wasmtime::component::Val>),
    /// No output, treated as `void`.
    Void,
}

impl Output {
    /// Take the first [Wasm value] from the output, unless void.
    pub fn take(self) -> Option<wasmtime::component::Val> {
        match self {
            Output::Value(v) => Some(v),
            Output::Values(vs) => vs.into_iter().next(),
            Output::Void => None,
        }
    }
}

impl TryFrom<Output> for Ipld {
    type Error = InterpreterError;

    fn try_from(output: Output) -> Result<Self, Self::Error> {
        match output {
            Output::Value(v) => Ipld::try_from(RuntimeVal::new(v)),
            Output::Values(vs) => {
                let ipld_vs = vs.into_iter().try_fold(vec![], |mut acc, v| {
                    let ipld = Ipld::try_from(RuntimeVal::new(v))?;
                    acc.push(ipld);
                    Ok::<_, Self::Error>(acc)
                })?;
                Ok(Ipld::List(ipld_vs))
            }
            Output::Void => Ok(Ipld::Null),
        }
    }
}
