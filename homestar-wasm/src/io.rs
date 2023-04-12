//! IO (input/output) types for the Wasm execution.

use anyhow::anyhow;
use enum_as_inner::EnumAsInner;
use homestar_core::workflow::{
    input::{self, Args, Parsed},
    Input,
};
use libipld::{serde::from_ipld, Ipld};
use std::{collections::btree_map::BTreeMap, fmt};
use wasmtime;

use crate::wasmtime::ipld::RuntimeVal;

///
#[derive(Clone, Debug, PartialEq, EnumAsInner)]
pub enum Arg {
    ///
    Ipld(Ipld),
    ///
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

impl input::Parse<Arg> for Input<Arg> {
    fn parse(&self) -> anyhow::Result<Parsed<Arg>> {
        if let Input::Ipld(ref ipld) = self {
            let map = from_ipld::<BTreeMap<String, Ipld>>(ipld.to_owned())?;

            let func = map
                .get("func")
                .ok_or_else(|| anyhow!("wrong task input format: {ipld:?}"))?;

            let wasm_args = map
                .get("args")
                .ok_or_else(|| anyhow!("wrong task input format: {ipld:?}"))?;

            let args: Args<Arg> = wasm_args.to_owned().try_into()?;
            Ok(Parsed::with_fn(from_ipld::<String>(func.to_owned())?, args))
        } else {
            Err(anyhow!("unexpected task input"))
        }
    }
}

///
#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    ///
    Value(wasmtime::component::Val),
    ///
    Values(Vec<wasmtime::component::Val>),
    ///
    Void,
}

impl TryFrom<Output> for Ipld {
    type Error = anyhow::Error;

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
