use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, listen_addr, multiaddr, retrieve_output,
        wait_for_socket_connection, ChildGuard, ProcInfo, TimeoutFutureExt, BIN_NAME,
        ED25519MULTIHASH, SECP256K1MULTIHASH,
    },
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
#[serial_test::parallel]
fn test_libp2p_receipt_gossip_integration() -> Result<()> {
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

    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        node_addresses = ["{node_addrb}"]
        [node.network.libp2p.mdns]
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
    let config1 = make_config!(toml);
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
            [node.network.libp2p.mdns]
            enable = false
            [node.network.metrics]
            port = {metrics_port2}
            [node.network.libp2p.rendezvous]
            enable_client = false
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
            .arg(rpc_port1.to_string())
            .arg("-w")
            .arg("tests/fixtures/test-workflow-add-one.json")
            .output();

        // Poll for published receipt messages
        let mut published_cids: Vec<Cid> = vec![];
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["published_receipt_pubsub"].is_object() {
                    published_cids.push(
                        Cid::from_str(json["published_receipt_pubsub"]["cid"].as_str().unwrap())
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

                if json["received_receipt_pubsub"].is_object() {
                    received_cids.push(
                        Cid::from_str(json["received_receipt_pubsub"]["cid"].as_str().unwrap())
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
            vec!["message received on receipts topic", ED25519MULTIHASH],
        );

        assert!(message_published);
        assert!(message_received);

        let config_fixture = config2.filename();
        let settings = Settings::load_from_file(PathBuf::from(config_fixture)).unwrap();
        let db = Db::setup_connection_pool(
            settings.node(),
            Some(proc_info2.db_path.display().to_string()),
        )
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
