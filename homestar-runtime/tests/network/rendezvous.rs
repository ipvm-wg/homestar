use crate::{
    make_config,
    utils::{
        check_for_line_with, count_lines_where, kill_homestar, listen_addr, multiaddr,
        retrieve_output, wait_for_socket_connection, wait_for_socket_connection_v6, ChildGuard,
        ProcInfo, TimeoutFutureExt, BIN_NAME, ED25519MULTIHASH, ED25519MULTIHASH2,
        ED25519MULTIHASH3, ED25519MULTIHASH4, ED25519MULTIHASH5, SECP256K1MULTIHASH,
    },
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
    thread,
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

#[test]
#[serial_test::parallel]
fn test_libp2p_connect_rendezvous_discovery_integration() -> Result<()> {
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
    let announce_addrb = multiaddr(proc_info2.listen_port, SECP256K1MULTIHASH);
    let node_addra = multiaddr(proc_info1.listen_port, ED25519MULTIHASH);

    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        [node.network.libp2p.rendezvous]
        enable_server = true
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
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
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let toml2 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        announce_addresses = ["{announce_addrb}"]
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
    );
    let config2 = make_config!(toml2);

    // Start a peer that will register with the rendezvous server
    let rendezvous_client1 = Command::new(BIN.as_os_str())
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
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection(ws_port2, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::task::spawn(async {
        // Subscribe to rendezvous server
        let ws_url1 = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port1);
        let client1 = WsClientBuilder::default().build(ws_url1).await.unwrap();
        let mut sub1: Subscription<Vec<u8>> = client1
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        // Subscribe to rendezvous client one
        let ws_url2 = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port2);
        let client2 = WsClientBuilder::default().build(ws_url2).await.unwrap();
        let mut sub2: Subscription<Vec<u8>> = client2
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        // Poll for client one registered with server
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["registered_rendezvous"].is_object() {
                    break;
                }
            } else {
                panic!("Rendezvous client one did not register with server in time");
            }
        }

        // Poll for server registered client one
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["peer_registered_rendezvous"].is_object() {
                    break;
                }
            } else {
                panic!("Rendezvous server did not confirm client one registration in time");
            }
        }

        // Start a peer that will discover the registrant through the rendezvous server
        let toml3 = format!(
            r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr3}"
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port3}
        [node.network.rpc]
        port = {rpc_port3}
        [node.network.webserver]
        port = {ws_port3}
        "#
        );
        let config3 = make_config!(toml3);

        let rendezvous_client2 = Command::new(BIN.as_os_str())
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
        let proc_guard_client2 = ChildGuard::new(rendezvous_client2);

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

        // Poll for discovered rendezvous message
        let mut discovered_rendezvous = false;
        let mut connection_established = false;
        loop {
            if let Ok(msg) = sub3.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discovered_rendezvous"].is_object() {
                    discovered_rendezvous = true
                } else if json["connection_established"].is_object()
                    && json["connection_established"]["peer_id"] == SECP256K1MULTIHASH
                {
                    connection_established = true
                }

                if discovered_rendezvous && connection_established {
                    break;
                }
            } else {
                panic!("Client two did not receive rendezvous discovery from server in time");
            }
        }

        // Poll for discovery served by rendezvous server
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discover_served_rendezvous"].is_object() {
                    break;
                }
            } else {
                panic!("Rendezvous server did not serve discovery in time");
            }
        }

        // Kill processes.
        let dead_server = kill_homestar(proc_guard_server.take(), None);
        let _ = kill_homestar(proc_guard_client1.take(), None);
        let dead_client2 = kill_homestar(proc_guard_client2.take(), None);

        // Retrieve logs.
        let stdout_server = retrieve_output(dead_server);
        let stdout_client2 = retrieve_output(dead_client2);

        // Check rendezvous server registered the client one
        let registered_client_one = check_for_line_with(
            stdout_server.clone(),
            vec!["registered peer through rendezvous", SECP256K1MULTIHASH],
        );

        // Check rendezvous served a discover request to client two
        let served_discovery_to_client_two = check_for_line_with(
            stdout_server.clone(),
            vec![
                "served rendezvous discover request to peer",
                ED25519MULTIHASH2,
            ],
        );

        assert!(registered_client_one);
        assert!(served_discovery_to_client_two);

        // Check that client two connected to client one.
        let two_connected_to_one = check_for_line_with(
            stdout_client2.clone(),
            vec!["peer connection established", SECP256K1MULTIHASH],
        );

        // Check client one was added to the Kademlia table
        let one_addded_to_dht = check_for_line_with(
            stdout_client2.clone(),
            vec![
                "added identified node to kademlia routing table",
                SECP256K1MULTIHASH,
            ],
        );

        // Check that DHT routing table was updated with client one
        let one_in_dht_routing_table = check_for_line_with(
            stdout_client2.clone(),
            vec![
                "kademlia routing table updated with peer",
                SECP256K1MULTIHASH,
            ],
        );

        assert!(one_addded_to_dht);
        assert!(one_in_dht_routing_table);
        assert!(two_connected_to_one);
    });

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_disconnect_rendezvous_discovery_integration() -> Result<()> {
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
    let announce_addrb = multiaddr(proc_info2.listen_port, SECP256K1MULTIHASH);
    let node_addra = multiaddr(proc_info1.listen_port, ED25519MULTIHASH);

    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        [node.network.libp2p.rendezvous]
        enable_server = true
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
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
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let toml2 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        announce_addresses = ["{announce_addrb}"]
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
    );
    let config2 = make_config!(toml2);

    // Start a peer that will register with the rendezvous server
    let rendezvous_client1 = Command::new(BIN.as_os_str())
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
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection(ws_port2, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::task::spawn(async {
        // Subscribe to rendezvous server
        let ws_url1 = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port1);
        let client1 = WsClientBuilder::default().build(ws_url1).await.unwrap();
        let mut sub1: Subscription<Vec<u8>> = client1
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        // Subscribe to rendezvous client one
        let ws_url2 = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port2);
        let client2 = WsClientBuilder::default().build(ws_url2).await.unwrap();
        let mut sub2: Subscription<Vec<u8>> = client2
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        // Wait for registration to complete.
        // TODO When we have WebSocket push events, listen on a registration event instead of using an arbitrary sleep.
        // thread::sleep(Duration::from_secs(2));

        // TODO Wait for clint 1 to register with server, server confirm registration

        // Poll for client one registered with server
        loop {
            if let Ok(msg) = sub2.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["registered_rendezvous"].is_object() {
                    break;
                }
            } else {
                panic!("Rendezvous client one did not register with server in time");
            }
        }

        // Poll for server registered client one
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["peer_registered_rendezvous"].is_object() {
                    break;
                }
            } else {
                panic!("Rendezvous server did not confirm client one registration in time");
            }
        }

        let toml3 = format!(
            r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr3}"
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port3}
        [node.network.rpc]
        port = {rpc_port3}
        [node.network.webserver]
        port = {ws_port3}
        "#
        );
        let config3 = make_config!(toml3);

        // Start a peer that will discover the registrant through the rendezvous server
        let rendezvous_client2 = Command::new(BIN.as_os_str())
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
        let proc_guard_client2 = ChildGuard::new(rendezvous_client2);

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

        // Poll for discovered rendezvous message
        let mut discovered_rendezvous = false;
        let mut connection_established = false;
        loop {
            if let Ok(msg) = sub3.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discovered_rendezvous"].is_object() {
                    discovered_rendezvous = true
                } else if json["connection_established"].is_object()
                    && json["connection_established"]["peer_id"] == SECP256K1MULTIHASH
                {
                    connection_established = true
                }

                if discovered_rendezvous && connection_established {
                    break;
                }
            } else {
                panic!("Client two did not receive rendezvous discovery from server in time");
            }
        }

        // Poll for discovery served by rendezvous server
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discover_served_rendezvous"].is_object() {
                    break;
                }
            } else {
                panic!("Rendezvous server did not serve discovery in time");
            }
        }

        // Kill server and client one.
        let _ = kill_homestar(proc_guard_server.take(), None);
        let _ = kill_homestar(proc_guard_client1.take(), None);

        // Poll for client two disconnected from client one.
        loop {
            if let Ok(msg) = sub3.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["connection_closed"].is_object()
                    && json["connection_closed"]["peer_id"] == SECP256K1MULTIHASH
                {
                    break;
                }
            } else {
                panic!("Client two did not receive rendezvous discovery from server in time");
            }
        }

        // Kill client two.
        let dead_client2 = kill_homestar(proc_guard_client2.take(), None);

        // Retrieve logs.
        let stdout = retrieve_output(dead_client2);

        // Check that client two disconnected from client one.
        let two_disconnected_from_one = check_for_line_with(
            stdout.clone(),
            vec!["peer connection closed", SECP256K1MULTIHASH],
        );

        // Check that client two was removed from the Kademlia table
        let two_removed_from_dht_table = check_for_line_with(
            stdout.clone(),
            vec!["removed peer from kademlia table", SECP256K1MULTIHASH],
        );

        assert!(two_disconnected_from_one);
        assert!(two_removed_from_dht_table);
    });

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_rendezvous_renew_registration_integration() -> Result<()> {
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
    let announce_addrb = multiaddr(proc_info2.listen_port, ED25519MULTIHASH3);
    let node_addra = multiaddr(proc_info1.listen_port, ED25519MULTIHASH);

    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        [node.network.libp2p.rendezvous]
        enable_server = true
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
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

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let toml2 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_3.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        announce_addresses = ["{announce_addrb}"]
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.rendezvous]
        registration_ttl = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
    );
    let config2 = make_config!(toml2);

    // Start a peer that will renew registrations with the rendezvous server once per second
    let rendezvous_client1 = Command::new(BIN.as_os_str())
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

    if wait_for_socket_connection(ws_port2, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // TODO Listen for client registered and server registered peer messages
    // with renewal should be more than one.

    // Collect logs for five seconds then kill proceses.
    let dead_server = kill_homestar(rendezvous_server, Some(Duration::from_secs(5)));
    let dead_client = kill_homestar(rendezvous_client1, Some(Duration::from_secs(5)));

    // Retrieve logs.
    let stdout_server = retrieve_output(dead_server);
    let stdout_client = retrieve_output(dead_client);

    // Count registrations on the server
    let server_registration_count = count_lines_where(
        stdout_server,
        vec![
            "registered peer through rendezvous",
            "12D3KooWJWoaqZhDaoEFshF7Rh1bpY9ohihFhzcW6d69Lr2NASuq",
        ],
    );

    // Count registrations on the client
    let client_registration_count = count_lines_where(
        stdout_client,
        vec![
            "registered self with rendezvous node",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(server_registration_count > 1);
    assert!(client_registration_count > 1);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_rendezvous_rediscovery_integration() -> Result<()> {
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

    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        [node.network.libp2p.rendezvous]
        enable_server = true
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
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
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let toml2 = format!(
        r#"
        [node]
        [node.network]
        poll_cache_interval = 100
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_4.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.rendezvous]
        discovery_interval = 1
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
    );
    let config2 = make_config!(toml2);

    // Start a peer that will discover with the rendezvous server once per second
    let rendezvous_client1 = Command::new(BIN.as_os_str())
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
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection_v6(rpc_port2, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // TODO Listen for client discover and server discover served messages
    // should be more than one for both (or move on at two)

    // Collect logs for five seconds then kill proceses.
    let dead_server = kill_homestar(proc_guard_server.take(), Some(Duration::from_secs(15)));
    let dead_client = kill_homestar(proc_guard_client1.take(), Some(Duration::from_secs(15)));

    // Retrieve logs.
    let stdout_server = retrieve_output(dead_server);
    let stdout_client = retrieve_output(dead_client);

    // Count discover requests on the server
    let server_discovery_count = count_lines_where(
        stdout_server,
        vec![
            "served rendezvous discover request to peer",
            ED25519MULTIHASH4,
        ],
    );

    // Count discovery responses the client
    let client_discovery_count = count_lines_where(
        stdout_client,
        vec![
            "received discovery from rendezvous server",
            ED25519MULTIHASH,
        ],
    );

    assert!(server_discovery_count > 1);
    assert!(client_discovery_count > 1);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_rendezvous_rediscover_on_expiration_integration() -> Result<()> {
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
    let announce_addrb = multiaddr(proc_info2.listen_port, ED25519MULTIHASH5);
    let node_addra = multiaddr(proc_info1.listen_port, ED25519MULTIHASH);

    let toml1 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        [node.network.libp2p.rendezvous]
        enable_server = true
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port1}
        [node.network.rpc]
        port = {rpc_port1}
        [node.network.webserver]
        port = {ws_port1}
        "#
    );
    let config1 = make_config!(toml1);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
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
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let toml2 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_5.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr2}"
        announce_addresses = ["{announce_addrb}"]
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.rendezvous]
        registration_ttl = 5
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port2}
        [node.network.rpc]
        port = {rpc_port2}
        [node.network.webserver]
        port = {ws_port2}
        "#
    );
    let config2 = make_config!(toml2);

    // Start a peer that will renew registrations with the rendezvous server every five seconds
    let rendezvous_client1 = Command::new(BIN.as_os_str())
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
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection_v6(rpc_port2, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Wait for registration to complete.
    // TODO When we have WebSocket push events, listen on a registration event instead of using an arbitrary sleep.
    thread::sleep(Duration::from_secs(2));

    // Start a peer that will discover with the rendezvous server when
    // a discovered registration expires. Note that by default discovery only
    // occurs every ten minutes, so discovery requests in this test are driven
    // by expirations.
    let toml3 = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr3}"
        node_addresses = ["{node_addra}"]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port3}
        [node.network.rpc]
        port = {rpc_port3}
        [node.network.webserver]
        port = {ws_port3}
        "#
    );
    let config3 = make_config!(toml3);

    let rendezvous_client2 = Command::new(BIN.as_os_str())
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
    let proc_guard_client2 = ChildGuard::new(rendezvous_client2);

    if wait_for_socket_connection(ws_port3, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for seven seconds then kill proceses.
    let dead_server = kill_homestar(proc_guard_server.take(), Some(Duration::from_secs(15)));
    let _ = kill_homestar(proc_guard_client1.take(), Some(Duration::from_secs(7)));
    let dead_client2 = kill_homestar(proc_guard_client2.take(), Some(Duration::from_secs(15)));

    // Retrieve logs.
    let stdout_server = retrieve_output(dead_server);
    let stdout_client2 = retrieve_output(dead_client2);

    // Count discover requests on the server
    let server_discovery_count = count_lines_where(
        stdout_server,
        vec![
            "served rendezvous discover request to peer",
            "12D3KooWK99VoVxNE7XzyBwXEzW7xhK7Gpv85r9F3V3fyKSUKPH5",
        ],
    );

    println!("server_discovery_count: {}", server_discovery_count);

    // Count discovery responses the client
    let client_discovery_count = count_lines_where(
        stdout_client2,
        vec![
            "received discovery from rendezvous server",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(server_discovery_count > 1);
    assert!(client_discovery_count > 1);

    Ok(())
}
