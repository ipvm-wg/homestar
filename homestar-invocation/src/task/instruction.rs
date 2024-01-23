//! An [Instruction] is the smallest unit of work that can be requested from a
//! UCAN, described via `resource`, `ability`.

use crate::{
    ipld::{self, DagCbor},
    pointer::AwaitResult,
    Error, Pointer, Unit,
};
use libipld::{cid::multibase::Base, serde::from_ipld, Ipld};
use schemars::{
    gen::SchemaGenerator,
    schema::{
        ArrayValidation, InstanceType, Metadata, ObjectValidation, Schema, SchemaObject,
        SingleOrVec,
    },
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt,
};
use url::Url;

const RESOURCE_KEY: &str = "rsc";
const OP_KEY: &str = "op";
const INPUT_KEY: &str = "input";
const NNC_KEY: &str = "nnc";

mod ability;
pub mod input;
mod nonce;
pub use ability::*;
pub use input::{Args, Input, Parse, Parsed};
pub use nonce::*;

/// Enumerator for `either` an expanded [Instruction] structure or
/// an [Pointer] (Cid wrapper).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    type Error = Error<RunInstruction<'a, T>>;

    fn try_from(run: RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Expanded(instruction) => Ok(instruction),
            e => Err(Error::InvalidDiscriminant(e)),
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
    type Error = Error<RunInstruction<'a, T>>;

    fn try_from(run: RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Ptr(ptr) => Ok(ptr),
            e => Err(Error::InvalidDiscriminant(e)),
        }
    }
}

impl<'a, 'b, T> TryFrom<&'b RunInstruction<'a, T>> for &'b Pointer
where
    T: fmt::Debug,
{
    type Error = Error<&'b RunInstruction<'a, T>>;

    fn try_from(run: &'b RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Ptr(ptr) => Ok(ptr),
            e => Err(Error::InvalidDiscriminant(e)),
        }
    }
}

impl<'a, 'b, T> TryFrom<&'b RunInstruction<'a, T>> for Pointer
where
    T: fmt::Debug,
{
    type Error = Error<&'b RunInstruction<'a, T>>;

    fn try_from(run: &'b RunInstruction<'a, T>) -> Result<Self, Self::Error> {
        match run {
            RunInstruction::Ptr(ptr) => Ok(ptr.to_owned()),
            e => Err(Error::InvalidDiscriminant(e)),
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
    type Error = Error<Unit>;

    fn try_from<'a>(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(_) => Ok(RunInstruction::Expanded(Instruction::try_from(ipld)?)),
            Ipld::Link(_) => Ok(RunInstruction::Ptr(Pointer::try_from(ipld)?)),
            other_ipld => Err(Error::unexpected_ipld(other_ipld)),
        }
    }
}

/// Instruction to be executed.
///
/// # Example
///
/// ```
/// use homestar_invocation::{
///     task::{
///         instruction::{Ability, Input},
///         Instruction,
///     },
///     Unit,
///  };
/// use libipld::Ipld;
/// use url::Url;
///
/// let wasm = "bafybeidbyqpmztqkeot33lz4ev2ftjhqrnbh67go56tlgbf7qmy5xyzvg4".to_string();
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
/// use homestar_invocation::{
///     pointer::{Await, AwaitResult},
///     task::{
///         instruction::{Ability, Input, Nonce},
///         Instruction,
///     },
///     Pointer, Unit,
///  };
/// use libipld::{cid::{multihash::{Code, MultihashDigest}, Cid}, Ipld, Link};
/// use url::Url;

