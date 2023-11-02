//! Test utilities for working with [workflow] items.
//!
//! [workflow]: crate::workflow

use crate::{
    ipld::DagCbor,
    workflow::{
        pointer::{Await, AwaitResult},
        prf::UcanPrf,
        Ability, Input, Instruction, InstructionResult, Nonce, Pointer, Receipt,
    },
};
use libipld::{
    cid::Cid,
    multihash::{Code, MultihashDigest},
    Ipld, Link,
};
use std::collections::BTreeMap;
use url::Url;

const RAW: u64 = 0x55;
const WASM_CID: &str = "bafkreihxcyjgyrz437ewzi7md55uqt2zf6yr3zn7xrfi4orc34xdc5jgrm";

type NonceBytes = Vec<u8>;

/// Return a `mocked` `wasm/run` [Instruction].
pub fn wasm_instruction<'a, T>() -> Instruction<'a, T> {
    let resource = Url::parse(format!("ipfs://{WASM_CID}").as_str()).unwrap();

    Instruction::new(
        resource,
        Ability::from("wasm/run"),
        Input::Ipld(Ipld::Map(BTreeMap::from([
            ("func".into(), Ipld::String("add_one".to_string())),
            ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
        ]))),
    )
}

/// Return `mocked` `wasm/run` [Instruction]'s, where the second is dependent on
/// the first.
pub fn related_wasm_instructions<'a, T>(
) -> (Instruction<'a, T>, Instruction<'a, T>, Instruction<'a, T>)
where
    Ipld: From<T>,
    T: Clone,
{
    let resource = Url::parse(format!("ipfs://{WASM_CID}").as_str()).unwrap();

    let instr = Instruction::new(
        resource.clone(),
        Ability::from("wasm/run"),
        Input::Ipld(Ipld::Map(BTreeMap::from([
            ("func".into(), Ipld::String("add_one".to_string())),
            ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
        ]))),
    );

    let promise = Await::new(
        Pointer::new(instr.clone().to_cid().unwrap()),
        AwaitResult::Ok,
    );

    let dep_instr1 = Instruction::new(
        resource.clone(),
        Ability::from("wasm/run"),
        Input::Ipld(Ipld::Map(BTreeMap::from([
            ("func".into(), Ipld::String("add_one".to_string())),
            (
                "args".into(),
                Ipld::List(vec![Ipld::try_from(promise.clone()).unwrap()]),
            ),
        ]))),
    );

    let another_promise = Await::new(
        Pointer::new(dep_instr1.clone().to_cid().unwrap()),
        AwaitResult::Ok,
    );

    let dep_instr2 = Instruction::new(
        resource,
        Ability::from("wasm/run"),
        Input::Ipld(Ipld::Map(BTreeMap::from([
            ("func".into(), Ipld::String("add_three".to_string())),
            (
                "args".into(),
                Ipld::List(vec![
                    Ipld::try_from(another_promise).unwrap(),
                    Ipld::try_from(promise).unwrap(),
                    Ipld::Integer(42),
                ]),
            ),
        ]))),
    );

    (instr, dep_instr1, dep_instr2)
}

/// Return a `mocked` `wasm/run` [Instruction], along with it's [Nonce] as bytes.
pub fn wasm_instruction_with_nonce<'a, T>() -> (Instruction<'a, T>, NonceBytes) {
    let resource = Url::parse(format!("ipfs://{WASM_CID}").as_str()).unwrap();
    let nonce = Nonce::generate();

    (
        Instruction::new_with_nonce(
            resource,
            Ability::from("wasm/run"),
            Input::Ipld(Ipld::Map(BTreeMap::from([
                ("func".into(), Ipld::String("add_one".to_string())),
                ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
            ]))),
            nonce.clone(),
        ),
        nonce.as_nonce96().unwrap().to_vec(),
    )
}

/// Return a `mocked` [Instruction].
pub fn instruction<'a, T>() -> Instruction<'a, T> {
    let resource = Url::parse(format!("ipfs://{WASM_CID}").as_str()).unwrap();

    Instruction::new(
        resource,
        Ability::from("ipld/fun"),
        Input::Ipld(Ipld::List(vec![Ipld::Bool(true)])),
    )
}

/// Return a `mocked` [Instruction], along with it's [Nonce] as bytes.
pub fn instruction_with_nonce<'a, T>() -> (Instruction<'a, T>, NonceBytes) {
    let resource = Url::parse(format!("ipfs://{WASM_CID}").as_str()).unwrap();
    let nonce = Nonce::generate();

    (
        Instruction::new_with_nonce(
            resource,
            Ability::from("ipld/fun"),
            Input::Ipld(Ipld::List(vec![Ipld::Bool(true)])),
            nonce.clone(),
        ),
        nonce.as_nonce96().unwrap().to_vec(),
    )
}

/// Return a `mocked` [Receipt] with an [Ipld] [InstructionResult].
pub fn receipt() -> Receipt<Ipld> {
    let h = Code::Blake3_256.digest(b"beep boop");
    let cid = Cid::new_v1(RAW, h);
    let link: Link<Cid> = Link::new(cid);
    Receipt::new(
        Pointer::new_from_link(link),
        InstructionResult::Ok(Ipld::Bool(true)),
        Ipld::Null,
        None,
        UcanPrf::default(),
    )
}
