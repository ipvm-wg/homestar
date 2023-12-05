use crate::utils::{
    check_lines_for, count_lines_where, kill_homestar, remove_db, retrieve_output, stop_homestar,
    wait_for_socket_connection, wait_for_socket_connection_v6, BIN_NAME,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use serial_test::file_serial;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

// #[cfg(all(feature = "websocket-notify", feature = "test-utils"))]
mod dht;
#[cfg(feature = "websocket-notify")]
mod gossip;
#[cfg(feature = "websocket-notify")]
mod notification;

#[allow(dead_code)]
static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[file_serial]
fn test_libp2p_generates_peer_id_serial() -> Result<()> {
    const DB: &str = "test_libp2p_generates_peer_id_serial.db";
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection_v6(9820, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_lines_for(
        stdout,
        vec![
            "local peer ID generated",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(logs_expected);

    remove_db(DB);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_listens_on_address_serial() -> Result<()> {
    const DB: &str = "test_libp2p_listens_on_address_serial.db";
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection_v6(9820, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_lines_for(
        stdout,
        vec![
            "local node is listening",
            "/ip4/127.0.0.1/tcp/7000",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(logs_expected);

    remove_db(DB);

    Ok(())
}

#[test]
#[file_serial]
fn test_rpc_listens_on_address_serial() -> Result<()> {
    const DB: &str = "test_rpc_listens_on_address_serial.db";
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection_v6(9820, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_lines_for(stdout, vec!["RPC server listening", "[::1]:9820"]);

    assert!(logs_expected);

    remove_db(DB);

    Ok(())
}

#[test]
#[file_serial]
fn test_websocket_listens_on_address_serial() -> Result<()> {
    const DB: &str = "test_websocket_listens_on_address_serial.db";
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection_v6(9820, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_lines_for(stdout, vec!["webserver listening", "127.0.0.1:8020"]);

    assert!(logs_expected);

    remove_db(DB);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_connect_known_peers_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_connect_known_peers_serial1.db";
    const DB2: &str = "test_libp2p_connect_known_peers_serial2.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection_v6(9820, 1000).is_err() {
        let _ = kill_homestar(homestar_proc1, None);
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

    if wait_for_socket_connection_v6(9821, 1000).is_err() {
        let _ = kill_homestar(homestar_proc2, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for five seconds then kill proceses.
    let dead_proc1 = kill_homestar(homestar_proc1, Some(Duration::from_secs(5)));
    let dead_proc2 = kill_homestar(homestar_proc2, Some(Duration::from_secs(5)));

    // Retrieve logs.
    let stdout1 = retrieve_output(dead_proc1);
    let stdout2 = retrieve_output(dead_proc2);

    // Check node two was added to the Kademlia table
    let two_addded_to_dht = check_lines_for(
        stdout1.clone(),
        vec![
            "added configured node to kademlia routing table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that DHT routing table was updated with node two
    let two_in_dht_routing_table = check_lines_for(
        stdout1.clone(),
        vec![
            "kademlia routing table updated with peer",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that node one connected to node two.
    let one_connected_to_two = check_lines_for(
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
    let one_addded_to_dht = check_lines_for(
        stdout2.clone(),
        vec![
            "added configured node to kademlia routing table",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check that DHT routing table was updated with node one
    let one_in_dht_routing_table = check_lines_for(
        stdout2.clone(),
        vec![
            "kademlia routing table updated with peer",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check that node two connected to node one.
    let two_connected_to_one = check_lines_for(
        stdout2,
        vec![
            "peer connection established",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(one_addded_to_dht);
    assert!(one_in_dht_routing_table);
    assert!(two_connected_to_one);

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_connect_after_mdns_discovery_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_connect_after_mdns_discovery_serial1.db";
    const DB2: &str = "test_libp2p_connect_after_mdns_discovery_serial2.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection_v6(9800, 1000).is_err() {
        let _ = kill_homestar(homestar_proc1, None);
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

    if wait_for_socket_connection_v6(9801, 1000).is_err() {
        let _ = kill_homestar(homestar_proc2, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for seven seconds then kill processes.
    let dead_proc1 = kill_homestar(homestar_proc1, Some(Duration::from_secs(7)));
    let dead_proc2 = kill_homestar(homestar_proc2, Some(Duration::from_secs(7)));

    // Retrieve logs.
    let stdout1 = retrieve_output(dead_proc1);
    let stdout2 = retrieve_output(dead_proc2);

    // Check that node one connected to node two.
    let one_connected_to_two = check_lines_for(
        stdout1.clone(),
        vec![
            "peer connection established",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check node two was added to the Kademlia table
    let two_addded_to_dht = check_lines_for(
        stdout1.clone(),
        vec![
            "added identified node to kademlia routing table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that DHT routing table was updated with node two
    let two_in_dht_routing_table = check_lines_for(
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
    let two_connected_to_one = check_lines_for(
        stdout2.clone(),
        vec![
            "peer connection established",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check node one was added to the Kademlia table
    let one_addded_to_dht = check_lines_for(
        stdout2.clone(),
        vec![
            "added identified node to kademlia routing table",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    // Check that DHT routing table was updated with node one
    let one_in_dht_routing_table = check_lines_for(
        stdout2,
        vec![
            "kademlia routing table updated with peer",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(two_connected_to_one);
    assert!(one_addded_to_dht);
    assert!(one_in_dht_routing_table);

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_connect_rendezvous_discovery_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_connect_rendezvous_discovery_serial1.db";
    const DB2: &str = "test_libp2p_connect_rendezvous_discovery_serial2.db";
    const DB3: &str = "test_libp2p_connect_rendezvous_discovery_serial3.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection(8024, 1000).is_err() {
        let _ = kill_homestar(rendezvous_server, None);
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

    if wait_for_socket_connection(8026, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client1, None);
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

    if wait_for_socket_connection(8027, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client2, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for five seconds then kill proceses.
    let dead_server = kill_homestar(rendezvous_server, Some(Duration::from_secs(5)));
    let _ = kill_homestar(rendezvous_client1, Some(Duration::from_secs(5)));
    let dead_client2 = kill_homestar(rendezvous_client2, Some(Duration::from_secs(5)));

    // Retrieve logs.
    let stdout_server = retrieve_output(dead_server);
    let stdout_client2 = retrieve_output(dead_client2);

    // Check rendezvous server registered the client one
    let registered_client_one = check_lines_for(
        stdout_server.clone(),
        vec![
            "registered peer through rendezvous",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check rendezvous served a discover request to client two
    let served_discovery_to_client_two = check_lines_for(
        stdout_server.clone(),
        vec![
            "served rendezvous discover request to peer",
            "12D3KooWK99VoVxNE7XzyBwXEzW7xhK7Gpv85r9F3V3fyKSUKPH5",
        ],
    );

    assert!(registered_client_one);
    assert!(served_discovery_to_client_two);

    // Check that client two connected to client one.
    let two_connected_to_one = check_lines_for(
        stdout_client2.clone(),
        vec![
            "peer connection established",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check client one was added to the Kademlia table
    let one_addded_to_dht = check_lines_for(
        stdout_client2.clone(),
        vec![
            "added identified node to kademlia routing table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that DHT routing table was updated with client one
    let one_in_dht_routing_table = check_lines_for(
        stdout_client2.clone(),
        vec![
            "kademlia routing table updated with peer",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(one_addded_to_dht);
    assert!(one_in_dht_routing_table);
    assert!(two_connected_to_one);

    remove_db(DB1);
    remove_db(DB2);
    remove_db(DB3);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_disconnect_mdns_discovery_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_disconnect_mdns_discovery_serial1.db";
    const DB2: &str = "test_libp2p_disconnect_mdns_discovery_serial2.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection(8000, 1000).is_err() {
        let _ = kill_homestar(homestar_proc1, None);
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

    if wait_for_socket_connection(8001, 1000).is_err() {
        let _ = kill_homestar(homestar_proc2, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Kill node two after seven seconds.
    let _ = kill_homestar(homestar_proc2, Some(Duration::from_secs(7)));

    // Collect logs for eight seconds then kill node one.
    let dead_proc1 = kill_homestar(homestar_proc1, Some(Duration::from_secs(8)));

    // Retrieve logs.
    let stdout = retrieve_output(dead_proc1);

    // Check that node two disconnected from node one.
    let two_disconnected_from_one = check_lines_for(
        stdout.clone(),
        vec![
            "peer connection closed",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that node two was removed from the Kademlia table
    let two_removed_from_dht_table = check_lines_for(
        stdout.clone(),
        vec![
            "removed peer from kademlia table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(two_disconnected_from_one);
    assert!(two_removed_from_dht_table);

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_disconnect_known_peers_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_disconnect_known_peers_serial1.db";
    const DB2: &str = "test_libp2p_disconnect_known_peers_serial2.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection_v6(9820, 1000).is_err() {
        let _ = kill_homestar(homestar_proc1, None);
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

    if wait_for_socket_connection_v6(9821, 1000).is_err() {
        let _ = kill_homestar(homestar_proc2, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Kill node two after seven seconds.
    let _ = kill_homestar(homestar_proc2, Some(Duration::from_secs(7)));

    // Collect logs for eight seconds then kill node one.
    let dead_proc1 = kill_homestar(homestar_proc1, Some(Duration::from_secs(8)));

    // Retrieve logs.
    let stdout = retrieve_output(dead_proc1);

    // Check that node two disconnected from node one.
    let two_disconnected_from_one = check_lines_for(
        stdout.clone(),
        vec![
            "peer connection closed",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that node two was not removed from the Kademlia table.
    let two_removed_from_dht_table = check_lines_for(
        stdout.clone(),
        vec![
            "removed peer from kademlia table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(two_disconnected_from_one);
    assert!(!two_removed_from_dht_table);

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_disconnect_rendezvous_discovery_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_disconnect_rendezvous_discovery_serial1.db";
    const DB2: &str = "test_libp2p_disconnect_rendezvous_discovery_serial2.db";
    const DB3: &str = "test_libp2p_disconnect_rendezvous_discovery_serial3.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection(8024, 1000).is_err() {
        let _ = kill_homestar(rendezvous_server, None);
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

    if wait_for_socket_connection(8026, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client1, None);
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

    if wait_for_socket_connection(8027, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client1, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Kill server and client one after five seconds
    let _ = kill_homestar(rendezvous_server, Some(Duration::from_secs(5)));
    let _ = kill_homestar(rendezvous_client1, Some(Duration::from_secs(5)));

    // Collect logs for seven seconds then kill process.
    let dead_client2 = kill_homestar(rendezvous_client2, Some(Duration::from_secs(7)));

    // Retrieve logs.
    let stdout = retrieve_output(dead_client2);

    // Check that client two disconnected from client one.
    let two_disconnected_from_one = check_lines_for(
        stdout.clone(),
        vec![
            "peer connection closed",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    // Check that client two was removed from the Kademlia table
    let two_removed_from_dht_table = check_lines_for(
        stdout.clone(),
        vec![
            "removed peer from kademlia table",
            "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc",
        ],
    );

    assert!(two_disconnected_from_one);
    assert!(two_removed_from_dht_table);

    remove_db(DB1);
    remove_db(DB2);
    remove_db(DB3);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_rendezvous_renew_registration_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_rendezvous_renew_registration_serial1.db";
    const DB2: &str = "test_libp2p_rendezvous_renew_registration_serial2.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection(8024, 1000).is_err() {
        let _ = kill_homestar(rendezvous_server, None);
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

    if wait_for_socket_connection(8028, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client1, None);
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

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_rendezvous_rediscovery_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_rendezvous_rediscovery_serial1.db";
    const DB2: &str = "test_libp2p_rendezvous_rediscovery_serial2.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection(8024, 1000).is_err() {
        let _ = kill_homestar(rendezvous_server, None);
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

    if wait_for_socket_connection_v6(9829, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client1, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for five seconds then kill proceses.
    let dead_server = kill_homestar(rendezvous_server, Some(Duration::from_secs(5)));
    let dead_client = kill_homestar(rendezvous_client1, Some(Duration::from_secs(5)));

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

    remove_db(DB1);
    remove_db(DB2);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_rendezvous_rediscover_on_expiration_serial() -> Result<()> {
    const DB1: &str = "test_libp2p_rendezvous_rediscover_on_expiration_serial1.db";
    const DB2: &str = "test_libp2p_rendezvous_rediscover_on_expiration_serial2.db";
    const DB3: &str = "test_libp2p_rendezvous_rediscover_on_expiration_serial3.db";
    let _ = stop_homestar();

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

    if wait_for_socket_connection(8024, 1000).is_err() {
        let _ = kill_homestar(rendezvous_server, None);
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

    if wait_for_socket_connection_v6(9830, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client1, None);
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

    if wait_for_socket_connection(8027, 1000).is_err() {
        let _ = kill_homestar(rendezvous_client1, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    // Collect logs for seven seconds then kill proceses.
    let dead_server = kill_homestar(rendezvous_server, Some(Duration::from_secs(7)));
    let _ = kill_homestar(rendezvous_client1, Some(Duration::from_secs(7)));
    let dead_client2 = kill_homestar(rendezvous_client2, Some(Duration::from_secs(7)));

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

    remove_db(DB1);
    remove_db(DB2);
    remove_db(DB3);

    Ok(())
}
