//! An [Instruction] is the smallest unit of work that can be requested from a
//! UCAN, described via `resource`, `ability`.

use super::{Ability, Input, Nonce, Pointer};
use anyhow::anyhow;
use libipld::{
    cbor::DagCborCodec,
    cid::{
        multibase::Base,
        multihash::{Code, MultihashDigest},
        Cid,
    },
    prelude::Codec,
    serde::from_ipld,
    Ipld,
};
use std::{borrow::Cow, collections::BTreeMap, fmt};
use url::Url;

const DAG_CBOR: u64 = 0x71;
const RESOURCE_KEY: &str = "rsc";
const OP_KEY: &str = "op";
const INPUT_KEY: &str = "input";
const NNC_KEY: &str = "nnc";

/// Enumerator for `either` an expanded [Instruction] structure or
/// an [Pointer] ([Cid] wrapper).
#[derive(Debug, Clone, PartialEq)]
pub enum RunInstruction<'a, T> {
    /// [Instruction] as an expanded structure.
    Expanded(Instruction<'a, T>),
    /// [Instruction] as a pointer.
    Ptr(Pointer),
}

impl<'a, T> From<Instruction<'a, T>> for RunInstruction<'a, T> {
    fn from(instruction: Instruction<'a, T>) -> Self {
        RunInstruction::Expanded(instruction)
    }
}

impl<'a, T> TryFrom<RunInstruction<'a, T>> for Instruction<'a, T>
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Expanded(instruction) => Ok(instruction),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<T> From<Pointer> for RunInstruction<'_, T> {
    fn from(ptr: Pointer) -> Self {
        RunInstruction::Ptr(ptr)
    }
}

impl<'a, T> TryFrom<RunInstruction<'a, T>> for Pointer
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Ptr(ptr) => Ok(ptr),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<'a, 'b, T> TryFrom<&'b RunInstruction<'a, T>> for &'b Pointer
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: &'b RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Ptr(ptr) => Ok(ptr),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<'a, 'b, T> TryFrom<&'b RunInstruction<'a, T>> for Pointer
where
    T: fmt::Debug,
{
    type Error = anyhow::Error;

    fn try_from(run: &'b RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Ptr(ptr) => Ok(ptr.to_owned()),
            e => Err(anyhow!("wrong discriminant: {e:?}")),
        }
    }
}

impl<T> From<RunInstruction<'_, T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(run: RunInstruction<'_, T>) -> Self {
        match run {
            RunInstruction::Expanded(instruction) => instruction.into(),
            RunInstruction::Ptr(instruction_ptr) => instruction_ptr.into(),
        }
    }
}

impl<T> TryFrom<Ipld> for RunInstruction<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from<'a>(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(_) => Ok(RunInstruction::Expanded(Instruction::try_from(ipld)?)),
            Ipld::Link(_) => Ok(RunInstruction::Ptr(Pointer::try_from(ipld)?)),
            _ => Err(anyhow!("unexpected conversion type")),
        }
    }
}

///
///
/// # Example
///
/// ```
/// use homestar_core::{Unit, workflow::{Ability, Input, Instruction}};
/// use libipld::Ipld;
/// use url::Url;
///
/// let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
/// let resource = Url::parse(format!("ipfs://{wasm}").as_str()).unwrap();
///
/// let instr = Instruction::unique(
///     resource,
///     Ability::from("wasm/run"),
///     Input::<Unit>::Ipld(Ipld::List(vec![Ipld::Bool(true)]))
/// );
/// ```
///
/// We can also set-up an [Instruction] with a Deferred input to await on:
/// ```
/// use homestar_core::{
///     workflow::{
///         pointer::{Await, AwaitResult},
///         Ability, Input, Instruction, Nonce, Pointer,
///     },
///     Unit,
/// };
/// use libipld::{cid::{multihash::{Code, MultihashDigest}, Cid}, Ipld, Link};
/// use url::Url;

/// let wasm = "bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".to_string();
/// let resource = Url::parse(format!("ipfs://{wasm}").as_str()).expect("IPFS URL");
/// let h = Code::Blake3_256.digest(b"beep boop");
/// let cid = Cid::new_v1(0x55, h);
/// let link: Link<Cid> = Link::new(cid);
/// let awaited_instr = Pointer::new_from_link(link);
///
/// let instr = Instruction::new_with_nonce(
///     resource,
///     Ability::from("wasm/run"),
///     Input::<Unit>::Deferred(Await::new(awaited_instr, AwaitResult::Ok)),
///     Nonce::generate()
/// );
///
/// // And covert it to a pointer:
/// let ptr = Pointer::try_from(instr).unwrap();
/// ```
/// [deferred promise]: super::pointer::Await
#[derive(Clone, Debug, PartialEq)]
pub struct Instruction<'a, T> {
    rsc: Url,
    op: Cow<'a, Ability>,
    input: Input<T>,
    nnc: Nonce,
}

