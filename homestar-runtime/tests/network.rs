use crate::utils::{
    check_for_line_with, count_lines_where, kill_homestar, retrieve_output,
    wait_for_socket_connection, wait_for_socket_connection_v6, ChildGuard, FileGuard, BIN_NAME,
};
use anyhow::Result;
use libp2p::Multiaddr;
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

#[cfg(all(feature = "websocket-notify", feature = "test-utils"))]
mod dht;
#[cfg(feature = "websocket-notify")]
mod gossip;
#[cfg(feature = "websocket-notify")]
mod notification;

#[allow(dead_code)]
static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
fn test_libp2p_generates_peer_id_integration() -> Result<()> {
    const DB: &str = "test_libp2p_generates_peer_id_integration.db";
    let _db_guard = FileGuard::new(DB);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(9820, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_for_line_with(
        stdout,
        vec![
            "local peer ID generated",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(logs_expected);

    Ok(())
}

#[test]
fn test_libp2p_listens_on_address_integration() -> Result<()> {
    const DB: &str = "test_libp2p_listens_on_address_integration.db";
    let _db_guard = FileGuard::new(DB);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(9820, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_for_line_with(
        stdout,
        vec![
            "local node is listening",
            "/ip4/127.0.0.1/tcp/7000",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(logs_expected);

    Ok(())
}

#[test]
fn test_rpc_listens_on_address_integration() -> Result<()> {
    const DB: &str = "test_rpc_listens_on_address_integration.db";
    let _db_guard = FileGuard::new(DB);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(9820, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_for_line_with(stdout, vec!["RPC server listening", "[::1]:9820"]);

    assert!(logs_expected);

    Ok(())
}

#[test]
fn test_websocket_listens_on_address_integration() -> Result<()> {
    const DB: &str = "test_websocket_listens_on_address_integration.db";
    let _db_guard = FileGuard::new(DB);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(9820, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(proc_guard.take(), None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_for_line_with(stdout, vec!["webserver listening", "127.0.0.1:8020"]);

    assert!(logs_expected);

    Ok(())
}

#[test]
fn test_libp2p_connect_known_peers_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_connect_known_peers_integration1.db";
    const DB2: &str = "test_libp2p_connect_known_peers_integration2.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);

    // Start two nodes configured to listen at 127.0.0.1 each with their own port.
    // The nodes are configured to dial each other through the node_addresses config.
    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection_v6(9820, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network2.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard2 = ChildGuard::new(homestar_proc2);

    if wait_for_socket_connection_v6(9821, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::block_on(async {
        // Check node endpoint to match
        let http_url = format!("http://localhost:{}", 8020);
        let http_resp = reqwest::get(format!("{}/node", http_url)).await.unwrap();
        assert_eq!(http_resp.status(), 200);
        let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();
        assert!(http_resp["nodeInfo"]["dynamic"]["connections"]
            .as_object()
            .unwrap()
            .get("16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<Multiaddr>()
            .is_ok());
        let static_info = http_resp["nodeInfo"]["static"].as_object().unwrap();
        let listeners = http_resp["nodeInfo"]["dynamic"]["listeners"]
            .as_array()
            .unwrap();
        assert_eq!(
            static_info.get("peer_id").unwrap(),
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
        );
        assert_eq!(listeners, &["/ip4/127.0.0.1/tcp/7000"]);
    });

    // Collect logs for five seconds then kill proceses.
    let dead_proc1 = kill_homestar(proc_guard1.take(), Some(Duration::from_secs(5)));
    let dead_proc2 = kill_homestar(proc_guard2.take(), Some(Duration::from_secs(5)));

    // Retrieve logs.
    let stdout1 = retrieve_output(dead_proc1);
    let stdout2 = retrieve_output(dead_proc2);

    // Check node two was added to the Kademlia table
    let two_addded_to_dht = check_for_line_with(
        stdout1.clone(),
        vec![
            "added configured node to kademlia routing table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that DHT routing table was updated with node two
    let two_in_dht_routing_table = check_for_line_with(
        stdout1.clone(),
        vec![
            "kademlia routing table updated with peer",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that node one connected to node two.
    let one_connected_to_two = check_for_line_with(
        stdout1,
        vec![
            "peer connection established",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(one_connected_to_two);
    assert!(two_in_dht_routing_table);
    assert!(two_addded_to_dht);

    // Check node one was added to the Kademlia table
    let one_addded_to_dht = check_for_line_with(
        stdout2.clone(),
        vec![
            "added configured node to kademlia routing table",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check that DHT routing table was updated with node one
    let one_in_dht_routing_table = check_for_line_with(
        stdout2.clone(),
        vec![
            "kademlia routing table updated with peer",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check that node two connected to node one.
    let two_connected_to_one = check_for_line_with(
        stdout2,
        vec![
            "peer connection established",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(one_addded_to_dht);
    assert!(one_in_dht_routing_table);
    assert!(two_connected_to_one);

    Ok(())
}

#[test]
fn test_libp2p_connect_after_mdns_discovery_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_connect_after_mdns_discovery_integration1.db";
    const DB2: &str = "test_libp2p_connect_after_mdns_discovery_integration2.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);

    // Start two nodes each configured to listen at 0.0.0.0 with no known peers.
    // The nodes are configured with port 0 to allow the OS to select a port.
    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_mdns1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection_v6(9800, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_mdns2.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard2 = ChildGuard::new(homestar_proc2);

    if wait_for_socket_connection_v6(9801, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for seven seconds then kill processes.
    let dead_proc1 = kill_homestar(proc_guard1.take(), Some(Duration::from_secs(7)));
    let dead_proc2 = kill_homestar(proc_guard2.take(), Some(Duration::from_secs(7)));

    // Retrieve logs.
    let stdout1 = retrieve_output(dead_proc1);
    let stdout2 = retrieve_output(dead_proc2);

    // Check that node one connected to node two.
    let one_connected_to_two = check_for_line_with(
        stdout1.clone(),
        vec![
            "peer connection established",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check node two was added to the Kademlia table
    let two_addded_to_dht = check_for_line_with(
        stdout1.clone(),
        vec![
            "added identified node to kademlia routing table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that DHT routing table was updated with node two
    let two_in_dht_routing_table = check_for_line_with(
        stdout1,
        vec![
            "kademlia routing table updated with peer",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(one_connected_to_two);
    assert!(two_addded_to_dht);
    assert!(two_in_dht_routing_table);

    // Check that node two connected to node one.
    let two_connected_to_one = check_for_line_with(
        stdout2.clone(),
        vec![
            "peer connection established",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check node one was added to the Kademlia table
    let one_addded_to_dht = check_for_line_with(
        stdout2.clone(),
        vec![
            "added identified node to kademlia routing table",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check that DHT routing table was updated with node one
    let one_in_dht_routing_table = check_for_line_with(
        stdout2,
        vec![
            "kademlia routing table updated with peer",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(two_connected_to_one);
    assert!(one_addded_to_dht);
    assert!(one_in_dht_routing_table);

    Ok(())
}

#[test]
fn test_libp2p_connect_rendezvous_discovery_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_connect_rendezvous_discovery_integration1.db";
    const DB2: &str = "test_libp2p_connect_rendezvous_discovery_integration2.db";
    const DB3: &str = "test_libp2p_connect_rendezvous_discovery_integration3.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);
    let _db_guard3 = FileGuard::new(DB3);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(8024, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Start a peer that will register with the rendezvous server
    let rendezvous_client1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous2.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection(8026, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Wait for registration to complete
    // TODO When we have WebSocket push events, listen on a registration event instead of using an arbitrary sleep
    thread::sleep(Duration::from_secs(2));

    // Start a peer that will discover the registrant through the rendezvous server
    let rendezvous_client2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous3.toml")
        .arg("--db")
        .arg(DB3)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_client2 = ChildGuard::new(rendezvous_client2);

    if wait_for_socket_connection(8027, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for five seconds then kill proceses.
    let dead_server = kill_homestar(proc_guard_server.take(), Some(Duration::from_secs(5)));
    let _ = kill_homestar(proc_guard_client1.take(), Some(Duration::from_secs(5)));
    let dead_client2 = kill_homestar(proc_guard_client2.take(), Some(Duration::from_secs(5)));

    // Retrieve logs.
    let stdout_server = retrieve_output(dead_server);
    let stdout_client2 = retrieve_output(dead_client2);

    // Check rendezvous server registered the client one
    let registered_client_one = check_for_line_with(
        stdout_server.clone(),
        vec![
            "registered peer through rendezvous",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check rendezvous served a discover request to client two
    let served_discovery_to_client_two = check_for_line_with(
        stdout_server.clone(),
        vec![
            "served rendezvous discover request to peer",
            "12D3KooWK99VoVxNE7XzyBwXEzW7xhK7Gpv85r9F3V3fyKSUKPH5",
        ],
    );

    assert!(registered_client_one);
    assert!(served_discovery_to_client_two);

    // Check that client two connected to client one.
    let two_connected_to_one = check_for_line_with(
        stdout_client2.clone(),
        vec![
            "peer connection established",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check client one was added to the Kademlia table
    let one_addded_to_dht = check_for_line_with(
        stdout_client2.clone(),
        vec![
            "added identified node to kademlia routing table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that DHT routing table was updated with client one
    let one_in_dht_routing_table = check_for_line_with(
        stdout_client2.clone(),
        vec![
            "kademlia routing table updated with peer",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(one_addded_to_dht);
    assert!(one_in_dht_routing_table);
    assert!(two_connected_to_one);

    Ok(())
}

#[test]
fn test_libp2p_disconnect_mdns_discovery_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_disconnect_mdns_discovery_integration1.db";
    const DB2: &str = "test_libp2p_disconnect_mdns_discovery_integration2.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);

    // Start two nodes each configured to listen at 0.0.0.0 with no known peers.
    // The nodes are configured with port 0 to allow the OS to select a port.
    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_mdns1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection(8000, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_mdns2.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard2 = ChildGuard::new(homestar_proc2);

    if wait_for_socket_connection(8001, 100).is_err() {
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
        vec![
            "peer connection closed",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that node two was removed from the Kademlia table
    let two_removed_from_dht_table = check_for_line_with(
        stdout.clone(),
        vec![
            "removed peer from kademlia table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(two_disconnected_from_one);
    assert!(two_removed_from_dht_table);

    Ok(())
}

#[test]
fn test_libp2p_disconnect_known_peers_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_disconnect_known_peers_integration1.db";
    const DB2: &str = "test_libp2p_disconnect_known_peers_integration2.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);

    // Start two nodes configured to listen at 127.0.0.1 each with their own port.
    // The nodes are configured to dial each other through the node_addresses config.
    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection_v6(9820, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network2.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard2 = ChildGuard::new(homestar_proc2);

    if wait_for_socket_connection_v6(9821, 100).is_err() {
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
        vec![
            "peer connection closed",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that node two was not removed from the Kademlia table.
    let two_removed_from_dht_table = check_for_line_with(
        stdout.clone(),
        vec![
            "removed peer from kademlia table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(two_disconnected_from_one);
    assert!(!two_removed_from_dht_table);

    Ok(())
}

#[test]
fn test_libp2p_disconnect_rendezvous_discovery_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_disconnect_rendezvous_discovery_integration1.db";
    const DB2: &str = "test_libp2p_disconnect_rendezvous_discovery_integration2.db";
    const DB3: &str = "test_libp2p_disconnect_rendezvous_discovery_integration3.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);
    let _db_guard3 = FileGuard::new(DB3);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(8024, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Start a peer that will register with the rendezvous server
    let rendezvous_client1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous2.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection(8026, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Wait for registration to complete.
    // TODO When we have WebSocket push events, listen on a registration event instead of using an arbitrary sleep.
    thread::sleep(Duration::from_secs(2));

    // Start a peer that will discover the registrant through the rendezvous server
    let rendezvous_client2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous3.toml")
        .arg("--db")
        .arg(DB3)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_client2 = ChildGuard::new(rendezvous_client2);

    if wait_for_socket_connection(8027, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Kill server and client one after five seconds
    let _ = kill_homestar(proc_guard_server.take(), Some(Duration::from_secs(5)));
    let _ = kill_homestar(proc_guard_client1.take(), Some(Duration::from_secs(5)));

    // Collect logs for seven seconds then kill process.
    let dead_client2 = kill_homestar(proc_guard_client2.take(), Some(Duration::from_secs(7)));

    // Retrieve logs.
    let stdout = retrieve_output(dead_client2);

    // Check that client two disconnected from client one.
    let two_disconnected_from_one = check_for_line_with(
        stdout.clone(),
        vec![
            "peer connection closed",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that client two was removed from the Kademlia table
    let two_removed_from_dht_table = check_for_line_with(
        stdout.clone(),
        vec![
            "removed peer from kademlia table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(two_disconnected_from_one);
    assert!(two_removed_from_dht_table);

    Ok(())
}

#[test]
fn test_libp2p_rendezvous_renew_registration_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_rendezvous_renew_registration_integration1.db";
    const DB2: &str = "test_libp2p_rendezvous_renew_registration_integration2.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection(8024, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Start a peer that will renew registrations with the rendezvous server once per second
    let rendezvous_client1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous4.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection(8028, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

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
fn test_libp2p_rendezvous_rediscovery_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_rendezvous_rediscovery_integration1.db";
    const DB2: &str = "test_libp2p_rendezvous_rediscovery_integration2.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(8024, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Start a peer that will discover with the rendezvous server once per second
    let rendezvous_client1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous5.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection_v6(9829, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for five seconds then kill proceses.
    let dead_server = kill_homestar(proc_guard_server.take(), Some(Duration::from_secs(5)));
    let dead_client = kill_homestar(proc_guard_client1.take(), Some(Duration::from_secs(5)));

    // Retrieve logs.
    let stdout_server = retrieve_output(dead_server);
    let stdout_client = retrieve_output(dead_client);

    // Count discover requests on the server
    let server_discovery_count = count_lines_where(
        stdout_server,
        vec![
            "served rendezvous discover request to peer",
            "12D3KooWRndVhVZPCiQwHBBBdg769GyrPUW13zxwqQyf9r3ANaba",
        ],
    );

    // Count discovery responses the client
    let client_discovery_count = count_lines_where(
        stdout_client,
        vec![
            "received discovery from rendezvous server",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(server_discovery_count > 1);
    assert!(client_discovery_count > 1);

    Ok(())
}

#[test]
fn test_libp2p_rendezvous_rediscover_on_expiration_integration() -> Result<()> {
    const DB1: &str = "test_libp2p_rendezvous_rediscover_on_expiration_integration1.db";
    const DB2: &str = "test_libp2p_rendezvous_rediscover_on_expiration_integration2.db";
    const DB3: &str = "test_libp2p_rendezvous_rediscover_on_expiration_integration3.db";

    let _db_guard1 = FileGuard::new(DB1);
    let _db_guard2 = FileGuard::new(DB2);
    let _db_guard3 = FileGuard::new(DB3);

    // Start a rendezvous server
    let rendezvous_server = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous1.toml")
        .arg("--db")
        .arg(DB1)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_server = ChildGuard::new(rendezvous_server);

    if wait_for_socket_connection(8024, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Start a peer that will renew registrations with the rendezvous server every five seconds
    let rendezvous_client1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous6.toml")
        .arg("--db")
        .arg(DB2)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_client1 = ChildGuard::new(rendezvous_client1);

    if wait_for_socket_connection_v6(9830, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Wait for registration to complete.
    // TODO When we have WebSocket push events, listen on a registration event instead of using an arbitrary sleep.
    thread::sleep(Duration::from_secs(2));

    // Start a peer that will discover with the rendezvous server when
    // a discovered registration expires. Note that by default discovery only
    // occurs every ten minutes, so discovery requests in this test are driven
    // by expirations.
    let rendezvous_client2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_rendezvous3.toml")
        .arg("--db")
        .arg(DB3)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard_client2 = ChildGuard::new(rendezvous_client2);

    if wait_for_socket_connection(8027, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for seven seconds then kill proceses.
    let dead_server = kill_homestar(proc_guard_server.take(), Some(Duration::from_secs(7)));
    let _ = kill_homestar(proc_guard_client1.take(), Some(Duration::from_secs(7)));
    let dead_client2 = kill_homestar(proc_guard_client2.take(), Some(Duration::from_secs(7)));

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

    Ok(())
}
