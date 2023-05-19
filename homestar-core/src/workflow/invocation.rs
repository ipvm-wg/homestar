//! [Invocation] is a signed [Task].
//!
//! [Task]: super::Task

use super::{Pointer, Task};
use anyhow::anyhow;
use libipld::{
    cbor::DagCborCodec,
    cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    },
    prelude::Codec,
    serde::from_ipld,
    Ipld,
};
use std::collections::BTreeMap;

const DAG_CBOR: u64 = 0x71;
const TASK_KEY: &str = "task";

/// A signed [Task] wrapper/container.
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation<'a, T> {
    task: Task<'a, T>,
}

impl<'a, T> From<Task<'a, T>> for Invocation<'a, T>
where
    Ipld: From<T>,
{
    fn from(task: Task<'a, T>) -> Self {
        Invocation::new(task)
    }
}

impl<'a, T> Invocation<'a, T>
where
    Ipld: From<T>,
{
    /// Create a new [Invocation] container.
    pub fn new(task: Task<'a, T>) -> Self {
        Self { task }
    }
}

impl<T> TryFrom<Invocation<'_, T>> for Ipld
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(invocation: Invocation<'_, T>) -> Result<Self, Self::Error> {
        let map = Ipld::Map(BTreeMap::from([(
            TASK_KEY.into(),
            invocation.task.try_into()?,
        )]));

        Ok(map)
    }
}

impl<T> TryFrom<Ipld> for Invocation<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        Ok(Self {
            task: Task::try_from(
                map.get(TASK_KEY)
                    .ok_or_else(|| anyhow!("no `task` set"))?
                    .to_owned(),
            )?,
        })
    }
}

impl<T> TryFrom<Invocation<'_, T>> for Pointer
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(invocation: Invocation<'_, T>) -> Result<Self, Self::Error> {
        Ok(Pointer::new(Cid::try_from(invocation)?))
    }
}

impl<T> TryFrom<Invocation<'_, T>> for Cid
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(invocation: Invocation<'_, T>) -> Result<Self, Self::Error> {
        let ipld: Ipld = invocation.try_into()?;
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(DAG_CBOR, hash))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
        Unit,
    };

    #[test]
    fn ipld_roundtrip() {
        let config = Resources::default();
        let instruction = test_utils::workflow::instruction::<Unit>();
        let task = Task::new(
            RunInstruction::Expanded(instruction.clone()),
            config.into(),
            UcanPrf::default(),
        );

        let invocation = Invocation::new(task);
        let ipld = Ipld::try_from(invocation.clone()).unwrap();
        assert_eq!(invocation, Invocation::try_from(ipld).unwrap());
    }
}