impl<T> Instruction<'_, T> {
    /// Create a new [Instruction] with an empty Nonce.
    pub fn new(rsc: Url, ability: Ability, input: Input<T>) -> Self {
        Self {
            rsc,
            op: Cow::from(ability),
            input,
            nnc: Nonce::Empty,
        }
    }

    /// Create a new [Instruction] with a given [Nonce].
    pub fn new_with_nonce(rsc: Url, ability: Ability, input: Input<T>, nnc: Nonce) -> Self {
        Self {
            rsc,
            op: Cow::from(ability),
            input,
            nnc,
        }
    }

    /// Create a unique [Instruction], with a default [Nonce] generator.
    pub fn unique(rsc: Url, ability: Ability, input: Input<T>) -> Self {
        Self {
            rsc,
            op: Cow::from(ability),
            input,
            nnc: Nonce::generate(),
        }
    }

    /// Return [Instruction] resource, i.e. [Url].
    pub fn resource(&self) -> &Url {
        &self.rsc
    }

    /// Return [Ability] associated with `op`.
    pub fn op(&self) -> &Ability {
        &self.op
    }

    /// Return [Instruction] [Input].
    pub fn input(&self) -> &Input<T> {
        &self.input
    }

    /// Return [Nonce] reference.
    pub fn nonce(&self) -> &Nonce {
        &self.nnc
    }
}

impl<T> TryFrom<Instruction<'_, T>> for Pointer
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(instruction: Instruction<'_, T>) -> Result<Self, Self::Error> {
        Ok(Pointer::new(Cid::try_from(instruction)?))
    }
}

impl<T> TryFrom<Instruction<'_, T>> for Cid
where
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(instruction: Instruction<'_, T>) -> Result<Self, Self::Error> {
        let ipld: Ipld = instruction.into();
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(DAG_CBOR, hash))
    }
}

impl<T> From<Instruction<'_, T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(instruction: Instruction<'_, T>) -> Self {
        Ipld::Map(BTreeMap::from([
            (RESOURCE_KEY.into(), instruction.rsc.to_string().into()),
            (OP_KEY.into(), instruction.op.to_string().into()),
            (INPUT_KEY.into(), instruction.input.into()),
            (NNC_KEY.into(), instruction.nnc.into()),
        ]))
    }
}

impl<T> TryFrom<&Ipld> for Instruction<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl<T> TryFrom<Ipld> for Instruction<'_, T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let rsc = match map.get(RESOURCE_KEY) {
            Some(Ipld::Link(cid)) => cid
                .to_string_of_base(Base::Base32Lower)
                .map_err(|e| anyhow!("failed to encode CID into multibase string: {e}"))
                .and_then(|txt| {
                    Url::parse(format!("{}{}", "ipfs://", txt).as_str())
                        .map_err(|e| anyhow!("failed to parse URL: {e}"))
                }),
            Some(Ipld::String(txt)) => {
                Url::parse(txt.as_str()).map_err(|e| anyhow!("failed to parse URL: {e}"))
            }
            _ => Err(anyhow!("no resource/with set.")),
        }?;

        Ok(Self {
            rsc,
            op: from_ipld(
                map.get(OP_KEY)
                    .ok_or_else(|| anyhow!("no `op` field set"))?
                    .to_owned(),
            )?,
            input: Input::try_from(
                map.get(INPUT_KEY)
                    .ok_or_else(|| anyhow!("no `input` field set"))?
                    .to_owned(),
            )?,
            nnc: Nonce::try_from(
                map.get(NNC_KEY)
                    .unwrap_or(&Ipld::String("".to_string()))
                    .to_owned(),
            )?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils, Unit};

    #[test]
    fn ipld_roundtrip() {
        let (instruction, bytes) = test_utils::workflow::instruction_with_nonce::<Unit>();
        let ipld = Ipld::from(instruction.clone());

        assert_eq!(
            ipld,
            Ipld::Map(BTreeMap::from([
                (
                    RESOURCE_KEY.into(),
                    Ipld::String(
                        "ipfs://bafkreidztuwoszw2dfnzufjpsjmzj67x574qcdm2autnhnv43o3t4zmh7i".into()
                    )
                ),
                (OP_KEY.into(), Ipld::String("ipld/fun".to_string())),
                (INPUT_KEY.into(), Ipld::List(vec![Ipld::Bool(true)])),
                (
                    NNC_KEY.into(),
                    Ipld::List(vec![Ipld::Integer(0), Ipld::Bytes(bytes)])
                )
            ]))
        );
        assert_eq!(instruction, ipld.try_into().unwrap())
    }
}
