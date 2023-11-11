use crate::utils::{
    kill_homestar, stop_homestar, wait_for_socket_connection, TimeoutFutureExt, BIN_NAME,
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
    net::Ipv4Addr,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

#[test]
#[file_serial]
fn test_connection_notifications_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_notification1.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let ws_port = 8022;
    if wait_for_socket_connection(8022, 1000).is_err() {
        let _ = kill_homestar(homestar_proc1, None);
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
            .arg("homestar2.db")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

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

        let _ = kill_homestar(homestar_proc2, None);

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

        let _ = kill_homestar(homestar_proc1, None);
    });

    Ok(())
}
