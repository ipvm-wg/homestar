#![allow(unused_must_use)]

#[cfg(not(windows))]
use crate::utils::kill_homestar_daemon;
use crate::{
    make_config,
    utils::{
        wait_for_socket_connection, wait_for_socket_connection_v6, ChildGuard, ProcInfo, BIN_NAME,
    },
};
use anyhow::Result;
use assert_cmd::prelude::*;
use once_cell::sync::Lazy;
use predicates::prelude::*;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
fn test_help_integration() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("ping"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("help"))
        .stdout(predicate::str::contains("version"));

    Command::new(BIN.as_os_str())
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("ping"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("help"))
        .stdout(predicate::str::contains("version"));

    Ok(())
}

#[test]
fn test_version_integration() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{} {}",
            BIN_NAME,
            env!("CARGO_PKG_VERSION")
        )));

    Ok(())
}

#[test]
fn test_server_not_running_integration() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("ping")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

    Command::new(BIN.as_os_str())
        .arg("stop")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("server was already shutdown")
                    .or(predicate::str::contains("No connection could be made"))),
        );

    Ok(())
}

#[test]
fn test_server_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        host = "::1"
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-db")
        .arg(&proc_info.db_path)
        .assert()
        .failure();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .arg("-p")
        .arg(rpc_port.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("::1"))
        .stdout(predicate::str::contains("pong"));

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .arg("-p")
        .arg(port_selector::random_free_port().unwrap().to_string())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

    Ok(())
}

#[test]
fn test_workflow_run_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
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
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg(rpc_port.to_string())
        .arg("-w")
        .arg("tests/fixtures/test-workflow-add-one.json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ipfs://bafybeidbyqpmztqkeot33lz4ev2ftjhqrnbh67go56tlgbf7qmy5xyzvg4",
        ))
        .stdout(predicate::str::contains("num_tasks"))
        .stdout(predicate::str::contains("progress_count"));

    // run another one of the same!
    Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg(rpc_port.to_string())
        .arg("-w")
        .arg("tests/fixtures/test-workflow-add-one.json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ipfs://bafybeidbyqpmztqkeot33lz4ev2ftjhqrnbh67go56tlgbf7qmy5xyzvg4",
        ))
        .stdout(predicate::str::contains("num_tasks"))
        .stdout(predicate::str::contains("progress_count"));

    Ok(())
}

#[test]
#[test_retry::retry]
#[cfg(not(windows))]
fn test_daemon_serial() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        host = "127.0.0.1"
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );
    let config = make_config!(toml);

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("-d")
        .env("DATABASE_URL", &proc_info.db_path)
        .stdout(Stdio::piped())
        .assert()
        .success();

    if wait_for_socket_connection(rpc_port, 1000).is_err() {
        kill_homestar_daemon().unwrap();
        panic!("Homestar server/runtime failed to start in time");
    }

    let res = Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(rpc_port.to_string())
        .assert()
        .try_success();

    match res {
        Ok(ok) => {
            ok.stdout(predicate::str::contains("127.0.0.1"))
                .stdout(predicate::str::contains("pong"));
        }
        Err(err) => {
            kill_homestar_daemon().unwrap();
            panic!("Err: {:?}", err);
        }
    }

    Ok(())
}

#[test]
fn test_server_v4_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        host = "127.0.0.1"
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
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(rpc_port.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    Ok(())
}
