use homestar_invocation::{
    pointer::{Await, AwaitResult},
    task::{
        self,
        instruction::{Args, Input, Parse},
    },
    Pointer,
};
use homestar_wasm::{
    io::{Arg, Output},
    wasmtime::{limits::StoreLimitsAsync, Error, State, World},
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
async fn test_wasm_exceeds_max_memory() {
    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let env = World::instantiate(
        wasm,
        "add_one",
        State::new(u64::MAX, StoreLimitsAsync::new(Some(10), None)),
    )
    .await;

    if let Err(Error::WasmRuntime(err)) = env {
        assert!(err.to_string().contains("exceeds memory limits"));
    } else {
        panic!("Expected WasmRuntimeError")
    }
}

#[tokio::test]
async fn test_execute_wat() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_two".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));
    // TODO: Replace this with updated versions and guest_wasm code.
    let wat = fs::read(fixtures("example_add_component.wat")).unwrap();
    let mut env = World::instantiate(wat, "add_two", State::default())
        .await
        .unwrap();
    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(3)));
}

#[tokio::test]
async fn test_execute_wat_cargo_component() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_two".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));
    let wat = fs::read(fixtures("example_add_cargo_component_wasi.wat")).unwrap();
    let mut env = World::instantiate(wat, "add_two", State::default())
        .await
        .unwrap();
    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(3)));
}

#[tokio::test]
async fn test_execute_wat_from_non_component() {
    let wat = fs::read(fixtures("example_add.wat")).unwrap();
    let env = World::instantiate(wat, "add_two", State::default()).await;
    assert!(env.is_err());
}

#[tokio::test]
async fn test_execute_wasm_cargo_component() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));

    let wasm = fs::read(fixtures("example_test_cargo_component.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "add_one", State::default())
        .await
        .unwrap();
    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(2)));
}

#[tokio::test]
async fn test_execute_wasm_cargo_component_wasi() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));

    let wasm = fs::read(fixtures("example_test_cargo_component_wasi.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "add_one", State::default())
        .await
        .unwrap();
    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(2)));
}

#[tokio::test]
async fn test_execute_wasm_underscore() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(1)])),
    ])));

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "add_one", State::default())
        .await
        .unwrap();
    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(2)));
}

#[tokio::test]
async fn test_execute_wasm_hyphen() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("add_one".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::Integer(10)])),
    ])));

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "add-one", State::default())
        .await
        .unwrap();
    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(11)));
}

#[tokio::test]
async fn test_wasm_wrong_fun() {
    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
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

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "append-string", State::default())
        .await
        .unwrap();

    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String(
            "Natural Science\nworld".into()
        ))
    );
}

