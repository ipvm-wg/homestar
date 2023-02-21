use ipvm_wasm::wasmtime;
use libipld::Ipld;
use std::{fs, path::PathBuf};

fn fixtures(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("fixtures/{file}"))
}

#[tokio::test]
async fn test_execute_wat() {
    let ipld = Ipld::List(vec![Ipld::Integer(1)]);

    let wat = fs::read(fixtures("add_one_component.wat")).unwrap();
    let mut env =
        wasmtime::World::instantiate(wat, "add-one".to_string(), wasmtime::State::default())
            .await
            .unwrap();
    let res = env.execute(ipld).await.unwrap();
    assert_eq!(res, Ipld::List(vec![Ipld::Integer(2)]));
}

#[tokio::test]
async fn test_execute_wat_from_non_component() {
    let wat = fs::read(fixtures("add_one.wat")).unwrap();
    let env =
        wasmtime::World::instantiate(wat, "add_one".to_string(), wasmtime::State::default()).await;
    assert!(env.is_err());
}

#[tokio::test]
async fn test_execute_wasm_underscore() {
    let ipld = Ipld::List(vec![Ipld::Integer(1)]);

    let wasm = fs::read(fixtures("add_one.wasm")).unwrap();
    let mut env =
        wasmtime::World::instantiate(wasm, "add_one".to_string(), wasmtime::State::default())
            .await
            .unwrap();
    let res = env.execute(ipld).await.unwrap();
    assert_eq!(res, Ipld::List(vec![Ipld::Integer(2)]));
}

#[tokio::test]
async fn test_execute_wasm_hyphen() {
    let ipld = Ipld::List(vec![Ipld::Integer(10)]);

    let wasm = fs::read(fixtures("add_one.wasm")).unwrap();
    let mut env =
        wasmtime::World::instantiate(wasm, "add-one".to_string(), wasmtime::State::default())
            .await
            .unwrap();
    let res = env.execute(ipld).await.unwrap();
    assert_eq!(res, Ipld::List(vec![Ipld::Integer(11)]));
}

#[tokio::test]
async fn test_wasm_wrong_fun() {
    let wasm = fs::read(fixtures("add_one.wasm")).unwrap();
    let env =
        wasmtime::World::instantiate(wasm, "add-onez".to_string(), wasmtime::State::default())
            .await;
    assert!(env.is_err());
}

#[tokio::test]
async fn test_append_string() {
    let ipld = Ipld::List(vec![Ipld::String("Natural Science".to_string())]);

    let wasm = fs::read(fixtures("ipvm_guest_wasm.wasm")).unwrap();
    let mut env = wasmtime::World::instantiate(
        wasm,
        "append-string".to_string(),
        wasmtime::State::default(),
    )
    .await
    .unwrap();
    let res = env.execute(ipld).await.unwrap();
    assert_eq!(
        res,
        Ipld::List(vec![Ipld::String("Natural Science\nworld".to_string())])
    );
}

#[tokio::test]
async fn test_execute_wasms_in_seq() {
    let ipld_int = Ipld::List(vec![Ipld::Integer(1)]);
    let ipld_str = Ipld::List(vec![Ipld::String("Natural Science".to_string())]);

    let wasm1 = fs::read(fixtures("add_one.wasm")).unwrap();
    let wasm2 = fs::read(fixtures("ipvm_guest_wasm.wasm")).unwrap();

    let mut env =
        wasmtime::World::instantiate(wasm1, "add_one".to_string(), wasmtime::State::default())
            .await
            .unwrap();
    let res = env.execute(ipld_int).await.unwrap();
    assert_eq!(res, Ipld::List(vec![Ipld::Integer(2)]));

    let env2 =
        wasmtime::World::instantiate_with_current_env(wasm2, "append_string".to_string(), &mut env)
            .await
            .unwrap();

    let res2 = env2.execute(ipld_str).await.unwrap();

    assert_eq!(
        res2,
        Ipld::List(vec![Ipld::String("Natural Science\nworld".to_string())])
    );
}
