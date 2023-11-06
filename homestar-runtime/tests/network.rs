use crate::utils::{check_lines_for, kill_homestar, retrieve_output, stop_homestar, BIN_NAME};
use anyhow::Result;
use once_cell::sync::Lazy;
use serial_test::file_serial;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

#[allow(dead_code)]
static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[file_serial]
fn test_libp2p_generates_peer_id_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

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

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_listens_on_address_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_lines_for(
        stdout,
        vec![
            "local node is listening",
            "/ip4/127.0.0.1/tcp/7000/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(logs_expected);

    Ok(())
}

#[test]
#[file_serial]
fn test_rpc_listens_on_address_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected = check_lines_for(stdout, vec!["RPC server listening", "[::1]:9820"]);

    assert!(logs_expected);

    Ok(())
}

#[cfg(feature = "websocket-server")]
#[test]
#[file_serial]
fn test_websocket_listens_on_address_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network1.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);
    let logs_expected =
        check_lines_for(stdout, vec!["websocket server listening", "127.0.0.1:8020"]);

    assert!(logs_expected);

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_connect_known_peers_serial() -> Result<()> {
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
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network2.toml")
        .arg("--db")
        .arg("homestar2.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

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

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_connect_after_mdns_discovery_serial() -> Result<()> {
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
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_mdns2.toml")
        .arg("--db")
        .arg("homestar2.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

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

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_disconnect_mdns_discovery_serial() -> Result<()> {
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
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_mdns2.toml")
        .arg("--db")
        .arg("homestar2.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

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

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_disconnect_known_peers_serial() -> Result<()> {
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
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_network2.toml")
        .arg("--db")
        .arg("homestar2.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

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
    assert_eq!(false, two_removed_from_dht_table);

    Ok(())
}
