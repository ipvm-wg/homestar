use crate::utils::{
    check_lines_for, kill_homestar, remove_db, retrieve_output, stop_homestar,
    wait_for_socket_connection, TimeoutFutureExt, BIN_NAME,
};
use anyhow::Result;
use homestar_runtime::{db::Database, Db, Receipt, Settings};
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use libipld::Cid;
use once_cell::sync::Lazy;
use serial_test::file_serial;
use std::{
    net::Ipv4Addr,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
    time::Duration,
};
use tokio::time::sleep;

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

#[test]
#[file_serial]
fn test_libp2p_dht_records() -> Result<()> {
    const DB1: &str = "test_libp2p_dht_records1.db";
    const DB2: &str = "test_libp2p_dht_records2.db";
    let _ = stop_homestar();

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_dht1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let ws_port1 = 7985;
    if wait_for_socket_connection(ws_port1, 1000).is_err() {
        let _ = kill_homestar(homestar_proc1, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::block_on(async {
        let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port1);
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
            .arg("tests/fixtures/test_dht2.toml")
            .arg("--db")
            .arg(DB2)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let ws_port2 = 7986;
        if wait_for_socket_connection(ws_port2, 1000).is_err() {
            let _ = kill_homestar(homestar_proc2, None);
            panic!("Homestar server/runtime failed to start in time");
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

        // Run test workflow with a single task on node one
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9785")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one-part-one.json")
            .output();

        // Poll for put receipt and workflow info messages
        let mut put_receipt = false;
        let mut put_workflow_info = false;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:putReceiptDht" {
                    put_receipt = true;
                }
            } else {
                panic!("Node one did not put receipt in time.")
            }

            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:putWorkflowInfoDht" {
                    put_workflow_info = true;
                }
            } else {
                panic!("Node one did not put workflow info in time.")
            }

            if put_receipt && put_workflow_info {
                break;
            }
        }

        // Run test workflow on node two.
        // The task in this workflow awaits the task run on node one,
        // which forces it to retrieve the result from the DHT.
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9786")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one-part-two.json")
            .output();

        // Poll for got receipt message
        let received_receipt_cid: Cid;
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:gotReceiptDht" {
                    received_receipt_cid = Cid::from_str(json["data"]["cid"].as_str().unwrap())
                        .expect("Unable to parse received receipt CID.");
                    break;
                }
            } else {
                panic!("Node two did not get receipt in time.")
            }
        }

        // Run the same workflow run on node one to retrieve
        // workflow info that should be available on the DHT.
        // This test must be run last or node one will complete
        // the first task on its own and not use the DHT.
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9786")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one-part-one.json")
            .output();

        // Poll for retrieved workflow info message
        let received_workflow_info_cid: Cid;
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:gotWorkflowInfoDht" {
                    received_workflow_info_cid =
                        Cid::from_str(json["data"]["cid"].as_str().unwrap())
                            .expect("Unable to parse received workflow info CID.");
                    break;
                }
            } else {
                panic!("Node two did not get workflow info in time.")
            }
        }

        // Check database for stored receipt and workflow info
        let settings =
            Settings::load_from_file(PathBuf::from("tests/fixtures/test_dht2.toml")).unwrap();
        let db = Db::setup_connection_pool(settings.node(), Some(DB2.to_string()))
            .expect("Failed to connect to node two database");

        let stored_receipt: Receipt =
            Db::find_receipt_by_cid(received_receipt_cid, &mut db.conn().unwrap()).unwrap_or_else(
                |_| {
                    panic!(
                        "Failed to find receipt with CID {} in database",
                        received_receipt_cid
                    )
                },
            );
        let stored_workflow_info =
            Db::get_workflow_info(received_workflow_info_cid, &mut db.conn().unwrap());

        assert_eq!(stored_receipt.cid(), received_receipt_cid);
        assert!(stored_workflow_info.is_ok());

        // Collect logs then kill proceses.
        let dead_proc1 = kill_homestar(homestar_proc1, None);
        let dead_proc2 = kill_homestar(homestar_proc2, None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check node one put receipt and workflow info
        let put_receipt_logged = check_lines_for(stdout1.clone(), vec!["receipt PUT onto DHT"]);
        let put_workflow_info_logged = check_lines_for(stdout1, vec!["workflow info PUT onto DHT"]);

        assert!(put_receipt_logged);
        assert!(put_workflow_info_logged);

        // Check node two received a receipt and workflow info from node one
        let retrieved_receipt_logged = check_lines_for(
            stdout2.clone(),
            vec![
                "found receipt record",
                "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            ],
        );
        let retrieved_workflow_info_logged = check_lines_for(
            stdout2,
            vec![
                "found workflow info",
                "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            ],
        );

        assert!(retrieved_receipt_logged);
        assert!(retrieved_workflow_info_logged);
    });

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_dht_insufficient_quorum_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_dht_insufficient_quorum_serial1.db";
    const DB2: &str = "test_libp2p_dht_insufficient_quorum_serial2.db";
    let _ = stop_homestar();

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_dht3.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let ws_port = 7987;
    if wait_for_socket_connection(ws_port, 1000).is_err() {
        let _ = kill_homestar(homestar_proc1, None);
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
            .arg("tests/fixtures/test_dht4.toml")
            .arg("--db")
            .arg(DB2)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let ws_port2 = 7988;
        if wait_for_socket_connection(ws_port2, 1000).is_err() {
            let _ = kill_homestar(homestar_proc1, None);
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

        // Run test workflow
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9787")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Wait for workflow to run
        // TODO Listen on an event instead of using an arbitrary sleep
        sleep(Duration::from_secs(1)).await;

        // Collect logs then kill proceses.
        let dead_proc1 = kill_homestar(homestar_proc1, None);
        let _ = kill_homestar(homestar_proc2, None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);

        // Check that DHT put record failed
        let put_failed = check_lines_for(stdout1, vec!["QuorumFailed", "error putting record"]);

        assert!(put_failed);
    });

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}
