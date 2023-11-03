#[cfg(feature = "ipfs")]
use crate::utils::startup_ipfs;
use crate::utils::{kill_homestar, stop_all_bins, BIN_NAME, IPFS};
use anyhow::Result;
use futures::StreamExt;
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use once_cell::sync::Lazy;
use retry::{delay::Exponential, retry};
use serial_test::file_serial;
use std::{
    fs,
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "subscribe_run_workflow";
const UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "unsubscribe_run_workflow";

#[test]
#[file_serial]
fn test_workflow_run_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

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

    let _ = fs::remove_file("homestar_test_workflow_run_serial.db");

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

    let mut homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_workflow2.toml")
        .arg("--db")
        .arg("homestar_test_workflow_run_serial.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let ws_port = 8061;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), ws_port);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        homestar_proc.kill().unwrap();
        panic!("Homestar server/runtime failed to start in time");
    }

    let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port);

    tokio_test::block_on(async {
        tokio_tungstenite::connect_async(ws_url.clone())
            .await
            .unwrap();

        let workflow_str =
            fs::read_to_string("tests/fixtures/test-workflow-image-pipeline.json").unwrap();
        let json: serde_json::Value = serde_json::from_str(&workflow_str).unwrap();
        let json_string = serde_json::to_string(&json).unwrap();
        let run_str = format!(r#"{{"name": "test","workflow": {}}}"#, json_string);
        let run: serde_json::Value = serde_json::from_str(&run_str).unwrap();

        let client = WsClientBuilder::default().build(ws_url).await.unwrap();
        let sub: Subscription<Vec<u8>> = client
            .subscribe(
                SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![run],
                UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await
            .unwrap();

        // we have 3 operations0
        sub.take(3)
            .for_each(|msg| async move {
                let json: serde_json::Value = serde_json::from_slice(&msg.unwrap()).unwrap();
                let check = json.get("metadata").unwrap();
                let expected1 = serde_json::json!({"name": "test", "replayed": true, "workflow": {"/": "bafyrmicvwgispoezdciv5z6w3coutfjjtnhtmbegpcrrocqd76y7dvtknq"}});
                let expected2 = serde_json::json!({"name": "test", "replayed": false, "workflow": {"/": "bafyrmicvwgispoezdciv5z6w3coutfjjtnhtmbegpcrrocqd76y7dvtknq"}});
                if check == &expected1 || check == &expected2 {
                    println!("JSONRPC response is expected");
                } else {
                    panic!("JSONRPC response is not expected");
                }
            })
            .await;
    });

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();
    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_all_bins();

    Ok(())
}
