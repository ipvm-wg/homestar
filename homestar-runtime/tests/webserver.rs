#[cfg(feature = "ipfs")]
use crate::utils::startup_ipfs;
use crate::utils::{
    kill_homestar, remove_db, stop_all_bins, wait_for_socket_connection, TimeoutFutureExt,
    BIN_NAME, IPFS,
};
use anyhow::Result;
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use once_cell::sync::Lazy;
use serial_test::file_serial;
use std::{
    fs,
    net::Ipv4Addr,
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "subscribe_run_workflow";
const UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "unsubscribe_run_workflow";

#[cfg(feature = "test-utils")]
#[test]
#[file_serial]
fn test_workflow_run_serial() -> Result<()> {
    const DB: &str = "ws_homestar_test_workflow_run.db";
    const IPFS_EXT: &str = "ws_homestar_test_workflow_run";

    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs(IPFS_EXT);

    let add_image_args = vec![
        "add",
        "--cid-version",
        "1",
        "../examples/websocket-relay/synthcat.png",
    ];

    let add_wasm_args = vec![
        "add",
        "--cid-version",
        "1",
        "../examples/websocket-relay/example_test.wasm",
    ];

    let _ = fs::remove_file(DB);

    let _ipfs_add_img = Command::new(IPFS)
        .args(add_image_args)
        .stdout(Stdio::piped())
        .output()
        .expect("`ipfs add` of synthcat.png");

    let _ipfs_add_wasm = Command::new(IPFS)
        .args(add_wasm_args)
        .stdout(Stdio::piped())
        .output()
        .expect("`ipfs add` of wasm mod");

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_workflow2.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let ws_port = 8061;
    if wait_for_socket_connection(ws_port, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port);

    tokio_test::block_on(async {
        let workflow_str =
            fs::read_to_string("tests/fixtures/test-workflow-image-pipeline.json").unwrap();
        let json: serde_json::Value = serde_json::from_str(&workflow_str).unwrap();
        let json_string = serde_json::to_string(&json).unwrap();
        let run_str = format!(r#"{{"name": "test","workflow": {}}}"#, json_string);
        let run: serde_json::Value = serde_json::from_str(&run_str).unwrap();

        let client1 = WsClientBuilder::default()
            .build(ws_url.clone())
            .await
            .unwrap();

        let mut sub1: Subscription<Vec<u8>> = client1
            .subscribe(
                SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![run.clone()],
                UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await
            .unwrap();

        // we have 3 operations
        let one = sub1
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&one.unwrap().unwrap()).unwrap();
        let check = json.get("metadata").unwrap();
        let expected = serde_json::json!({"name": "test", "replayed": false, "workflow": {"/": "bafyrmihfhdhxmhotbgn5digt6n7vgz2ukisafhjozki2e6nwtvunep3mrm"}});
        assert_eq!(check, &expected);

        let two = sub1
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&two.unwrap().unwrap()).unwrap();
        let check = json.get("metadata").unwrap();
        let expected = serde_json::json!({"name": "test", "replayed": false, "workflow": {"/": "bafyrmihfhdhxmhotbgn5digt6n7vgz2ukisafhjozki2e6nwtvunep3mrm"}});
        assert_eq!(check, &expected);

        let three = sub1
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&three.unwrap().unwrap()).unwrap();
        let check = json.get("metadata").unwrap();
        let expected = serde_json::json!({"name": "test", "replayed": false, "workflow": {"/": "bafyrmihfhdhxmhotbgn5digt6n7vgz2ukisafhjozki2e6nwtvunep3mrm"}});
        assert_eq!(check, &expected);

        // separate subscription, only 3 events too
        let mut sub2: Subscription<Vec<u8>> = client1
            .subscribe(
                SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![run.clone()],
                UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await
            .unwrap();

        let msg = sub2
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();
        let check = json.get("metadata").unwrap();
        let expected = serde_json::json!({"name": "test", "replayed": true, "workflow": {"/": "bafyrmihfhdhxmhotbgn5digt6n7vgz2ukisafhjozki2e6nwtvunep3mrm"}});
        assert_eq!(check, &expected);

        assert!(sub2
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());
        assert!(sub2
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());

        let client2 = WsClientBuilder::default().build(ws_url).await.unwrap();
        let mut sub3: Subscription<Vec<u8>> = client2
            .subscribe(
                SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![run],
                UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await
            .unwrap();

        let _ = sub2
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .is_err();

        assert!(sub3
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());
        assert!(sub2
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());
        assert!(sub2
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());

        let another_run_str = format!(r#"{{"name": "another_test","workflow": {}}}"#, json_string);
        let another_run: serde_json::Value = serde_json::from_str(&another_run_str).unwrap();
        let mut sub4: Subscription<Vec<u8>> = client2
            .subscribe(
                SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![another_run],
                UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await
            .unwrap();

        let _ = sub3
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .is_err();
        assert!(sub4
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());
        assert!(sub4
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());
        assert!(sub4
            .next()
            .with_timeout(std::time::Duration::from_millis(25000))
            .await
            .unwrap()
            .is_some());
    });

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();
    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_all_bins();
    remove_db(DB);

    Ok(())
}
