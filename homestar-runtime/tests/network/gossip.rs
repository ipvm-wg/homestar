use crate::utils::{
    check_for_line_with, kill_homestar, retrieve_output, wait_for_socket_connection, ChildGuard,
    FileGuard, TimeoutFutureExt, BIN_NAME,
};
use anyhow::Result;
use homestar_runtime::{db::Database, Db, Settings};
use itertools::Itertools;
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use libipld::Cid;
use once_cell::sync::Lazy;
use std::{
    net::Ipv4Addr,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

#[test]
fn test_libp2p_receipt_gossip_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_receipt_gossip_integration1.db";
    const DB2: &str = "_test_libp2p_receipt_gossip_integration2.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_gossip1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    let ws_port = 7990;
    if wait_for_socket_connection(ws_port, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::block_on(async {
        let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port);
        let client = WsClientBuilder::default()
            .build(ws_url.clone())
            .await
            .unwrap();

        let mut sub1: Subscription<Vec<u8>> = client
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
            .arg("tests/fixtures/test_gossip2.toml")
            .arg("--db")
            .arg(DB2)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let proc_guard2 = ChildGuard::new(homestar_proc2);

        let ws_port2 = 7991;
        if wait_for_socket_connection(ws_port2, 100).is_err() {
            panic!("Homestar server/runtime failed to start in time");
        }

        // Poll for connection established message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:connectionEstablished" {
                    break;
                }
            } else {
                panic!("Node one did not establish a connection with node two in time.")
            }
        }

        let ws_url2 = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port2);
        let client2 = WsClientBuilder::default()
            .build(ws_url2.clone())
            .await
            .unwrap();

        let mut sub2: Subscription<Vec<u8>> = client2
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        // Run test workflow on node one
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9790")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for published receipt messages
        let mut published_cids: Vec<Cid> = vec![];
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:publishedReceiptPubsub" {
                    published_cids.push(
                        Cid::from_str(json["data"]["cid"].as_str().unwrap())
                            .expect("Unable to parse published receipt CID."),
                    );
                }
            } else {
                panic!("Node one did not publish receipt in time.")
            }

            if published_cids.len() == 2 {
                break;
            }
        }

        // Poll for received receipt messages
        let mut received_cids: Vec<Cid> = vec![];
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:receivedReceiptPubsub" {
                    received_cids.push(
                        Cid::from_str(json["data"]["cid"].as_str().unwrap())
                            .expect("Unable to parse received receipt CID."),
                    );
                }
            } else {
                panic!("Node two did not receive receipt in time.")
            }

            if received_cids.len() == 2 {
                break;
            }
        }

        // Collect logs then kill proceses.
        let dead_proc1 = kill_homestar(proc_guard1.take(), None);
        let dead_proc2 = kill_homestar(proc_guard2.take(), None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check node one published a receipt
        let message_published =
            check_for_line_with(stdout1, vec!["message published on receipts topic"]);

        // Check node two received a receipt from node one
        let message_received = check_for_line_with(
            stdout2,
            vec![
                "message received on receipts topic",
                "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            ],
        );

        assert!(message_published);
        assert!(message_received);

        let settings =
            Settings::load_from_file(PathBuf::from("tests/fixtures/test_gossip2.toml")).unwrap();
        let db = Db::setup_connection_pool(settings.node(), Some(DB2.to_string()))
            .expect("Failed to connect to node two database");

        // Check database for stored receipts
        let stored_receipts: Vec<_> = received_cids
            .iter()
            .map(|cid| {
                Db::find_receipt_by_cid(*cid, &mut db.conn().unwrap()).unwrap_or_else(|_| {
                    panic!("Failed to find receipt with CID {} in database", *cid)
                })
            })
            .collect_vec();

        assert_eq!(stored_receipts.len(), 2)
    });

    Ok(())
}
