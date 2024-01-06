use crate::utils::{
    check_for_line_with, kill_homestar, remove_db, retrieve_output, stop_homestar,
    wait_for_socket_connection, TimeoutFutureExt, BIN_NAME,
};
use anyhow::Result;
use diesel::RunQueryDsl;
use homestar_runtime::{
    db::{self, schema, Database},
    Db, Settings,
};
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

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

#[test]
#[file_serial]
fn test_libp2p_dht_records_serial() -> Result<()> {
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

    let ws_port1 = 7980;
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

        let ws_port2 = 7981;
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
            .arg("9780")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one-part-one.json")
            .output();

        // Poll for put receipt and workflow info messages
        let mut put_receipt = false;
        let mut put_workflow_info = false;
        let mut receipt_quorum_success = false;
        let mut workflow_info_quorum_success = false;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:putReceiptDht" {
                    put_receipt = true;
                } else if json["type"].as_str().unwrap() == "network:putWorkflowInfoDht" {
                    put_workflow_info = true;
                } else if json["type"].as_str().unwrap() == "network:receiptQuorumSuccess" {
                    receipt_quorum_success = true;
                } else if json["type"].as_str().unwrap() == "network:workflowInfoQuorumSuccess" {
                    workflow_info_quorum_success = true;
                }
            } else {
                panic!(
                    r#"Expected notifications from node one did not arrive in time:
  - Put receipt to DHT: {}
  - Put workflow info to DHT: {}
  - Receipt quorum succeeded: {}
  - Workflow info quorum succeeded: {}
  "#,
                    put_receipt,
                    put_workflow_info,
                    receipt_quorum_success,
                    workflow_info_quorum_success
                );
            }

            if put_receipt
                && put_workflow_info
                && receipt_quorum_success
                && workflow_info_quorum_success
            {
                break;
            }
        }

        // TODO Bring back tests for receipts retrieved from DHT
        // both here and below.

        // Run test workflow on node two.
        // The task in this workflow awaits the task run on node one,
        // which forces it to retrieve the result from the DHT.
        // let _ = Command::new(BIN.as_os_str())
        //     .arg("run")
        //     .arg("-p")
        //     .arg("9781")
        //     .arg("-w")
        //     .arg("tests/fixtures/test-workflow-add-one-part-two.json")
        //     .output();

        // Poll for got receipt message
        // let received_receipt_cid: Cid;
        // loop {
        //     if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(120)).await {
        //         let json: serde_json::Value =
        //             serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

        //         if json["type"].as_str().unwrap() == "network:gotReceiptDht" {
        //             received_receipt_cid = Cid::from_str(json["data"]["cid"].as_str().unwrap())
        //                 .expect("Unable to parse received receipt CID.");
        //             break;
        //         }
        //     } else {
        //         panic!("Node two did not get receipt in time.")
        //     }
        // }

        // Run the same workflow run on node one to retrieve
        // workflow info that should be available on the DHT.
        // This test must be run last or node one will complete
        // the first task on its own and not use the DHT.
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9781")
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

        // let stored_receipt: Receipt =
        //     Db::find_receipt_by_cid(received_receipt_cid, &mut db.conn().unwrap()).unwrap_or_else(
        //         |_| {
        //             panic!(
        //                 "Failed to find receipt with CID {} in database",
        //                 received_receipt_cid
        //             )
        //         },
        //     );
        let stored_workflow_info =
            Db::get_workflow_info(received_workflow_info_cid, &mut db.conn().unwrap());

        // assert_eq!(stored_receipt.cid(), received_receipt_cid);
        assert!(stored_workflow_info.is_ok());

        // Collect logs then kill proceses.
        let dead_proc1 = kill_homestar(homestar_proc1, None);
        let dead_proc2 = kill_homestar(homestar_proc2, None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check node one put receipt and workflow info
        let put_receipt_logged = check_for_line_with(stdout1.clone(), vec!["receipt PUT onto DHT"]);
        let put_workflow_info_logged =
            check_for_line_with(stdout1.clone(), vec!["workflow info PUT onto DHT"]);
        let receipt_quorum_success_logged =
            check_for_line_with(stdout1.clone(), vec!["quorum success for receipt record"]);
        let workflow_info_quorum_success_logged =
            check_for_line_with(stdout1, vec!["quorum success for workflow info record"]);

        assert!(put_receipt_logged);
        assert!(put_workflow_info_logged);
        assert!(receipt_quorum_success_logged);
        assert!(workflow_info_quorum_success_logged);

        // // Check node two received a receipt and workflow info from node one
        // let retrieved_receipt_logged = check_for_line_with(
        //     stdout2.clone(),
        //     vec![
        //         "found receipt record",
        //         "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        //     ],
        // );
        let retrieved_workflow_info_logged = check_for_line_with(
            stdout2,
            vec![
                "found workflow info",
                "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            ],
        );

        // assert!(retrieved_receipt_logged);
        assert!(retrieved_workflow_info_logged);
    });

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_dht_quorum_failure_serial() -> Result<()> {
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

    let ws_port = 7982;
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

        let ws_port2 = 7983;
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
            .arg("9782")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for quorum failure messages
        let mut receipt_quorum_failure = false;
        let mut workflow_info_quorum_failure = false;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:receiptQuorumFailure" {
                    receipt_quorum_failure = true
                } else if json["type"].as_str().unwrap() == "network:workflowInfoQuorumFailure" {
                    workflow_info_quorum_failure = true
                }
            } else {
                panic!(
                    r#"Expected notifications from node one did not arrive in time:
  - Receipt quorum failure: {}
  - Workflow info failure: {}
  "#,
                    receipt_quorum_failure, workflow_info_quorum_failure
                );
            }

            if receipt_quorum_failure && workflow_info_quorum_failure {
                break;
            }
        }

        // Collect logs then kill proceses.
        let dead_proc1 = kill_homestar(homestar_proc1, None);
        let _ = kill_homestar(homestar_proc2, None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);

        // Check that receipt and workflow info quorums failed
        let receipt_quorum_failure_logged = check_for_line_with(
            stdout1.clone(),
            vec!["QuorumFailed", "error propagating receipt record"],
        );
        let workflow_info_quorum_failure_logged = check_for_line_with(
            stdout1,
            vec!["QuorumFailed", "error propagating workflow info record"],
        );

        assert!(receipt_quorum_failure_logged);
        assert!(workflow_info_quorum_failure_logged);
    });

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_dht_workflow_info_provider_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_dht_workflow_info_provider_records1.db";
    const DB2: &str = "test_libp2p_dht_workflow_info_provider_records2.db";
    let _ = stop_homestar();

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_dht5.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let ws_port1 = 7984;
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
            .arg("tests/fixtures/test_dht6.toml")
            .arg("--db")
            .arg(DB2)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let ws_port2 = 7985;
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

        // Run test workflow on node one
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9784")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // TODO Should not fail with this timeout present
        // tokio::time::sleep(Duration::from_secs(5)).await

        // Run the same workflow run on node two.
        // Node two should be request workflow info from
        // node one instead of waiting to get the record
        // from the DHT.
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9785")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for sent workflow info message
        let sent_workflow_info_cid: Cid;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(60)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:sentWorkflowInfo" {
                    sent_workflow_info_cid = Cid::from_str(json["data"]["cid"].as_str().unwrap())
                        .expect("Unable to parse sent workflow info CID.");
                    break;
                }
            } else {
                panic!("Node one did not send workflow info in time.")
            }
        }

        assert_eq!(
            sent_workflow_info_cid.to_string(),
            "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq"
        );

        // Poll for retrieved workflow info message
        let received_workflow_info_cid: Cid;
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["type"].as_str().unwrap() == "network:receivedWorkflowInfo" {
                    received_workflow_info_cid =
                        Cid::from_str(json["data"]["cid"].as_str().unwrap())
                            .expect("Unable to parse received workflow info CID.");
                    break;
                }
            } else {
                panic!("Node two did not get workflow info in time.")
            }
        }

        assert_eq!(
            received_workflow_info_cid.to_string(),
            "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq"
        );

        // Check database for workflow info
        let settings =
            Settings::load_from_file(PathBuf::from("tests/fixtures/test_dht6.toml")).unwrap();
        let db = Db::setup_connection_pool(settings.node(), Some(DB2.to_string()))
            .expect("Failed to connect to node two database");

        let stored_workflow_info =
            Db::get_workflow_info(received_workflow_info_cid, &mut db.conn().unwrap());

        assert!(stored_workflow_info.is_ok());

        // Collect logs then kill proceses.
        let dead_proc1 = kill_homestar(homestar_proc1, None);
        let dead_proc2 = kill_homestar(homestar_proc2, None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check node one providing workflow info
        let providing_workflow_info_logged = check_for_line_with(
            stdout1.clone(),
            vec![
                "successfully providing",
                "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq",
            ],
        );

        // Check node two got workflow info providers
        let got_workflow_info_provider_logged = check_for_line_with(
            stdout2.clone(),
            vec![
                "got workflow info providers",
                "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            ],
        );

        // Check node one sent workflow info
        let sent_workflow_info_logged = check_for_line_with(
            stdout1.clone(),
            vec![
                "sent workflow info to peer",
                "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
                "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq",
            ],
        );

        // Check node two received workflow info
        let received_workflow_info_logged = check_for_line_with(
            stdout2.clone(),
            vec![
                "received workflow info from peer",
                "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
                "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq",
            ],
        );

        assert!(providing_workflow_info_logged);
        assert!(got_workflow_info_provider_logged);
        assert!(sent_workflow_info_logged);
        assert!(received_workflow_info_logged);
    });

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_dht_workflow_info_provider_recursive_serial() -> Result<()> {
    // Start 3 nodes (a, b, c):
    // - a peers with b and c
    // - b peers with a
    // - c peers with a
    //
    // 1. Start a, b, and c
    // 2. Wait for connection between a and b to be established
    // 3. Wait for connection between a and c to be established
    // 4. Run workflow on a
    // 5. Wait for network:putWorkflowInfoDht on a
    // 6. Run workflow on b
    // 7. Wait for network:GotWorkflowInfoDht on b
    // 8. Delete a's DB
    // 9. Run workflow on c
    // 10. Wait for network:receivedWorkflowInfo on c (from b, through a)

    const DB1: &str = "test_libp2p_dht_workflow_info_provider_recursive1.db";
    const DB2: &str = "test_libp2p_dht_workflow_info_provider_recursive2.db";
    const DB3: &str = "test_libp2p_dht_workflow_info_provider_recursive3.db";
    let _ = stop_homestar();

    tokio_test::block_on(async move {
        let homestar_proc1 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg("tests/fixtures/test_dht7.toml")
            .arg("--db")
            .arg(DB1)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let ws_port1 = 7986;
        if wait_for_socket_connection(ws_port1, 1000).is_err() {
            let _ = kill_homestar(homestar_proc1, None);
            panic!("Homestar server/runtime failed to start in time");
        }

        let ws_url1 = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port1);
        let client1 = WsClientBuilder::default()
            .build(ws_url1.clone())
            .await
            .unwrap();

        let mut sub1: Subscription<Vec<u8>> = client1
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
            .arg("tests/fixtures/test_dht8.toml")
            .arg("--db")
            .arg(DB2)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let ws_port2 = 7987;
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

        let homestar_proc3 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg("tests/fixtures/test_dht9.toml")
            .arg("--db")
            .arg(DB3)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let ws_port3 = 7988;
        if wait_for_socket_connection(ws_port3, 1000).is_err() {
            let _ = kill_homestar(homestar_proc3, None);
            panic!("Homestar server/runtime failed to start in time");
        }

        let ws_url3 = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port3);
        let client3 = WsClientBuilder::default()
            .build(ws_url3.clone())
            .await
            .unwrap();

        let mut sub3: Subscription<Vec<u8>> = client3
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        // Poll node one for connection established with node two message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                println!("node1: {json}");

                if json["type"].as_str().unwrap() == "network:connectionEstablished" {
                    assert_eq!(
                        json["data"]["peerId"],
                        "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc"
                    );

                    break;
                }
            } else {
                panic!("Node one did not establish a connection with node two in time.")
            }
        }

        // Poll node one for connection established with node three message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                println!("node1: {json}");

                if json["type"].as_str().unwrap() == "network:connectionEstablished" {
                    assert_eq!(
                        json["data"]["peerId"],
                        "12D3KooWK99VoVxNE7XzyBwXEzW7xhK7Gpv85r9F3V3fyKSUKPH5"
                    );

                    break;
                }
            } else {
                panic!("Node one did not establish a connection with node three in time.")
            }
        }

        // Run test workflow on node one
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9786")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for put workflow info messages
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                println!("node1: {json}");

                if json["type"].as_str().unwrap() == "network:putWorkflowInfoDht" {
                    assert_eq!(
                        json["data"]["cid"].as_str().unwrap(),
                        "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq"
                    );

                    break;
                }
            } else {
                panic!("Node one did not put workflow info in time.")
            }
        }

        // Run the same workflow run on node two
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9787")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for got workflow info messages on node two
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                println!("node2: {json}");

                if json["type"].as_str().unwrap() == "network:gotWorkflowInfoDht" {
                    assert_eq!(
                        json["data"]["cid"].as_str().unwrap(),
                        "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq"
                    );

                    break;
                }
            } else {
                panic!("Node two did not get workflow info in time.")
            }
        }

        let db =
            db::Db::setup_connection_pool(&Settings::load().unwrap().node(), Some(DB1.to_string()))
                .unwrap();

        diesel::delete(schema::workflows_receipts::table)
            .execute(&mut db.conn().unwrap())
            .unwrap();

        diesel::delete(schema::workflows::table)
            .execute(&mut db.conn().unwrap())
            .unwrap();

        // Run the workflow on node three.
        // We expect node three to request workflow info
        // from node one, which claims it is a provider. But
        // node one no longer has the workflow info and should
        // request it from node two.
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg("9788")
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for received workflow info messages on node three, which
        // should come from node one
        loop {
            if let Ok(msg) = sub3.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                println!("node3: {json}");

                if json["type"].as_str().unwrap() == "network:receivedWorkflowInfo" {
                    assert_eq!(
                        json["data"]["provider"],
                        "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc"
                    );

                    assert_eq!(
                        json["data"]["cid"].as_str().unwrap(),
                        "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq"
                    );

                    break;
                }
            } else {
                panic!("Node three did not receive workflow info in time.")
            }
        }

        // TODO Check that node three stored the workflow info in its database.

        let _ = kill_homestar(homestar_proc1, None);
        let _ = kill_homestar(homestar_proc2, None);
        let _ = kill_homestar(homestar_proc3, None);

        // TODO Check for logs that indicate:
        //   - Node three sent workflow info as a provider
        //   - Node one received workflow info from node two provider
        //   - Node one forwarded workflow info to node three
        //   - Node three received the workflow info from node one

        remove_db(DB1);
        remove_db(DB2);
        remove_db(DB3);
    });

    Ok(())
}
