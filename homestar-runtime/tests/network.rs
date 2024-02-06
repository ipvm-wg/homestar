use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, listen_addr, retrieve_output,
        wait_for_socket_connection_v6, ChildGuard, ProcInfo, BIN_NAME, ED25519MULTIHASH,
        SECP256K1MULTIHASH,
    },
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

#[cfg(feature = "websocket-notify")]
mod connection;
#[cfg(all(feature = "websocket-notify", feature = "test-utils"))]
mod dht;
#[cfg(feature = "websocket-notify")]
mod gossip;
mod mdns;
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
