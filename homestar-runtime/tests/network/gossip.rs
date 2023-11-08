use crate::utils::{
    check_lines_for, kill_homestar, remove_db, retrieve_output, startup_ipfs, stop_all_bins,
    BIN_NAME, IPFS,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use serial_test::file_serial;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[file_serial]
fn test_libp2p_receipt_gossip_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    let add_wasm_args = vec![
        "add",
        "--cid-version",
        "1",
        "../homestar-wasm/fixtures/example_add.wasm",
    ];

    let _ipfs_add_wasm = Command::new(IPFS)
        .args(add_wasm_args)
        .stdout(Stdio::piped())
        .output()
        .expect("`ipfs add` of wasm mod");

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_gossip1.toml")
        .arg("--db")
        .arg("homestar_test_libp2p_receipt_gossip_serial1.db")
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
        .arg("tests/fixtures/test_gossip2.toml")
        .arg("--db")
        .arg("homestar_test_libp2p_receipt_gossip_serial2.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Wait for nodes to be come online
    thread::sleep(Duration::from_secs(2));

    let _ = Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg("9790")
        .arg("-w")
        .arg("tests/fixtures/test-workflow-add-one.json")
        .output();

    // Collect logs for ten seconds then kill proceses.
    let dead_proc1 = kill_homestar(homestar_proc1, Some(Duration::from_secs(7)));
    let dead_proc2 = kill_homestar(homestar_proc2, Some(Duration::from_secs(7)));

    // Retrieve logs.
    let stdout1 = retrieve_output(dead_proc1);
    let stdout2 = retrieve_output(dead_proc2);

    // Check node one published a receipt
    let message_published = check_lines_for(stdout1, vec!["message published on receipts topic"]);

    // Check node two received a receipt from node one
    let message_received = check_lines_for(
        stdout2,
        vec![
            "message received on receipts topic",
            "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
        ],
    );

    assert!(message_published);
    assert!(message_received);

    let _ = stop_all_bins();

    Ok(())
}