#[tokio::test]
async fn test_crop_base64_wasi() {
    let img_uri = r#"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABQAAAAUCAYAAACNiR0NAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAG9SURBVHgBrVRLTgJBEK3qaYhLtiZGxxMoN8CdISTiCYSEmLgSTkA8AXFFAgv0BkNMDDu4gXAC2vhhy9IIU2X1CGb4T5CXzCRd3f26Xv0QBOna+ykiVJjBha2A3XhMlbz8vsFsdeCOtP/CAAn4H4bAcKbGMarsgMwiYVUqIj6FHYERXBU2oHV7BYI95HsH6VKk5eUzkw0TPqfDCwK400iGWDXmw+BrJ9mSoE/X59VBZ2/vazjy4xIyzk3tat6Tp8Kh54+d5J8HgRZuhsksWjf7xssfD5npNaxsXvLV9PDz9cGxlSaB7sopA0uQbfQlEeoorAalBvvC5E4IO1KLj0L2ABGQqb+lCLAd8sgsSI5KFtxHXii3GUJxPZWuf5QhIgici7WEwavAKSsFNsB2mCQru5HQFqfW2sAGSLveLuuwBULR7X77fluSlYMVyNQ+LVlx2Z6ec8+TXzOunY5XmK07C1smo3GsTEDFFW/Nls2vBYwtH/G0R9I1gYlUAh04kSzk1g4SuasXjCJZLuWCfVbTg8AEkaAQl3fBViDuKemM0ropExWWg2K6iHYhk8NVMmhF2FazUUiMhKQkXdb9AfsesrssluqmAAAAAElFTkSuQmCC"#;
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("crop-base64".to_string())),
        (
            "args".into(),
            Ipld::List(vec![
                Ipld::String(img_uri.to_string()),
                Ipld::Integer(10),
                Ipld::Integer(10),
                Ipld::Integer(50),
                Ipld::Integer(50),
            ]),
        ),
    ])));

    let wasm = fs::read(fixtures("example_test_wasi_component.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "crop-base64", State::default())
        .await
        .unwrap();

    let res = env.execute(ipld.parse().unwrap().into()).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_host_funs_wasi() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        (
            "func".into(),
            Ipld::String("host_fmt_current_time".to_string()),
        ),
        ("args".into(), Ipld::List(vec![])),
    ])));

    let wasm = fs::read(fixtures("example_test_wasi_component.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "host_fmt_current_time", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld.parse().unwrap().into())
        .await
        .unwrap()
        .take()
        .unwrap();
    assert!(matches!(res, wasmtime::component::Val::String(_)));
}

#[tokio::test]
async fn test_option_return_with_pop() {
    let ipld1 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("pop".to_string())),
        ("args".into(), Ipld::List(vec![Ipld::List(vec![])])),
    ])));

    let ipld2 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("pop".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::List(vec![Ipld::Integer(1)])]),
        ),
    ])));

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "pop", State::default())
        .await
        .unwrap();

    let res = env.execute(ipld1.parse().unwrap().into()).await.unwrap();
    let option_val_none = match res {
        Output::Value(wasmtime::component::Val::Option(val)) => val,
        _ => panic!("Expected Wit Option"),
    };
    assert_eq!(option_val_none.value(), None);

    let res = env.execute(ipld2.parse().unwrap().into()).await.unwrap();
    let option_val_some = match res {
        Output::Value(wasmtime::component::Val::Option(val)) => val,
        _ => panic!("Expected Wit Option"),
    };
    assert_eq!(
        option_val_some.value(),
        Some(wasmtime::component::Val::S32(1)).as_ref()
    );
}

#[tokio::test]
async fn test_result_return_with_binary_search() {
    let ipld1 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("binary_search".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::List(vec![Ipld::Integer(1)]), Ipld::Integer(2)]),
        ),
    ])));

    let ipld2 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("binary_search".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::List(vec![Ipld::Integer(1)]), Ipld::Integer(1)]),
        ),
    ])));

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "binary_search", State::default())
        .await
        .unwrap();

    let res = env.execute(ipld1.parse().unwrap().into()).await.unwrap();
    let result_val_err = match res {
        Output::Value(wasmtime::component::Val::Result(val)) => val,
        _ => panic!("Expected Wit Result"),
    };
    assert_eq!(result_val_err.value(), Err(None));

    let res = env.execute(ipld2.parse().unwrap().into()).await.unwrap();
    let result_val_ok = match res {
        Output::Value(wasmtime::component::Val::Result(val)) => val,
        _ => panic!("Expected Wit Result"),
    };
    assert_eq!(
        result_val_ok.value(),
        Ok(Some(wasmtime::component::Val::S32(0)).as_ref())
    );
}

