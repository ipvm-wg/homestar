use crate::utils::{kill_homestar, retrieve_output, stop_homestar, BIN_NAME};
use anyhow::Result;
use once_cell::sync::Lazy;
use predicates::prelude::*;
use serial_test::file_serial;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

#[allow(dead_code)]
static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[file_serial]
fn test_libp2p_generates_peer_id() -> Result<()> {
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

    let dead_proc = kill_homestar(homestar_proc);
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
fn test_libp2p_listens_on_address() -> Result<()> {
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

    let dead_proc = kill_homestar(homestar_proc);
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
fn test_rpc_listens_on_address() -> Result<()> {
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

    let dead_proc = kill_homestar(homestar_proc);
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
fn test_websocket_listens_on_address() -> Result<()> {
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

    let dead_proc = kill_homestar(homestar_proc);
    let stdout = retrieve_output(dead_proc);

    assert_eq!(
        true,
        predicate::str::contains("websocket server listening on 127.0.0.1:9091")
            .eval(stdout.as_str())
    );

    Ok(())
}

// #[cfg(feature = "test-utils")]
// #[test]
// #[file_serial]
// fn test_libp2p_connect() -> Result<()> {
//     let _ = stop_homestar();

//     let mut homestar_proc1 = Command::new(BIN.as_os_str())
//         .arg("start")
//         .arg("-c")
//         .arg("tests/test_node1/config/settings.toml")
//         .arg("--db")
//         .arg("homestar1.db")
//         .stdout(Stdio::piped())
//         .spawn()
//         .unwrap();

// let dead_proc1 = kill_homestar(homestar_proc1);
// thread::sleep(Duration::from_millis(600));
// let homestar_proc2 = Command::new(BIN.as_os_str())
//     .arg("start")
//     .arg("-c")
//     .arg("tests/test_node2/config/settings.toml")
//     .arg("--db")
//     .arg("homestar2.db")
//     .stderr(Stdio::piped())
//     .spawn()
//     .unwrap();

// let output: Output;

// let dead_proc1 = kill_homestar(homestar_proc1);
// let _ = kill_homestar(homestar_proc2);
// let _ = stop_homestar();

// let output = dead_proc1
//     .unwrap()
//     .wait_with_output()
//     .expect("failed to wait on child");

// assert_eq!(
//     true,
//     predicate::str::contains("local peer ID generated")
//         .eval(String::from_utf8(output.stdout).unwrap().as_str())
// );
// println!("{:?}", output);

// Ok(())
// }
