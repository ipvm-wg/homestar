use homestar_core::workflow::{
    input::{Args, Parse},
    pointer::{Await, AwaitResult},
    Input, InstructionResult, Pointer,
};
use homestar_wasm::{
    io::{Arg, Output},
    wasmtime::{State, World},
};
use libipld::{
    cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    },
    Ipld, Link,
};
use std::{collections::BTreeMap, fs, path::PathBuf};

fn fixtures(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("fixtures/{file}"))
}

#[tokio::test]
async fn test_execute_wat() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));
    // TODO: Replace this with updated versions and guest_wasm code.
    let wat = fs::read(fixtures("add_one_component.wat")).unwrap();
    let mut env = World::instantiate(wat, "add-one", State::default())
        .await
        .unwrap();
    let res = env
        .execute(ipld.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(2)));
}

#[tokio::test]
async fn test_execute_wat_from_non_component() {
    let wat = fs::read(fixtures("add_one.wat")).unwrap();
    let env = World::instantiate(wat, "add_one", State::default()).await;
    assert!(env.is_err());
}

#[tokio::test]
async fn test_execute_wasm_underscore() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));

    let wasm = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "add_one", State::default())
        .await
        .unwrap();
    let res = env
        .execute(ipld.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(2)));
}

#[tokio::test]
async fn test_execute_wasm_hyphen() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(10)])),
    ])));

    let wasm = fs::read(fixtures("homestar-guest-wasm.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "add-one", State::default())
        .await
        .unwrap();
    let res = env
        .execute(ipld.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(11)));
}

#[tokio::test]
async fn test_wasm_wrong_fun() {
    let wasm = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();
    let env = World::instantiate(wasm, "add-onez", State::default()).await;
    assert!(env.is_err());
}

#[tokio::test]
async fn test_append_string() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("append-string".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::String("Natural Science".to_string())]),
        ),
    ])));

    let wasm = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "append-string", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String(
            "Natural Science\nworld".into()
        ))
    );
}

#[tokio::test]
async fn test_matrix_transpose() {
    let ipld_inner = Ipld::List(vec![
        Ipld::List(vec![Ipld::Integer(1), Ipld::Integer(2), Ipld::Integer(3)]),
        Ipld::List(vec![Ipld::Integer(4), Ipld::Integer(5), Ipld::Integer(6)]),
        Ipld::List(vec![Ipld::Integer(7), Ipld::Integer(8), Ipld::Integer(9)]),
    ]);
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("transpose".to_string())),
        ("args".into(), Ipld::List(vec![ipld_inner.clone()])),
    ])));

    let wasm = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "transpose", State::default())
        .await
        .unwrap();

    let transposed = env
        .execute(ipld.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    let transposed_ipld = Ipld::try_from(transposed).unwrap();

    assert_ne!(transposed_ipld, ipld_inner);

    let ipld_transposed_map = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("transpose".to_string())),
        ("args".into(), Ipld::List(vec![transposed_ipld])),
    ])));

    let retransposed = env
        .execute(ipld_transposed_map.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    let retransposed_ipld = Ipld::try_from(retransposed).unwrap();

    assert_eq!(retransposed_ipld, ipld_inner);
}

#[tokio::test]
async fn test_execute_wasms_in_seq() {
    let ipld_int = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));

    let ipld_str = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("append_string".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::String("Natural Science".to_string())]),
        ),
    ])));

    let wasm = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();

    let mut env = World::instantiate(wasm.clone(), "add_one", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld_int.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(2)));

    let env2 = World::instantiate_with_current_env(wasm, "append_string", &mut env)
        .await
        .unwrap();

    let res = env2
        .execute(ipld_str.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String(
            "Natural Science\nworld".into()
        ))
    );
}

#[tokio::test]
async fn test_multiple_args() {
    let wasm = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();

    let ipld_str = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("join-strings".to_string())),
        (
            "args".into(),
            Ipld::List(vec![
                Ipld::String("Round".to_string()),
                Ipld::String("about".to_string()),
            ]),
        ),
    ])));

    let mut env = World::instantiate(wasm, "join-strings", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld_str.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String("Roundabout".into()))
    );
}

#[tokio::test]
async fn test_execute_wasms_in_seq_with_threaded_result() {
    let ipld_step_1 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("join-strings".to_string())),
        (
            "args".into(),
            Ipld::List(vec![
                Ipld::String("Round".to_string()),
                Ipld::String("about".to_string()),
            ]),
        ),
    ])));

    let h = Code::Blake3_256.digest(b"beep boop");
    let cid = Cid::new_v1(0x55, h);
    let link: Link<Cid> = Link::new(cid);
    let invoked_instr = Pointer::new_from_link(link);

    let promise = Await::new(invoked_instr, AwaitResult::Ok);

    let ipld_step_2 = Input::<Arg>::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("join-strings".to_string())),
        (
            "args".into(),
            Ipld::List(vec![
                Ipld::try_from(promise).unwrap(),
                Ipld::String("about".to_string()),
            ]),
        ),
    ])));

    let wasm1 = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();

    let mut env = World::instantiate(wasm1.clone(), "join-strings", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld_step_1.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String("Roundabout".into()))
    );

    let env2 = World::instantiate_with_current_env(wasm1, "join-strings", &mut env)
        .await
        .unwrap();

    let parsed: Args<Arg> = ipld_step_2.parse().unwrap().try_into().unwrap();

    // Short-circuit resolve with known value.
    let resolved = parsed
        .resolve(|_| {
            Ok(InstructionResult::Ok(Arg::Value(
                wasmtime::component::Val::String("RoundRound".into()),
            )))
        })
        .unwrap();

    let res2 = env2.execute(resolved).await.unwrap();
    assert_eq!(
        res2,
        Output::Value(wasmtime::component::Val::String("RoundRoundabout".into()))
    );
}

#[tokio::test]
async fn test_execute_wasms_with_multiple_inits() {
    let ipld_step_1 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("join-strings".to_string())),
        (
            "args".into(),
            Ipld::List(vec![
                Ipld::String("Round".to_string()),
                Ipld::String("about".to_string()),
            ]),
        ),
    ])));

    let h = Code::Blake3_256.digest(b"beep boop");
    let cid = Cid::new_v1(0x55, h);
    let link: Link<Cid> = Link::new(cid);
    let invoked_instr = Pointer::new_from_link(link);

    let promise = Await::new(invoked_instr, AwaitResult::Ok);

    let ipld_step_2 = Input::<Arg>::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("join-strings".to_string())),
        (
            "args".into(),
            Ipld::List(vec![
                Ipld::try_from(promise).unwrap(),
                Ipld::String("about".to_string()),
            ]),
        ),
    ])));

    let wasm1 = fs::read(fixtures("homestar_guest_wasm.wasm")).unwrap();

    let mut env = World::instantiate(wasm1.clone(), "join-strings", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld_step_1.parse().unwrap().try_into().unwrap())
        .await
        .unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String("Roundabout".into()))
    );

    let mut env2 = World::instantiate(wasm1, "join-strings", State::default())
        .await
        .unwrap();

    let parsed: Args<Arg> = ipld_step_2.parse().unwrap().try_into().unwrap();

    // Short-circuit resolve with known value.
    let resolved = parsed
        .resolve(|_| {
            Ok(InstructionResult::Ok(Arg::Ipld(Ipld::String(
                "RoundRound".into(),
            ))))
        })
        .unwrap();

    let res2 = env2.execute(resolved).await.unwrap();
    assert_eq!(
        res2,
        Output::Value(wasmtime::component::Val::String("RoundRoundabout".into()))
    );
}
