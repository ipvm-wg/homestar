use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, listen_addr, multiaddr, retrieve_output,
        wait_for_socket_connection, ChildGuard, ProcInfo, TimeoutFutureExt, BIN_NAME,
        ED25519MULTIHASH, ED25519MULTIHASH2, ED25519MULTIHASH3, ED25519MULTIHASH5,
        SECP256K1MULTIHASH,
    },
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
#[serial_test::parallel]
fn test_libp2p_dht_records_integration() -> Result<()> {
    let proc_info1 = ProcInfo::new().unwrap();
    let proc_info2 = ProcInfo::new().unwrap();

    let rpc_port1 = proc_info1.rpc_port;
    let rpc_port2 = proc_info2.rpc_port;
    let metrics_port1 = proc_info1.metrics_port;
    let metrics_port2 = proc_info2.metrics_port;
    let ws_port1 = proc_info1.ws_port;
    let ws_port2 = proc_info2.ws_port;
    let listen_addr1 = listen_addr(proc_info1.listen_port);
    let listen_addr2 = listen_addr(proc_info2.listen_port);
    let node_addra = multiaddr(proc_info1.listen_port, ED25519MULTIHASH);
    let node_addrb = multiaddr(proc_info2.listen_port, SECP256K1MULTIHASH);
    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        node_addresses = ["{node_addrb}"]
        [node.network.libp2p.dht]
        p2p_receipt_timeout = 3000
        p2p_workflow_info_timeout = 3000
        receipt_quorum = 1
        workflow_quorum = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg(config1.filename())
        .arg("--db")
        .arg(&proc_info1.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
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

        let toml2 = format!(
            r#"
            [node]
            [node.network.keypair_config]
            existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
            [node.network.libp2p]
            listen_address = "{listen_addr2}"
            node_addresses = ["{node_addra}"]
            [node.network.libp2p.dht]
            p2p_receipt_timeout = 3000
            p2p_workflow_info_timeout = 3000
            receipt_quorum = 1
            workflow_quorum = 1
            [node.network.libp2p.mdns]
            enable = false
            [node.network.libp2p.pubsub]
            enable = false
            [node.network.libp2p.rendezvous]
            enable_client = false
            [node.network.metrics]
            port = {metrics_port2}
            [node.network.rpc]
            port = {rpc_port2}
            [node.network.webserver]
            port = {ws_port2}
            "#
        );
        let config2 = make_config!(toml2);

        let homestar_proc2 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg(config2.filename())
            .arg("--db")
            .arg(&proc_info2.db_path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let proc_guard2 = ChildGuard::new(homestar_proc2);

        if wait_for_socket_connection(ws_port2, 1000).is_err() {
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

                if json["connection_established"].is_object() {
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
            .arg(rpc_port1.to_string())
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

                if json["put_receipt_dht"].is_object() {
                    put_receipt = true;
                } else if json["put_workflow_info_dht"].is_object() {
                    put_workflow_info = true;
                } else if json["receipt_quorum_success_dht"].is_object() {
                    receipt_quorum_success = true;
                } else if json["workflow_info_quorum_success_dht"].is_object() {
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
        //     .arg(rpc_port2.to_string())
        //     .arg("-w")
        //     .arg("tests/fixtures/test-workflow-add-one-part-two.json")
        //     .output();

        // Poll for got receipt message
        // let received_receipt_cid: Cid;
        // loop {
        //     if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(120)).await {
        //         let json: serde_json::Value =
        //             serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

        //         if json["got_receipt_dht"].is_object() {
        //             received_receipt_cid = Cid::from_str(json["got_receipt_dht"]["cid"].as_str().unwrap())
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
            .arg(rpc_port2.to_string())
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one-part-one.json")
            .output();

        // Poll for retrieved workflow info message
        let received_workflow_info_cid: Cid;
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["got_workflow_info_dht"].is_object() {
                    received_workflow_info_cid =
                        Cid::from_str(json["got_workflow_info_dht"]["cid"].as_str().unwrap())
                            .expect("Unable to parse received workflow info CID.");
                    break;
                }
            } else {
                panic!("Node two did not get workflow info in time.")
            }
        }

        // Check database for stored receipt and workflow info
        let config_fixture = config2.filename();
        let settings = Settings::load_from_file(PathBuf::from(config_fixture)).unwrap();
        let db = Db::setup_connection_pool(
            settings.node(),
            Some(proc_info2.db_path.display().to_string()),
        )
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
        let dead_proc1 = kill_homestar(proc_guard1.take(), None);
        let dead_proc2 = kill_homestar(proc_guard2.take(), None);

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
        //         ED25519MULTIHASH,
        //     ],
        // );
        let retrieved_workflow_info_logged =
            check_for_line_with(stdout2, vec!["found workflow info", ED25519MULTIHASH]);

        // assert!(retrieved_receipt_logged);
        assert!(retrieved_workflow_info_logged);
    });

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_dht_quorum_failure_intregration() -> Result<()> {
    let proc_info1 = ProcInfo::new().unwrap();
    let proc_info2 = ProcInfo::new().unwrap();

    let rpc_port1 = proc_info1.rpc_port;
    let rpc_port2 = proc_info2.rpc_port;
    let metrics_port1 = proc_info1.metrics_port;
    let metrics_port2 = proc_info2.metrics_port;
    let ws_port1 = proc_info1.ws_port;
    let ws_port2 = proc_info2.ws_port;
    let listen_addr1 = listen_addr(proc_info1.listen_port);
    let listen_addr2 = listen_addr(proc_info2.listen_port);
    let node_addra = multiaddr(proc_info2.listen_port, ED25519MULTIHASH3);
    let node_addrb = multiaddr(proc_info1.listen_port, ED25519MULTIHASH2);
    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.dht]
        receipt_quorum = 100
        workflow_quorum = 100
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg(config1.filename())
        .arg("--db")
        .arg(&proc_info1.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
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

        let toml2 = format!(
            r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_3.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        node_addresses = ["{node_addrb}"]
        [node.network.libp2p.dht]
        receipt_quorum = 100
        workflow_quorum = 100
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
        );
        let config2 = make_config!(toml2);

        let homestar_proc2 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg(config2.filename())
            .arg("--db")
            .arg(&proc_info2.db_path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let proc_guard2 = ChildGuard::new(homestar_proc2);

        if wait_for_socket_connection(ws_port2, 1000).is_err() {
            panic!("Homestar server/runtime failed to start in time");
        }

        // Poll for connection established message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_established"].is_object() {
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
            .arg(rpc_port1.to_string())
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

                if json["receipt_quorum_failure_dht"].is_object() {
                    receipt_quorum_failure = true
                }

                if json["receipt_quorum_failure_dht"].is_object() {
                    receipt_quorum_failure = true
                } else if json["workflow_info_quorum_failure_dht"].is_object() {
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
        let dead_proc1 = kill_homestar(proc_guard1.take(), None);
        let _dead_proc2 = kill_homestar(proc_guard2.take(), None);

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

    Ok(())
}

#[test]
#[allow(unused_must_use)]
#[flaky_test::flaky_test]
#[serial_test::serial]
fn test_libp2p_dht_workflow_info_provider_integration() -> Result<()> {
    let proc_info1 = ProcInfo::new().unwrap();
    let proc_info2 = ProcInfo::new().unwrap();

    let rpc_port1 = proc_info1.rpc_port;
    let rpc_port2 = proc_info2.rpc_port;
    let metrics_port1 = proc_info1.metrics_port;
    let metrics_port2 = proc_info2.metrics_port;
    let ws_port1 = proc_info1.ws_port;
    let ws_port2 = proc_info2.ws_port;
    let listen_addr1 = listen_addr(proc_info1.listen_port);
    let listen_addr2 = listen_addr(proc_info2.listen_port);
    let node_addra = multiaddr(proc_info1.listen_port, ED25519MULTIHASH2);
    let node_addrb = multiaddr(proc_info2.listen_port, ED25519MULTIHASH5);
    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        idle_connection_timeout = 240
        listen_address = "{listen_addr1}"
        node_addresses = ["{node_addrb}"]
        [node.network.libp2p.dht]
        receipt_quorum = 1
        workflow_quorum = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg(config1.filename())
        .arg("--db")
        .arg(&proc_info1.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
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

        let toml2 = format!(
            r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_5.pem" }}
        [node.network.libp2p]
        idle_connection_timeout = 240
        listen_address = "{listen_addr2}"
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.dht]
        p2p_workflow_info_timeout = 0
        receipt_quorum = 1
        workflow_quorum = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
        );
        let config2 = make_config!(toml2);

        let homestar_proc2 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg(config2.filename())
            .arg("--db")
            .arg(&proc_info2.db_path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let proc_guard2 = ChildGuard::new(homestar_proc2);

        if wait_for_socket_connection(ws_port2, 1000).is_err() {
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

                if json["connection_established"].is_object() {
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
            .arg(rpc_port1.to_string())
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // We want node two to request workflow info directly from node one
        // because of timeouts not because workflow info was missing from the
        // DHT, so we give node one time to put add workflow info to the DHT.
        tokio::time::sleep(Duration::from_secs(9)).await;

        // Run the same workflow run on node two.
        // Node two should be request workflow info from
        // node one instead of waiting to get the record
        // from the DHT.
        let _ = Command::new(BIN.as_os_str())
            .arg("run")
            .arg("-p")
            .arg(rpc_port2.to_string())
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for sent workflow info message
        let sent_workflow_info_cid: Cid;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["sent_workflow_info"].is_object() {
                    sent_workflow_info_cid =
                        Cid::from_str(json["sent_workflow_info"]["cid"].as_str().unwrap())
                            .expect("Unable to parse sent workflow info CID.");
                    break;
                }
            } else {
                panic!("Node one did not send workflow info in time.")
            }
        }

        assert_eq!(
            sent_workflow_info_cid.to_string(),
            "bafyrmibetj4cwo5lfz63zc4qtjvs4xmzvsxucggruo6rnvw7x62fggrii4"
        );

        // Poll for retrieved workflow info message
        let received_workflow_info_cid: Cid;
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(60)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["received_workflow_info"].is_object() {
                    received_workflow_info_cid =
                        Cid::from_str(json["received_workflow_info"]["cid"].as_str().unwrap())
                            .expect("Unable to parse received workflow info CID.");
                    break;
                }
            } else {
                panic!("Node two did not get workflow info in time.")
            }
        }

        assert_eq!(
            received_workflow_info_cid.to_string(),
            "bafyrmibetj4cwo5lfz63zc4qtjvs4xmzvsxucggruo6rnvw7x62fggrii4"
        );

        // Check database for workflow info
        let settings = Settings::load_from_file(PathBuf::from(config2.filename())).unwrap();
        let db = Db::setup_connection_pool(
            settings.node(),
            Some(proc_info2.db_path.display().to_string()),
        )
        .expect("Failed to connect to node two database");

        let stored_workflow_info =
            Db::get_workflow_info(received_workflow_info_cid, &mut db.conn().unwrap());

        assert!(stored_workflow_info.is_ok());

        // Collect logs then kill proceses.
        let dead_proc1 = kill_homestar(proc_guard1.take(), None);
        let dead_proc2 = kill_homestar(proc_guard2.take(), None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check node one providing workflow info
        let providing_workflow_info_logged = check_for_line_with(
            stdout1.clone(),
            vec![
                "successfully providing",
                "bafyrmibetj4cwo5lfz63zc4qtjvs4xmzvsxucggruo6rnvw7x62fggrii4",
            ],
        );

        // Check node two got workflow info providers
        let got_workflow_info_provider_logged = check_for_line_with(
            stdout2.clone(),
            vec!["got workflow info providers", ED25519MULTIHASH2],
        );

        // Check node one sent workflow info
        let sent_workflow_info_logged = check_for_line_with(
            stdout1.clone(),
            vec![
                "sent workflow info to peer",
                ED25519MULTIHASH5,
                "bafyrmibetj4cwo5lfz63zc4qtjvs4xmzvsxucggruo6rnvw7x62fggrii4",
            ],
        );

        // Check node two received workflow info
        let received_workflow_info_logged = check_for_line_with(
            stdout2.clone(),
            vec![
                "received workflow info from peer",
                ED25519MULTIHASH2,
                "bafyrmibetj4cwo5lfz63zc4qtjvs4xmzvsxucggruo6rnvw7x62fggrii4",
            ],
        );

        assert!(providing_workflow_info_logged);
        assert!(got_workflow_info_provider_logged);
        assert!(sent_workflow_info_logged);
        assert!(received_workflow_info_logged);
    });

    Ok(())
}

#[ignore]
#[test]
#[serial_test::parallel]
fn test_libp2p_dht_workflow_info_provider_recursive_integration() -> Result<()> {
    // NOTE: We are ignoring this test for now because we do not have a means
    // to properly isolate node a from node c. In the future when nodes are
    // partitioned as private nodes or from NATs, we will bring this test back.
    //
    // Start 3 nodes (a, b, c):
    // - a peers with b and c
    // - b peers with a
    // - c peers with a
    //
    // 1. Start a, b, and c
    // 2. Wait for connection between a and b to be established
    // 3. Wait for connection between a and c to be established
    // 4. Run workflow on a
    // 5. Wait for put_workflow_info_dht on a
    // 6. Run workflow on b
    // 7. Wait for got_workflow_info_dht on b
    // 8. Delete a's DB
    // 9. Run workflow on c
    // 10. Wait for network:receivedWorkflowInfo on c (from b, through a)
    let proc_info1 = ProcInfo::new().unwrap();
    let proc_info2 = ProcInfo::new().unwrap();
    let proc_info3 = ProcInfo::new().unwrap();

    let rpc_port1 = proc_info1.rpc_port;
    let rpc_port2 = proc_info2.rpc_port;
    let rpc_port3 = proc_info3.rpc_port;
    let metrics_port1 = proc_info1.metrics_port;
    let metrics_port2 = proc_info2.metrics_port;
    let metrics_port3 = proc_info3.metrics_port;
    let ws_port1 = proc_info1.ws_port;
    let ws_port2 = proc_info2.ws_port;
    let ws_port3 = proc_info3.ws_port;
    let listen_addr1 = listen_addr(proc_info1.listen_port);
    let listen_addr2 = listen_addr(proc_info2.listen_port);
    let listen_addr3 = listen_addr(proc_info3.listen_port);
    let node_addra = multiaddr(proc_info1.listen_port, ED25519MULTIHASH);
    let node_addrb = multiaddr(proc_info2.listen_port, SECP256K1MULTIHASH);
    let node_addrc = multiaddr(proc_info3.listen_port, ED25519MULTIHASH2);
    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        node_addresses = ["{node_addrb}", "{node_addrc}"]
        # Force node one to request from node two
        # as a provider instead of from DHT
        p2p_workflow_info_timeout = 0
        p2p_provider_timeout = 10000
        receipt_quorum = 1
        workflow_quorum = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    tokio_test::block_on(async move {
        let homestar_proc1 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg(config1.filename())
            .arg("--db")
            .arg(&proc_info1.db_path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let _proc_guard1 = ChildGuard::new(homestar_proc1);

        if wait_for_socket_connection(ws_port1, 1000).is_err() {
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

        let toml2 = format!(
            r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        node_addresses = ["{node_addra}"]
        # Allow node two to request workflow info from DHT
        p2p_workflow_info_timeout = 5000
        p2p_provider_timeout = 0
        receipt_quorum = 1
        workflow_quorum = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
        );
        let config2 = make_config!(toml2);

        let homestar_proc2 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg(config2.filename())
            .arg("--db")
            .arg(&proc_info2.db_path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let _proc_guard2 = ChildGuard::new(homestar_proc2);

        if wait_for_socket_connection(ws_port2, 1000).is_err() {
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

        let toml3 = format!(
            r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr3}"
        node_addresses = ["{node_addra}"]
        # Allow node two to request workflow info from DHT
        p2p_workflow_info_timeout = 0
        p2p_provider_timeout = 10000
        receipt_quorum = 1
        workflow_quorum = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.pubsub]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port3}
        [node.network.rpc]
        port = {rpc_port3}
        [node.network.webserver]
        port = {ws_port3}
        "#
        );
        let config3 = make_config!(toml3);

        let homestar_proc3 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg(config3.filename())
            .arg("--db")
            .arg(&proc_info3.db_path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let _guard3 = ChildGuard::new(homestar_proc3);

        if wait_for_socket_connection(ws_port3, 1000).is_err() {
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

                if json["connection_established"].is_object() {
                    assert_eq!(
                        json["connection_established"]["peer_id"],
                        SECP256K1MULTIHASH.to_string()
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

                if json["connection_established"].is_object() {
                    assert_eq!(
                        json["connection_established"]["peerId"],
                        ED25519MULTIHASH2.to_string()
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
            .arg(rpc_port1.to_string())
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for put workflow info messages
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                println!("node1: {json}");

                if json["put_workflow_info_dht"].is_object() {
                    assert_eq!(
                        json["put_workflow_info_dht"]["cid"].as_str().unwrap(),
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
            .arg(rpc_port2.to_string())
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for got workflow info messages on node two
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                println!("node2: {json}");

                if json["got_workflow_info_dht"].is_object() {
                    assert_eq!(
                        json["got_workflow_info_dht"]["cid"].as_str().unwrap(),
                        "bafyrmihctgawsskx54qyt3clcaq2quc42pqxzhr73o6qjlc3rc4mhznotq"
                    );

                    break;
                }
            } else {
                panic!("Node two did not get workflow info in time.")
            }
        }

        let db = db::Db::setup_connection_pool(
            &Settings::load().unwrap().node(),
            Some(proc_info1.db_path.display().to_string()),
        )
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
            .arg(rpc_port3.to_string())
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

        // TODO Check for logs that indicate:
        //   - Node three sent workflow info as a provider
        //   - Node one received workflow info from node two provider
        //   - Node one forwarded workflow info to node three
        //   - Node three received the workflow info from node one
    });

    Ok(())
}