/// let wasm = "bafybeidbyqpmztqkeot33lz4ev2ftjhqrnbh67go56tlgbf7qmy5xyzvg4".to_string();
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    type Error = Error<Unit>;

    fn try_from(instruction: Instruction<'_, T>) -> Result<Self, Self::Error> {
        Ok(Pointer::new(instruction.to_cid()?))
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
    type Error = Error<Unit>;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl<T> TryFrom<Ipld> for Instruction<'_, T>
where
    T: From<Ipld>,
{
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let rsc = match map.get(RESOURCE_KEY) {
            Some(Ipld::Link(cid)) => cid
                .to_string_of_base(Base::Base32Lower) // Cid v1
                .map_err(Error::<Unit>::CidEncode)
                .and_then(|txt| {
                    Url::parse(format!("{}{}", "ipfs://", txt).as_str())
                        .map_err(Error::ParseResource)
                }),
            Some(Ipld::String(txt)) => Url::parse(txt.as_str()).map_err(Error::ParseResource),
            _ => Err(Error::MissingField(RESOURCE_KEY.to_string())),
        }?;

        Ok(Self {
            rsc,
            op: from_ipld(
                map.get(OP_KEY)
                    .ok_or_else(|| Error::<Unit>::MissingField(OP_KEY.to_string()))?
                    .to_owned(),
            )?,
            input: Input::try_from(
                map.get(INPUT_KEY)
                    .ok_or_else(|| Error::<String>::MissingField(INPUT_KEY.to_string()))?
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

impl<'a, T> DagCbor for Instruction<'a, T> where Ipld: From<T> {}

impl<'a, T> JsonSchema for Instruction<'a, T> {
    fn schema_name() -> String {
        "run".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-invocation::task::Instruction")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        struct InputConditional {
            if_schema: Schema,
            then_schema: Schema,
            else_schema: Schema,
        }

        fn input_conditional(gen: &mut SchemaGenerator) -> InputConditional {
            let if_schema = SchemaObject {
                instance_type: None,
                object: Some(Box::new(ObjectValidation {
                    properties: BTreeMap::from([(
                        "op".to_owned(),
                        Schema::Object(SchemaObject {
                            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
                            const_value: Some(json!("wasm/run")),
                            ..Default::default()
                        }),
                    )]),
                    ..Default::default()
                })),
                ..Default::default()
            };

            let func_schema = SchemaObject {
                instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
                metadata: Some(Box::new(Metadata {
                    description: Some("The function to call on the Wasm resource".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            };

            let args_schema = SchemaObject {
                instance_type: Some(SingleOrVec::Single(InstanceType::Array.into())),
                metadata: Some(Box::new(Metadata {
                    description: Some(
                        "Arguments to the function. May await a result from another task."
                            .to_string(),
                    ),
                    ..Default::default()
                })),
                array: Some(Box::new(ArrayValidation {
                    items: Some(SingleOrVec::Vec(vec![
                        gen.subschema_for::<ipld::schema::IpldStub>(),
                        gen.subschema_for::<AwaitResult>(),
                    ])),
                    ..Default::default()
                })),
                ..Default::default()
            };

            let input_schema = SchemaObject {
                instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
                object: Some(Box::new(ObjectValidation {
                    properties: BTreeMap::from([
                        ("func".to_string(), Schema::Object(func_schema)),
                        ("args".to_string(), Schema::Object(args_schema)),
                    ]),
                    required: BTreeSet::from(["func".to_string(), "args".to_string()]),
                    ..Default::default()
                })),
                ..Default::default()
            };

            let then_schema = SchemaObject {
                instance_type: None,
                object: Some(Box::new(ObjectValidation {
                    properties: BTreeMap::from([(
                        "input".to_string(),
                        Schema::Object(input_schema),
                    )]),
                    ..Default::default()
                })),
                ..Default::default()
            };

            InputConditional {
                if_schema: Schema::Object(if_schema),
                then_schema: Schema::Object(then_schema),
                else_schema: Schema::Bool(false),
            }
        }

        let op_schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
            metadata: Some(Box::new(Metadata {
                description: Some("Function executor".to_string()),
                ..Default::default()
            })),
            enum_values: Some(vec![json!("wasm/run")]),
            ..Default::default()
        };

        let mut schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
            metadata: Some(Box::new(Metadata {
                title: Some("Run instruction".to_string()),
                description: Some("An instruction that runs a function from a resource, executor that will run the function, inputs to the executor, and optional nonce".to_string()),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                properties: BTreeMap::from([
                    ("rsc".to_owned(), <Url>::json_schema(gen)),
                    ("op".to_owned(), Schema::Object(op_schema)),
                    ("nnc".to_owned(), <Nonce>::json_schema(gen))
                ]),
                required: BTreeSet::from(["rsc".to_string(), "op".to_string(), "input".to_string(), "nnc".to_string()]),
                ..Default::default()
            })),
            ..Default::default()
        };

        let input = input_conditional(gen);
        schema.subschemas().if_schema = Some(Box::new(input.if_schema));
        schema.subschemas().then_schema = Some(Box::new(input.then_schema));
        schema.subschemas().else_schema = Some(Box::new(input.else_schema));

        schema.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils, Unit, DAG_CBOR};
    use libipld::{
        cbor::DagCborCodec,
        multihash::{Code, MultihashDigest},
        prelude::Codec,
        Cid,
    };

    #[test]
    fn ipld_roundtrip() {
        let (instruction, bytes) = test_utils::instruction_with_nonce::<Unit>();
        let ipld = Ipld::from(instruction.clone());

        assert_eq!(
            ipld,
            Ipld::Map(BTreeMap::from([
                (
                    RESOURCE_KEY.into(),
                    Ipld::String(
                        "ipfs://bafybeidbyqpmztqkeot33lz4ev2ftjhqrnbh67go56tlgbf7qmy5xyzvg4".into()
                    )
                ),
                (OP_KEY.into(), Ipld::String("ipld/fun".to_string())),
                (INPUT_KEY.into(), Ipld::List(vec![Ipld::Bool(true)])),
                (NNC_KEY.into(), Ipld::Bytes(bytes))
            ]))
        );
        assert_eq!(instruction, ipld.try_into().unwrap())
    }

    #[test]
    fn ipld_cid_trials() {
        let a_cid =
            Cid::try_from("bafyrmiev5j2jzjrqncbfqo6pbraiw7r2p527m4z3bbm6ir3o5kdz2zwcjy").unwrap();
        let ipld = libipld::ipld!({"input":
                        {
                            "args": [{"await/ok": a_cid}, "111111"],
                            "func": "join-strings"
                        },
                        "nnc": "", "op": "wasm/run",
                        "rsc": "ipfs://bafybeidbyqpmztqkeot33lz4ev2ftjhqrnbh67go56tlgbf7qmy5xyzvg4"});

        let instruction = Instruction::<Unit>::try_from(ipld.clone()).unwrap();
        let instr_cid = instruction.to_cid().unwrap();

        let bytes = DagCborCodec.encode(&ipld).unwrap();
        let hash = Code::Sha3_256.digest(&bytes);
        let ipld_to_cid = Cid::new_v1(DAG_CBOR, hash);

        assert_eq!(ipld_to_cid, instr_cid);
    }

    #[test]
    fn ser_de() {
        let (instruction, _bytes) = test_utils::instruction_with_nonce::<Unit>();
        let ser = serde_json::to_string(&instruction).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(instruction, de);
    }
}
