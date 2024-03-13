use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, retrieve_output, wait_for_socket_connection,
        ChildGuard, ProcInfo, TimeoutFutureExt, BIN_NAME,
    },
};
use anyhow::Result;
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use once_cell::sync::Lazy;
use std::{
    fs,
    net::Ipv4Addr,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "subscribe_run_workflow";
const UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "unsubscribe_run_workflow";
const AWAIT_CID: &str = "bafyrmic7gy3muoxiyepl44iwrkxxrnkl5oa77scqq3ptuigrm5faiqfgka";

#[test]
#[serial_test::parallel]
fn test_workflow_run_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);
    let homestar_proc = Command::new(BIN.as_os_str())
        .env("RUST_BACKTRACE", "0")
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection(ws_port, 1000).is_err() {
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
        let mut received_cids = 0;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(45)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();
                let check = json.get("metadata").unwrap();
                let expected = serde_json::json!({"name": "test", "replayed": false, "workflow": {"/": format!("{AWAIT_CID}")}});
                assert_eq!(check, &expected);
                received_cids += 1;
            } else {
                panic!("Node one did not publish receipt in time.")
            }

            if received_cids == 3 {
                received_cids = 0;
                break;
            }
        }

        // separate subscription, only 3 events too
        let mut sub2: Subscription<Vec<u8>> = client1
            .subscribe(
                SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![run.clone()],
                UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await
            .unwrap();

        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(45)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();
                let check = json.get("metadata").unwrap();
                let expected = serde_json::json!({"name": "test", "replayed": true, "workflow": {"/": format!("{AWAIT_CID}")}});
                assert_eq!(check, &expected);
                received_cids += 1;
            } else {
                panic!("Node one did not publish receipt in time.")
            }

            if received_cids == 3 {
                received_cids = 0;
                break;
            }
        }

        let client2 = WsClientBuilder::default()
            .build(ws_url.clone())
            .await
            .unwrap();
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
            .with_timeout(Duration::from_secs(10))
            .await
            .is_err();

        loop {
            if let Ok(msg) = sub3.next().with_timeout(Duration::from_secs(45)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();
                let check = json.get("metadata").unwrap();
                let expected = serde_json::json!({"name": "test", "replayed": true, "workflow": {"/": format!("{AWAIT_CID}")}});
                assert_eq!(check, &expected);
                received_cids += 1;
            } else {
                panic!("Node one did not publish receipt in time.")
            }

            if received_cids == 3 {
                received_cids = 0;
                break;
            }
        }

        let _ = sub3
            .next()
            .with_timeout(Duration::from_secs(10))
            .await
            .is_err();

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

        loop {
            if let Ok(msg) = sub4.next().with_timeout(Duration::from_secs(45)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();
                let check = json.get("metadata").unwrap();
                let expected = serde_json::json!({"name": "another_test", "replayed": true, "workflow": {"/": format!("{AWAIT_CID}")}});
                assert_eq!(check, &expected);
                received_cids += 1;
            } else {
                panic!("Node one did not publish receipt in time.")
            }

            if received_cids == 3 {
                break;
            }
        }

        let _ = sub4
            .next()
            .with_timeout(Duration::from_secs(10))
            .await
            .is_err();
    });

    // Collect logs then kill proceses.
    let dead_proc = kill_homestar(proc_guard.take(), None);

    // Retrieve logs.
    let stdout = retrieve_output(dead_proc);

    // Check tht Wasm guest functions logged.
    let crop_ran = check_for_line_with(stdout.clone(), vec!["run-fn", "crop"]);
    let rotate_ran = check_for_line_with(stdout.clone(), vec!["run-fn", "rotate90"]);
    let grayscale_ran = check_for_line_with(stdout, vec!["run-fn", "grayscale"]);

    assert!(crop_ran);
    assert!(rotate_ran);
    assert!(grayscale_ran);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_workflow_run_integration_cbor() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);
    let homestar_proc = Command::new(BIN.as_os_str())
        .env("RUST_BACKTRACE", "0")
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection(ws_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port);

    tokio_test::block_on(async {
        let run_cbor = fs::read("tests/fixtures/test-workflow-image-pipeline.cbor").unwrap();
        let client = WsClientBuilder::default()
            .build(ws_url.clone())
            .await
            .unwrap();

        let mut sub: Subscription<Vec<u8>> = client
            .subscribe(
                SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![run_cbor],
                UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await
            .unwrap();

        // we have 3 operations
        let mut received_cids = 0;
        loop {
            if let Ok(msg) = sub.next().with_timeout(Duration::from_secs(45)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();
                let check = json.get("metadata").unwrap();
                let expected = serde_json::json!({"name": "test", "replayed": false, "workflow": {"/": format!("{AWAIT_CID}")}});
                assert_eq!(check, &expected);
                received_cids += 1;
            } else {
                panic!("Node one did not publish receipt in time.")
            }

            if received_cids == 3 {
                break;
            }
        }

        let _ = sub
            .next()
            .with_timeout(Duration::from_secs(10))
            .await
            .is_err();
    });

    Ok(())
}