#[tokio::test]
async fn test_result_input_record_return_with_num_keys() {
    let ipld1 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("num_to_kv".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::List(vec![Ipld::Integer(1), Ipld::Null])]),
        ),
    ])));
    let ipld2 = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("num_to_kv".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::List(vec![Ipld::Null, "bad stuff".into()])]),
        ),
    ])));

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "num_to_kv", State::default())
        .await
        .unwrap();

    let res = env.execute(ipld1.parse().unwrap().into()).await.unwrap();
    let result_val_on_ok = match res {
        Output::Value(wasmtime::component::Val::Record(val)) => val,
        _ => panic!("Expected Wit Record"),
    };
    let mut fields = result_val_on_ok.fields();
    let (k1, v1) = fields.next().unwrap();
    let (k2, v2) = fields.next().unwrap();
    assert_eq!(k1, "name");
    assert_eq!(
        v1,
        &wasmtime::component::Val::String("1".to_string().into())
    );

    assert_eq!(k2, "val");
    if let wasmtime::component::Val::Option(val) = v2 {
        assert_eq!(val.value(), Some(wasmtime::component::Val::U32(1)).as_ref());
    } else {
        panic!("Expected Wit Option");
    }

    let res = env.execute(ipld2.parse().unwrap().into()).await.unwrap();
    let result_val_on_err = match res {
        Output::Value(wasmtime::component::Val::Record(val)) => val,
        _ => panic!("Expected Wit Record"),
    };
    let mut fields = result_val_on_err.fields();
    let (k1, v1) = fields.next().unwrap();
    let (k2, v2) = fields.next().unwrap();
    assert_eq!(k1, "name");
    assert_eq!(
        v1,
        &wasmtime::component::Val::String("NAN".to_string().into())
    );

    assert_eq!(k2, "val");
    if let wasmtime::component::Val::Option(val) = v2 {
        assert_eq!(val.value(), None);
    } else {
        panic!("Expected Wit Option");
    }
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

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "transpose", State::default())
        .await
        .unwrap();

    let transposed = env.execute(ipld.parse().unwrap().into()).await.unwrap();

    let transposed_ipld = Ipld::try_from(transposed).unwrap();

    assert_ne!(transposed_ipld, ipld_inner);

    let ipld_transposed_map = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("transpose".to_string())),
        ("args".into(), Ipld::List(vec![transposed_ipld])),
    ])));

    let retransposed = env
        .execute(ipld_transposed_map.parse().unwrap().into())
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

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();

    let mut env = World::instantiate(wasm.clone(), "add_one", State::default())
        .await
        .unwrap();

    let res = env.execute(ipld_int.parse().unwrap().into()).await.unwrap();

    assert_eq!(res, Output::Value(wasmtime::component::Val::S32(2)));

    let env2 = World::instantiate_with_current_env(wasm, "append_string", &mut env)
        .await
        .unwrap();

    let res = env2
        .execute(ipld_str.parse().unwrap().into())
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
    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();

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

    let res = env.execute(ipld_str.parse().unwrap().into()).await.unwrap();

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
            Ipld::List(vec![Ipld::from(promise), Ipld::String("about".to_string())]),
        ),
    ])));

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();

    let mut env = World::instantiate(wasm.clone(), "join-strings", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld_step_1.parse().unwrap().into())
        .await
        .unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String("Roundabout".into()))
    );

    let env2 = World::instantiate_with_current_env(wasm, "join-strings", &mut env)
        .await
        .unwrap();

    let parsed: Args<Arg> = ipld_step_2.parse().unwrap().into();

    // Short-circuit resolve with known value.
    let resolved = parsed
        .resolve(|_| {
            Box::pin(async {
                Ok(task::Result::Ok(Arg::Value(
                    wasmtime::component::Val::String("RoundRound".into()),
                )))
            })
        })
        .await
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
            Ipld::List(vec![Ipld::from(promise), Ipld::String("about".to_string())]),
        ),
    ])));

    let wasm = fs::read(fixtures("example_test.wasm")).unwrap();

    let mut env = World::instantiate(wasm.clone(), "join-strings", State::default())
        .await
        .unwrap();

    let res = env
        .execute(ipld_step_1.parse().unwrap().into())
        .await
        .unwrap();

    assert_eq!(
        res,
        Output::Value(wasmtime::component::Val::String("Roundabout".into()))
    );

    let mut env2 = World::instantiate(wasm, "join-strings", State::default())
        .await
        .unwrap();

    let parsed: Args<Arg> = ipld_step_2.parse().unwrap().into();

    // Short-circuit resolve with known value.
    let resolved = parsed
        .resolve(|_| {
            Box::pin(async {
                Ok(task::Result::Ok(Arg::Value(
                    wasmtime::component::Val::String("RoundRound".into()),
                )))
            })
        })
        .await
        .unwrap();

    let res2 = env2.execute(resolved).await.unwrap();
    assert_eq!(
        res2,
        Output::Value(wasmtime::component::Val::String("RoundRoundabout".into()))
    );
}

#[tokio::test]
async fn test_subtract() {
    let ipld = Input::Ipld(Ipld::Map(BTreeMap::from([
        ("func".into(), Ipld::String("subtract".to_string())),
        (
            "args".into(),
            Ipld::List(vec![Ipld::Integer(1), Ipld::Integer(1)]),
        ),
    ])));

    let wasm = fs::read(fixtures("example_subtract.wasm")).unwrap();
    let mut env = World::instantiate(wasm, "subtract", State::default())
        .await
        .unwrap();
    let res = env.execute(ipld.parse().unwrap().into()).await.unwrap();
    assert_eq!(res, Output::Value(wasmtime::component::Val::Float64(0.0)));
}
