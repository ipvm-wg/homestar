use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, listen_addr, retrieve_output,
        wait_for_socket_connection, wait_for_socket_connection_v6, ChildGuard, ProcInfo, BIN_NAME,
        ED25519MULTIHASH, SECP256K1MULTIHASH,
    },
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

#[cfg(feature = "websocket-notify")]
mod autonat;
#[cfg(feature = "websocket-notify")]
mod connection;
#[cfg(all(feature = "websocket-notify", feature = "test-utils"))]
mod dht;
#[cfg(feature = "websocket-notify")]
mod gossip;
#[cfg(feature = "websocket-notify")]
mod mdns;
#[cfg(feature = "websocket-notify")]
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
        .env("RUST_BACKTRACE", "0")
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
        .env("RUST_BACKTRACE", "0")
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
        .env("RUST_BACKTRACE", "0")
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
        .env("RUST_BACKTRACE", "0")
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
fn test_node_info_endpoint_integration() -> Result<()> {
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
    let config1 = make_config!(toml);

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env("RUST_BACKTRACE", "0")
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg(config1.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection(ws_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::block_on(async {
        // Check node endpoint to match
        let http_url = format!("http://localhost:{}", ws_port);
        let http_resp = reqwest::get(format!("{}/node", http_url)).await.unwrap();
        assert_eq!(http_resp.status(), 200);
        let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();
        assert_eq!(
            http_resp,
            serde_json::json!({
                    "static": {"peer_id": ED25519MULTIHASH},
                    "dynamic": {"listeners": [format!("{listen_addr}")], "connections": {}}
            })
        );
    });

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_discovery_endpoint_integration() -> Result<()> {
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
    let config1 = make_config!(toml);

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env("RUST_BACKTRACE", "0")
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg(config1.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection(ws_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::block_on(async {
        // Check discovery endpoint to match
        let http_url = format!("http://localhost:{}", ws_port);
        let http_resp = reqwest::get(format!("{}/rpc_discover", http_url))
            .await
            .unwrap();
        assert_eq!(http_resp.status(), 200);
        let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();

        const API_SCHEMA_DOC: &str = include_str!("../schemas/api.json");
        assert_eq!(http_resp, serde_json::json!(API_SCHEMA_DOC));
    });

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
        .env("RUST_BACKTRACE", "0")
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
