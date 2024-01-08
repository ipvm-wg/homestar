use crate::utils::{
    kill_homestar, remove_db, wait_for_socket_connection, ChildGuard, TimeoutFutureExt, BIN_NAME,
};
use anyhow::Result;
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use once_cell::sync::Lazy;
use std::{
    net::Ipv4Addr,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

#[test]
fn test_connection_notifications_integration() -> Result<()> {
    const DB1: &str = "test_connection_notifications_integration1.db";
    const DB2: &str = "test_connection_notifications_integration2.db";

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_notification1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _guard1 = ChildGuard::new(homestar_proc1);

    let ws_port = 8022;
    if wait_for_socket_connection(8022, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port);
    tokio_test::block_on(async {
        tokio_tungstenite::connect_async(ws_url.clone())
            .await
            .unwrap();

        let client = WsClientBuilder::default()
            .build(ws_url.clone())
            .await
            .unwrap();
        let mut sub: Subscription<Vec<u8>> = client
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        let homestar_proc2 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg("tests/fixtures/test_notification2.toml")
            .arg("--db")
            .arg(DB2)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let guard2 = ChildGuard::new(homestar_proc2);

        // Poll for connection established message
        loop {
            if let Ok(msg) = sub.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:connectionEstablished".to_string() {
                    break;
                }
            } else {
                panic!("Node one did not receive a connection established message in time.")
            }
        }

        let _ = kill_homestar(guard2.take(), None);

        // Poll for connection closed message
        loop {
            if let Ok(msg) = sub.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:connectionClosed".to_string() {
                    break;
                }
            } else {
                panic!("Node one did not receive a connection closed message in time.")
            }
        }

        // Check node endpoint to match
        let http_url = format!("http://localhost:{}", 8022);
        let http_resp = reqwest::get(format!("{}/node", http_url)).await.unwrap();
        assert_eq!(http_resp.status(), 200);
        let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();
        assert_eq!(
            http_resp,
            serde_json::json!({
                "nodeInfo": {
                    "static": {"peer_id": "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"},
                    "dynamic": {"listeners": ["/ip4/127.0.0.1/tcp/7010"], "connections": {}}
                }
            })
        );

        remove_db(DB1);
        remove_db(DB2);
    });

    Ok(())
}
