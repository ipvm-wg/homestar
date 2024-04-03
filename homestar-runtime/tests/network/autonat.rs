use crate::{
    make_config,
    utils::{
        check_for_line_with, kill_homestar, listen_addr, multiaddr, retrieve_output,
        subscribe_network_events, wait_for_socket_connection, ChildGuard, ProcInfo,
        TimeoutFutureExt, BIN_NAME, ED25519MULTIHASH, SECP256K1MULTIHASH,
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
fn test_autonat_confirms_address_integration() -> Result<()> {
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

    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p]
        listen_address = "{listen_addr1}"
        [node.network.libp2p.autonat]
        boot_delay = 1
        retry_interval = 3
        throttle_server_period = 2
        only_public_ips = false
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
        .arg(&proc_info1.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let proc_guard1 = ChildGuard::new(homestar_proc1);

    if wait_for_socket_connection(ws_port1, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    tokio_test::block_on(async {
        let toml2 = format!(
            r#"
            [node]
            [node.network.keypair_config]
            existing = {{ key_type = "secp256k1", path = "./fixtures/__testkey_secp256k1.der" }}
            [node.network.libp2p]
            listen_address = "{listen_addr2}"
            node_addresses = ["{node_addra}"]
            [node.network.libp2p.autonat]
            boot_delay = 1
            retry_interval = 3
            throttle_server_period = 2
            only_public_ips = false
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
        let proc_guard2 = ChildGuard::new(homestar_proc2);

        if wait_for_socket_connection(ws_port2, 1000).is_err() {
            panic!("Homestar server/runtime failed to start in time");
        }

        let mut net_events = subscribe_network_events(ws_port2).await;
        let sub = net_events.sub();

        // Poll for status changed autonat message
        loop {
            if let Ok(msg) = sub.next().with_timeout(Duration::from_secs(30)).await {
                let json: serde_json::Value =
                    serde_json::from_slice(&msg.unwrap().unwrap()).unwrap();

                if json["status_changed_autonat"].is_object()
                    && json["status_changed_autonat"]["status"] == "Public"
                {
                    break;
                }
            } else {
                panic!("Node two did not receive a NAT public status message in time.")
            }
        }

        // Kill proceses.
        let dead_proc1 = kill_homestar(proc_guard1.take(), None);
        let dead_proc2 = kill_homestar(proc_guard2.take(), None);

        // Retrieve logs.
        let stdout1 = retrieve_output(dead_proc1);
        let stdout2 = retrieve_output(dead_proc2);

        // Check node one successfully probed an address for node two
        let one_confirmed_address = check_for_line_with(
            stdout1,
            vec![
                "successfully probed an external address for a peer",
                SECP256K1MULTIHASH,
            ],
        );

        // Check node two received a probe confirmation from node one
        let two_received_address_confirmation = check_for_line_with(
            stdout2,
            vec![
                "peer successfully probed an external address",
                ED25519MULTIHASH,
            ],
        );

        assert!(one_confirmed_address);
        assert!(two_received_address_confirmation);
    });

    Ok(())
}
