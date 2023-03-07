//! Input paramters for [Task] execution.
//!
//! [Task]: super::Task

use crate::workflow::pointer::{
    Await, AwaitResult, InvokedTaskPointer, ERR_BRANCH, OK_BRANCH, PTR_BRANCH,
};
use anyhow::anyhow;
use libipld::{serde::from_ipld, Ipld};
use std::{collections::btree_map::BTreeMap, result::Result};

/// Contains parameters expected by the [URI]/[Ability] pair.
///
/// Left to the executor to define the shape of this data, per job.
///
/// [URI]: [url::Url]
/// [Ability]: super::Ability
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    /// Literals
    Ipld(Ipld),

    /// Values from another Task
    Deferred(Await),
}

impl From<Input> for Ipld {
    fn from(input: Input) -> Self {
        match input {
            Input::Ipld(ipld) => ipld,
            Input::Deferred(promise) => Await::into(promise),
        }
    }
}

impl From<Await> for Input {
    fn from(promise: Await) -> Self {
        Input::Deferred(promise)
    }
}

impl TryFrom<Ipld> for Input {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let Ok(map) = from_ipld::<BTreeMap<String, Ipld>>(ipld.to_owned()) else {
            return Ok(Input::Ipld(ipld))
        };

        map.get_key_value(OK_BRANCH)
            .or_else(|| map.get_key_value(ERR_BRANCH))
            .or_else(|| map.get_key_value(PTR_BRANCH))
            .map_or(Ok(Input::Ipld(ipld)), |(branch, ipld)| {
                let invoked_task = InvokedTaskPointer::try_from(ipld)?;
                Ok(Input::Deferred(Await::new(
                    invoked_task,
                    AwaitResult::result(branch)
                        .ok_or_else(|| anyhow!("wrong branch name: {branch}"))?,
                )))
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::workflow::{Ability, Nonce, Task};
    use url::Url;

    fn task<'a>() -> Task<'a> {
        let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
        let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();
        let nonce = Nonce::generate();

        Task::new(
            resource,
            Ability::from("wasm/run"),
            Input::Ipld(Ipld::List(vec![Ipld::Integer(88)])),
            Some(nonce.clone()),
        )
    }

    #[test]
    fn input_ipld_ipld_rountrip() {
        let input = Input::Ipld(Ipld::List(vec![Ipld::Bool(true)]));
        let ipld = Ipld::from(input.clone());

        assert_eq!(ipld, Ipld::List(vec![Ipld::Bool(true)]));
        assert_eq!(input, ipld.try_into().unwrap());
    }

    #[test]
    fn input_deferred_ipld_rountrip() {
        let task = task();
        let ptr: InvokedTaskPointer = task.try_into().unwrap();
        let input = Input::Deferred(Await::new(ptr.clone(), AwaitResult::Ptr));
        let ipld = Ipld::from(input.clone());

        assert_eq!(
            ipld,
            Ipld::Map(BTreeMap::from([(PTR_BRANCH.into(), Ipld::Link(ptr.cid()))]))
        );
        assert_eq!(input, ipld.try_into().unwrap());
    }
}
