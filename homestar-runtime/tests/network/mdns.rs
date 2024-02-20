use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, retrieve_output, subscribe_network_events,
        wait_for_socket_connection, wait_for_socket_connection_v6, ChildGuard, ProcInfo,
        TimeoutFutureExt, BIN_NAME, ED25519MULTIHASH2, ED25519MULTIHASH5,
    },
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[serial_test::serial]
fn test_libp2p_connection_after_mdns_discovery_serial() -> Result<()> {
    let proc_info1 = ProcInfo::new().unwrap();
    let proc_info2 = ProcInfo::new().unwrap();

    let rpc_port1 = proc_info1.rpc_port;
    let rpc_port2 = proc_info2.rpc_port;
    let metrics_port1 = proc_info1.metrics_port;
    let metrics_port2 = proc_info2.metrics_port;
    let ws_port1 = proc_info1.ws_port;
    let ws_port2 = proc_info2.ws_port;

    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        listen_address = "/ip4/0.0.0.0/tcp/0"
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

    // Start a node configured to listen at 0.0.0.0 with no known peers.
    // The node is configured with port 0 to allow the OS to select a port.
    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env("RUST_BACKTRACE", "0")
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

    if wait_for_socket_connection_v6(rpc_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let toml2 = format!(
        r#"
            [node]
            [node.network.keypair_config]
            existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_5.pem" }}
            [node.network.libp2p]
            listen_address = "/ip4/0.0.0.0/tcp/0"
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

    tokio_test::block_on(async {
        // Subscribe to node one
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

        // Start a second node configured to listen at 0.0.0.0 with no known peers.
        // The node is configured with port 0 to allow the OS to select a port.
        let homestar_proc2 = Command::new(BIN.as_os_str())
            .env("RUST_BACKTRACE", "0")
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

        // Poll for mDNS discovered message and connection established messages on node one.
        let mut discovered_mdns = false;
        let mut connection_established = false;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discovered_mdns"].is_object() {
                    discovered_mdns = true;
                } else if json["connection_established"].is_object()
                    && json["connection_established"]["peer_id"] == ED25519MULTIHASH5
                {
                    connection_established = true;
                }
            } else {
                panic!(
                    r#"Expected notifications from node one did not arrive in time:
  - mDNS discovered: {}
  - Connection established: {}
  "#,
                    discovered_mdns, connection_established
                );
            }

            if connection_established && discovered_mdns {
                break;
            }
        }

        // Kill node two.
        let dead_proc2 = kill_homestar(proc_guard2.take(), None);

        // Poll for client two disconnected from client one.
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_closed"].is_object()
                    && json["connection_closed"]["peer_id"] == ED25519MULTIHASH5
                {
                    break;
                }
            } else {
                panic!("Client two did not receive rendezvous discovery from server in time");
            }
        }

        // Kill node one.
        let dead_proc1 = kill_homestar(proc_guard1.take(), None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check that node one connected to node two.
        let one_connected_to_two = check_for_line_with(
            stdout1.clone(),
            vec!["peer connection established", ED25519MULTIHASH5],
        );

        // Check node two was added to the Kademlia table
        let two_addded_to_dht = check_for_line_with(
            stdout1.clone(),
            vec![
                "added identified node to kademlia routing table",
                ED25519MULTIHASH5,
            ],
        );

        // Check that DHT routing table was updated with node two
        let two_in_dht_routing_table = check_for_line_with(
            stdout1.clone(),
            vec![
                "kademlia routing table updated with peer",
                ED25519MULTIHASH5,
            ],
        );

        assert!(one_connected_to_two);
        assert!(two_addded_to_dht);
        assert!(two_in_dht_routing_table);

        // Check that node two connected to node one.
        let two_connected_to_one = check_for_line_with(
            stdout2.clone(),
            vec!["peer connection established", ED25519MULTIHASH2],
        );

        // Check node one was added to the Kademlia table
        let one_addded_to_dht = check_for_line_with(
            stdout2.clone(),
            vec![
                "added identified node to kademlia routing table",
                ED25519MULTIHASH2,
            ],
        );

        // Check that DHT routing table was updated with node one
        let one_in_dht_routing_table = check_for_line_with(
            stdout2,
            vec![
                "kademlia routing table updated with peer",
                ED25519MULTIHASH2,
            ],
        );

        assert!(two_connected_to_one);
        assert!(one_addded_to_dht);
        assert!(one_in_dht_routing_table);

        // Check that node two disconnected from node one.
        let two_disconnected_from_one = check_for_line_with(
            stdout1.clone(),
            vec!["peer connection closed", ED25519MULTIHASH5],
        );

        // Check that node two was removed from the Kademlia table
        let two_removed_from_dht_table = check_for_line_with(
            stdout1.clone(),
            vec!["removed peer from kademlia table", ED25519MULTIHASH5],
        );

        assert!(two_disconnected_from_one);
        assert!(two_removed_from_dht_table);
    });

    Ok(())
}
