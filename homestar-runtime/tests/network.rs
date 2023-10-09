use crate::utils::{kill_homestar, retrieve_output, stop_homestar, BIN_NAME};
use anyhow::Result;
use once_cell::sync::Lazy;
use predicates::prelude::*;
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
        .arg("tests/fixtures/network_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("local peer ID generated").eval(stdout.as_str())
    );
    assert_eq!(
        true,
        predicate::str::contains("peer_id=12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
            .eval(stdout.as_str())
    );

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_listens_on_address_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/network_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("local node is listening on /ip4/127.0.0.1/tcp/7000/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
            .eval(stdout.as_str())
    );

    Ok(())
}

#[test]
#[file_serial]
fn test_rpc_listens_on_address_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/network_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("RPC server listening on [::1]:3032").eval(stdout.as_str())
    );

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
        .arg("tests/fixtures/network_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("websocket server listening on 127.0.0.1:9092")
            .eval(stdout.as_str())
    );

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
        .arg("tests/fixtures/network_node1/config/settings.toml")
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
        .arg("tests/fixtures/network_node2/config/settings.toml")
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

    // Check that node one connected to node two.
    assert_eq!(
        true,
        predicate::str::contains("peer connection established").eval(stdout1.as_str())
    );
    assert_eq!(
        true,
        predicate::str::contains("peer_id=16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc")
            .eval(stdout1.as_str())
    );

    // Check that node two connected to node one.
    assert_eq!(
        true,
        predicate::str::contains("peer connection established").eval(stdout2.as_str())
    );
    assert_eq!(
        true,
        predicate::str::contains("peer_id=12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
            .eval(stdout2.as_str())
    );

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_connect_after_mdns_discovery_serial() -> Result<()> {
    let _ = stop_homestar();

    // Start two nodes each configured to listen at 0.0.0.0 with no known peers.
    // The nodes are configured with port 0 to allow the OS to selectn a port.
    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/mdns_connect1.toml")
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
        .arg("tests/fixtures/mdns_connect2.toml")
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
    assert_eq!(
        true,
        predicate::str::contains("peer connection established").eval(stdout1.as_str())
    );
    assert_eq!(
        true,
        predicate::str::contains("peer_id=16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc")
            .eval(stdout1.as_str())
    );

    // Check that node two connected to node one.
    assert_eq!(
        true,
        predicate::str::contains("peer connection established").eval(stdout2.as_str())
    );
    assert_eq!(
        true,
        predicate::str::contains("peer_id=12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN")
            .eval(stdout2.as_str())
    );

    Ok(())
}
