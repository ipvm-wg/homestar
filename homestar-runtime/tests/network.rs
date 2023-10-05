use crate::utils::{kill_homestar, retrieve_output, stop_homestar, BIN_NAME};
use anyhow::Result;
use once_cell::sync::Lazy;
use predicates::prelude::*;
use serial_test::file_serial;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread,
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
        .arg("tests/test_node1/config/settings.toml")
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
        predicate::str::contains("peer_id=12D3KooWBYAug7e9eE7z1z1jaYjfVQCfRy8r3keVYD8jdsEGZHr")
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
        .arg("tests/test_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("local node is listening on /ip4/127.0.0.1/tcp/7000/p2p/12D3KooWBYAug7e9eE7z1z1jaYjfVQCfRy8r3keVYD8jdsEGZHrT")
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
        .arg("tests/test_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("RPC server listening on [::1]:3031").eval(stdout.as_str())
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
        .arg("tests/test_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let dead_proc = kill_homestar(homestar_proc, None);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("websocket server listening on 127.0.0.1:9091")
            .eval(stdout.as_str())
    );

    Ok(())
}

#[test]
#[file_serial]
fn test_libp2p_connection_serial() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/test_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/test_node2/config/settings.toml")
        .arg("--db")
        .arg("homestar2.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Delays one second before kill
    let dead_proc1 = kill_homestar(homestar_proc1, Some(Duration::from_secs(5)));
    let dead_proc2 = kill_homestar(homestar_proc2, Some(Duration::from_secs(5)));

    let stdout1 = retrieve_output(dead_proc1);
    let stdout2 = retrieve_output(dead_proc2);

    // Homestar node 1 connects to node 2
    assert_eq!(
        true,
        predicate::str::contains("peer connection established").eval(stdout1.as_str())
    );
    assert_eq!(
        true,
        predicate::str::contains("peer_id=16Uiu2HAm4nuRTJXGe1VtYWrtoxTnJdjfM2ohWvXsFxyNXG2F395P")
            .eval(stdout1.as_str())
    );

    // Homestar node 2 connects to node 1
    assert_eq!(
        true,
        predicate::str::contains("peer connection established").eval(stdout2.as_str())
    );
    assert_eq!(
        true,
        predicate::str::contains("peer_id=12D3KooWBYAug7e9eE7z1z1jaYjfVQCfRy8r3keVYD8jdsEGZHrT")
            .eval(stdout2.as_str())
    );

    Ok(())
}
