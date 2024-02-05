use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, listen_addr, multiaddr, retrieve_output,
        wait_for_socket_connection_v6, ChildGuard, ProcInfo, BIN_NAME, ED25519MULTIHASH,
        SECP256K1MULTIHASH,
    },
};
use anyhow::Result;
use libp2p::Multiaddr;
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

#[cfg(all(feature = "websocket-notify", feature = "test-utils"))]
mod dht;
#[cfg(feature = "websocket-notify")]
mod gossip;
mod mdns;
#[cfg(feature = "websocket-notify")]
mod notification;
mod rendezvous;

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[serial_test::parallel]
fn test_libp2p_generates_peer_id_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let listen_addr = listen_addr(proc_info.listen_port);
    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr}"
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected =
        check_for_line_with(stdout, vec!["local peer ID generated", ED25519MULTIHASH]);

    assert!(logs_expected);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_listens_on_address_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let listen_addr = listen_addr(proc_info.listen_port);
    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
        [node.network.libp2p]
        listen_address = "{listen_addr}"
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_for_line_with(
        stdout,
        vec![
            "local node is listening",
            listen_addr.to_string().as_str(),
            SECP256K1MULTIHASH,
        ],
    );

    assert!(logs_expected);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_rpc_listens_on_address_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let listen_addr = listen_addr(proc_info.listen_port);
    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
        [node.network.libp2p]
        listen_address = "{listen_addr}"
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_for_line_with(
        stdout,
        vec!["RPC server listening", &format!("[::1]:{rpc_port}")],
    );

    assert!(logs_expected);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_websocket_listens_on_address_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let listen_addr = listen_addr(proc_info.listen_port);
    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
        [node.network.libp2p]
        listen_address = "{listen_addr}"
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_for_line_with(
        stdout,
        vec!["webserver listening", &format!("127.0.0.1:{ws_port}")],
    );

    assert!(logs_expected);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_connect_known_peers_integration() -> Result<()> {
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
        bootstrap_interval = 1
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

    let config1 = make_config!(toml1);
    // Start two nodes configured to listen at 127.0.0.1 each with their own port.
    // The nodes are configured to dial each other through the node_addresses config.
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

    if wait_for_socket_connection_v6(rpc_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

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

    if wait_for_socket_connection_v6(rpc_port2, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::block_on(async {
        // Check node endpoint to match
        let http_url = format!("http://localhost:{}", ws_port2);
        let http_resp = reqwest::get(format!("{}/node", http_url)).await.unwrap();
        assert_eq!(http_resp.status(), 200);
        let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();
        assert!(http_resp["nodeInfo"]["dynamic"]["connections"]
            .as_object()
            .unwrap()
            .get(ED25519MULTIHASH)
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<Multiaddr>()
            .is_ok());
        let static_info = http_resp["nodeInfo"]["static"].as_object().unwrap();
        let listeners = http_resp["nodeInfo"]["dynamic"]["listeners"]
            .as_array()
            .unwrap();
        assert_eq!(static_info.get("peer_id").unwrap(), SECP256K1MULTIHASH);
        assert_eq!(listeners, &[listen_addr2.to_string()]);
    });

    // Collect logs for five seconds then kill proceses.
    let dead_proc1 = kill_homestar(proc_guard1.take(), Some(Duration::from_secs(5)));
    let dead_proc2 = kill_homestar(proc_guard2.take(), Some(Duration::from_secs(5)));

    // Retrieve logs.
    let stdout1 = retrieve_output(dead_proc1);
    let stdout2 = retrieve_output(dead_proc2);

    // Check that node bootsrapped itself on the 1 second delay.
    let bootstrapped = check_for_line_with(
        stdout1.clone(),
        vec!["successfully bootstrapped node", ED25519MULTIHASH],
    );

    // Check node two was added to the Kademlia table
    let two_added_to_dht = check_for_line_with(
        stdout1.clone(),
        vec![
            "added configured node to kademlia routing table",
            SECP256K1MULTIHASH,
        ],
    );

    // Check that DHT routing table was updated with node two
    let two_in_dht_routing_table = check_for_line_with(
        stdout1.clone(),
        vec![
            "kademlia routing table updated with peer",
            SECP256K1MULTIHASH,
        ],
    );

    // Check that node one connected to node two.
    let one_connected_to_two = check_for_line_with(
        stdout1,
        vec!["peer connection established", SECP256K1MULTIHASH],
    );

    assert!(bootstrapped);
    assert!(one_connected_to_two);
    assert!(two_in_dht_routing_table);
    assert!(two_added_to_dht);

    // Check node one was added to the Kademlia table
    let one_addded_to_dht = check_for_line_with(
        stdout2.clone(),
        vec![
            "added configured node to kademlia routing table",
            ED25519MULTIHASH,
        ],
    );

    // Check that DHT routing table was updated with node one
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

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_disconnect_known_peers_integration() -> Result<()> {
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

    let config1 = make_config!(toml1);
    // Start two nodes configured to listen at 127.0.0.1 each with their own port.
    // The nodes are configured to dial each other through the node_addresses config.
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

    if wait_for_socket_connection_v6(rpc_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

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
        .arg(proc_info2.db_path.clone())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard2 = ChildGuard::new(homestar_proc2);

    if wait_for_socket_connection_v6(rpc_port2, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Kill node two after seven seconds.
    let _ = kill_homestar(proc_guard2.take(), Some(Duration::from_secs(7)));

    // Collect logs for eight seconds then kill node one.
    let dead_proc1 = kill_homestar(proc_guard1.take(), Some(Duration::from_secs(8)));

    // Retrieve logs.
    let stdout = retrieve_output(dead_proc1);

    // Check that node two disconnected from node one.
    let two_disconnected_from_one = check_for_line_with(
        stdout.clone(),
        vec!["peer connection closed", SECP256K1MULTIHASH],
    );

    // Check that node two was not removed from the Kademlia table.
    let two_removed_from_dht_table = check_for_line_with(
        stdout.clone(),
        vec!["removed peer from kademlia table", SECP256K1MULTIHASH],
    );

    assert!(two_disconnected_from_one);
    assert!(!two_removed_from_dht_table);

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_libp2p_configured_with_known_dns_multiaddr() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let listen_addr = listen_addr(proc_info.listen_port);

    let known_peer_id = "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
    // from ipfs bootstrap list
    let dns_node_addr = format!("/dnsaddr/bootstrap.libp2p.io/p2p/{}", known_peer_id);
    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519_2.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr}"
        node_addresses = ["{dns_node_addr}"]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.libp2p.rendezvous]
        enable_client = false
        enable_server = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);

    let multiaddr_not_supported =
        check_for_line_with(stdout.clone(), vec!["MultiaddrNotSupported"]);

    // This can connect to known dns multiaddrs, but won't over GHA.
    // let connected_to_known_peer =
    //     check_for_line_with(stdout, vec!["peer connection established", known_peer_id]);
    // assert!(connected_to_known_peer);

    // Check that we don't receive a MultiaddrNotSupported error.
    assert!(!multiaddr_not_supported);

    Ok(())
}
