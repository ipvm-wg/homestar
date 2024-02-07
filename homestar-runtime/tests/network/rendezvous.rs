use crate::{
    make_config,
    utils::{
        check_for_line_with, count_lines_where, kill_homestar, listen_addr, multiaddr,
        retrieve_output, subscribe_network_events, wait_for_socket_connection,
        wait_for_socket_connection_v6, ChildGuard, ProcInfo, TimeoutFutureExt, BIN_NAME,
        ED25519MULTIHASH, ED25519MULTIHASH2, ED25519MULTIHASH3, ED25519MULTIHASH4,
        ED25519MULTIHASH5, SECP256K1MULTIHASH,
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
fn test_libp2p_connection_rendezvous_discovery_integration() -> Result<()> {
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

    tokio_test::block_on(async {
        // Subscribe to rendezvous server
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

        // Start a peer that will register with the rendezvous server
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

        if wait_for_socket_connection(ws_port2, 1000).is_err() {
            panic!("Homestar server/runtime failed to start in time");
        }

        // Poll for server registered client one
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["peer_registered_rendezvous"].is_object()
                    && json["peer_registered_rendezvous"]["peer_id"] == SECP256K1MULTIHASH
                {
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
            .env("RUST_BACKTRACE", "0")
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

        // Subscribe to rendezvous client two
        let mut net_events3 = subscribe_network_events(ws_port3).await;
        let sub3 = net_events3.sub();

        // Poll for discovery served by rendezvous server
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discover_served_rendezvous"].is_object()
                    && json["discover_served_rendezvous"]["enquirer"] == ED25519MULTIHASH2
                {
                    break;
                }
            } else {
                panic!("Rendezvous server did not serve discovery to client two in time");
            }
        }

        // Kill server and registrant.
        let dead_server = kill_homestar(proc_guard_server.take(), None);
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

        // Kill discoverer.
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

        // Check that client two disconnected from client one.
        let two_disconnected_from_one = check_for_line_with(
            stdout_client2.clone(),
            vec!["peer connection closed", SECP256K1MULTIHASH],
        );

        // Check that client two was removed from the Kademlia table
        let two_removed_from_dht_table = check_for_line_with(
            stdout_client2.clone(),
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

    tokio_test::block_on(async {
        // Subscribe to rendezvous server
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

        // Start a peer that will renew registrations with the rendezvous server once per second
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

        if wait_for_socket_connection(ws_port2, 1000).is_err() {
            panic!("Homestar server/runtime failed to start in time");
        }

        // Poll for server registered client twice.
        let mut peer_registered_count = 0;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["peer_registered_rendezvous"].is_object()
                    && json["peer_registered_rendezvous"]["peer_id"] == ED25519MULTIHASH3
                {
                    peer_registered_count += 1;
                }
            } else {
                panic!("Server did not register client twice in time");
            }

            if peer_registered_count == 2 {
                break;
            }
        }

        // Collect logs for five seconds then kill proceses.
        let dead_server = kill_homestar(rendezvous_server, None);
        let dead_client = kill_homestar(rendezvous_client1, None);

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
    });

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

    tokio_test::block_on(async {
        // Subscribe to rendezvous server
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

        // Start a peer that will discover with the rendezvous server once per second
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

        // Poll for server provided discovery twice twice
        let mut discover_served_count = 0;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discover_served_rendezvous"].is_object()
                    && json["discover_served_rendezvous"]["enquirer"] == ED25519MULTIHASH4
                {
                    discover_served_count += 1;
                }
            } else {
                panic!("Server did not provide discovery twice in time");
            }

            if discover_served_count == 2 {
                break;
            }
        }

        // Collect logs for five seconds then kill proceses.
        let dead_server = kill_homestar(proc_guard_server.take(), None);
        let dead_client = kill_homestar(proc_guard_client1.take(), None);

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
    });

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

    tokio_test::block_on(async {
        // Subscribe to rendezvous server
        let mut net_events1 = subscribe_network_events(ws_port1).await;
        let sub1 = net_events1.sub();

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

        // Poll for server registered client one the first time
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["peer_registered_rendezvous"].is_object()
                    && json["peer_registered_rendezvous"]["peer_id"] == ED25519MULTIHASH5
                {
                    break;
                }
            } else {
                panic!("Server did not receive registration from client one in time");
            }
        }

        // Start a peer that will discover with the rendezvous server when
        // a discovered registration expires. Note that by default discovery only
        // occurs every ten minutes, so discovery requests in this test are driven
        // by client one expirations.
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
            .env("RUST_BACKTRACE", "0")
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

        // Poll for discovery served to client two twice
        let mut discovered_count = 0;
        loop {
            if let Ok(msg) = sub1.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["discover_served_rendezvous"].is_object()
                    && json["discover_served_rendezvous"]["enquirer"] == ED25519MULTIHASH2
                {
                    discovered_count += 1;
                }
            } else {
                panic!("Server did not serve discovery to client two twice in time");
            }

            if discovered_count == 2 {
                break;
            }
        }

        // Collect logs for seven seconds then kill proceses.
        let dead_server = kill_homestar(proc_guard_server.take(), None);
        let _ = kill_homestar(proc_guard_client1.take(), None);
        let dead_client2 = kill_homestar(proc_guard_client2.take(), None);

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
    });

    Ok(())
}
