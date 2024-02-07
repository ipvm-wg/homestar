use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, listen_addr, multiaddr, retrieve_output,
        subscribe_network_events, wait_for_socket_connection, ChildGuard, ProcInfo,
        TimeoutFutureExt, BIN_NAME, ED25519MULTIHASH, SECP256K1MULTIHASH,
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
#[serial_test::parallel]
fn test_connection_notifications_integration() -> Result<()> {
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
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

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

        // Poll for connection established message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_established"].is_object() {
                    break;
                }
            } else {
                panic!("Node one did not receive a connection established message in time.")
            }
        }

        let dead_proc2 = kill_homestar(proc_guard2.take(), None);

        // Poll for connection closed message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_closed"].is_object() {
                    break;
                }
            } else {
                panic!("Node one did not receive a connection closed message in time.")
            }
        }

        // Kill proceses.
        let dead_proc1 = kill_homestar(proc_guard1.take(), None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check node one added node two to Kademlia table
        let two_added_to_dht = check_for_line_with(
            stdout1.clone(),
            vec![
                "added configured node to kademlia routing table",
                SECP256K1MULTIHASH,
            ],
        );

        // Check node one DHT routing table was updated with node two
        let two_in_dht_routing_table = check_for_line_with(
            stdout1.clone(),
            vec![
                "kademlia routing table updated with peer",
                SECP256K1MULTIHASH,
            ],
        );

        // Check that node one connected to node two.
        let one_connected_to_two = check_for_line_with(
            stdout1.clone(),
            vec!["peer connection established", SECP256K1MULTIHASH],
        );

        // Check that node two disconnected from node one.
        let two_disconnected_from_one = check_for_line_with(
            stdout1.clone(),
            vec!["peer connection closed", SECP256K1MULTIHASH],
        );

        // Check that node two was not removed from the Kademlia table.
        let two_removed_from_dht_table = check_for_line_with(
            stdout1.clone(),
            vec!["removed peer from kademlia table", SECP256K1MULTIHASH],
        );

        assert!(one_connected_to_two);
        assert!(two_in_dht_routing_table);
        assert!(two_added_to_dht);
        assert!(two_disconnected_from_one);
        assert!(!two_removed_from_dht_table);

        // Check node two added node one to Kademlia table
        let one_addded_to_dht = check_for_line_with(
            stdout2.clone(),
            vec![
                "added configured node to kademlia routing table",
                ED25519MULTIHASH,
            ],
        );

        // Check node two DHT routing table was updated with node one
        let one_in_dht_routing_table = check_for_line_with(
            stdout2.clone(),
            vec!["kademlia routing table updated with peer", ED25519MULTIHASH],
        );

        // Check that node two connected to node one.
        let two_connected_to_one = check_for_line_with(
            stdout2,
            vec!["peer connection established", ED25519MULTIHASH],
        );

        assert!(one_addded_to_dht);
        assert!(one_in_dht_routing_table);
        assert!(two_connected_to_one);
    });

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_redial_on_connection_closed_integration() -> Result<()> {
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
    let node_addrb = multiaddr(proc_info2.listen_port, SECP256K1MULTIHASH);

    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        node_addresses = ["{node_addrb}"]
        dial_interval = 3
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

    let toml2 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        node_addresses = []
        [node.network.libp2p.mdns]
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

    let config1 = make_config!(toml1);
    let config2 = make_config!(toml2);

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

    tokio_test::block_on(async {
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

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

        kill_homestar(proc_guard2.take(), None);

        // Poll for connection closed message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_closed"].is_object() {
                    break;
                }
            } else {
                panic!("Connection between node one and node two did not close in time.")
            }
        }

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

        // Poll for connection established message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_established"].is_object() {
                    break;
                }
            } else {
                panic!("Node one did not redial node two in time.")
            }
        }
    });

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_redial_on_connection_error_integration() -> Result<()> {
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
    let node_addrb = multiaddr(proc_info2.listen_port, SECP256K1MULTIHASH);

    let toml1 = format!(
        r#"
            [node]
            [node.network.keypair_config]
            existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
            [node.network.libp2p]
            listen_address = "{listen_addr1}"
            node_addresses = ["{node_addrb}"]
            dial_interval = 3
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

    let toml2 = format!(
        r#"
            [node]
            [node.network.keypair_config]
            existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
            [node.network.libp2p]
            listen_address = "{listen_addr2}"
            node_addresses = []
            [node.network.libp2p.mdns]
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

    let config1 = make_config!(toml1);
    let config2 = make_config!(toml2);

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

    tokio_test::block_on(async {
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

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

        kill_homestar(proc_guard2.take(), None);

        // Poll for connection closed message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_closed"].is_object() {
                    break;
                }
            } else {
                panic!("Connection between node one and node two did not close in time.")
            }
        }

        // Poll for outgoing connection error message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["outgoing_connection_error"].is_object() {
                    break;
                }
            } else {
                panic!("Connection between node one and node two did not close in time.")
            }
        }

        // Poll for outgoing connection error message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["outgoing_connection_error"].is_object() {
                    break;
                }
            } else {
                panic!("Connection between node one and node two did not close in time.")
            }
        }

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

        // Poll for connection established message
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_established"].is_object() {
                    break;
                }
            } else {
                panic!("Node one did not redial node two in time.")
            }
        }
    });

    Ok(())
}
