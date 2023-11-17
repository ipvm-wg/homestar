#[cfg(not(windows))]
use crate::utils::kill_homestar_daemon;
use crate::utils::{
    kill_homestar, remove_db, stop_homestar, wait_for_socket_connection,
    wait_for_socket_connection_v6, BIN_NAME,
};
use anyhow::Result;
use assert_cmd::prelude::*;
use once_cell::sync::Lazy;
use predicates::prelude::*;
use serial_test::file_serial;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[file_serial]
fn test_help_serial() -> Result<()> {
    let _ = stop_homestar();

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

    let _ = stop_homestar();

    Ok(())
}

#[test]
#[file_serial]
fn test_version_serial() -> Result<()> {
    let _ = stop_homestar();

    Command::new(BIN.as_os_str())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{} {}",
            BIN_NAME,
            env!("CARGO_PKG_VERSION")
        )));

    let _ = stop_homestar();

    Ok(())
}

#[test]
#[file_serial]
fn test_server_not_running_serial() -> Result<()> {
    let _ = stop_homestar();

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

    let _ = stop_homestar();

    Ok(())
}

#[test]
#[file_serial]
fn test_server_serial() -> Result<()> {
    const DB: &str = "test_server_serial.db";
    let _ = stop_homestar();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-db")
        .arg(DB)
        .assert()
        .failure();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_v6.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection_v6(9837, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .arg("-p")
        .arg("9837")
        .assert()
        .success()
        .stdout(predicate::str::contains("::1"))
        .stdout(predicate::str::contains("pong"));

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .arg("-p")
        .arg("9835")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();

    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_homestar();
    remove_db(DB);

    Ok(())
}

#[test]
#[file_serial]
fn test_workflow_run_serial() -> Result<()> {
    const DB: &str = "test_workflow_run_serial.db";

    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_workflow1.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection_v6(9840, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg("9840")
        .arg("-w")
        .arg("tests/fixtures/test-workflow-add-one.json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        ))
        .stdout(predicate::str::contains("num_tasks"))
        .stdout(predicate::str::contains("progress_count"));

    // run another one of the same!
    Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg("9840")
        .arg("-w")
        .arg("tests/fixtures/test-workflow-add-one.json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ipfs://bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia",
        ))
        .stdout(predicate::str::contains("num_tasks"))
        .stdout(predicate::str::contains("progress_count"));

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();
    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_homestar();
    remove_db(DB);

    Ok(())
}

#[test]
#[file_serial]
#[cfg(not(windows))]
fn test_daemon_serial() -> Result<()> {
    const DB: &str = "test_daemon_serial.db";
    let _ = stop_homestar();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_v4.toml")
        .arg("-d")
        .env("DATABASE_URL", DB)
        .stdout(Stdio::piped())
        .assert()
        .success();

    if wait_for_socket_connection(9000, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg("9000")
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    let _ = stop_homestar();
    let _ = kill_homestar_daemon();
    remove_db(DB);

    Ok(())
}

#[test]
#[file_serial]
fn test_server_v4_serial() -> Result<()> {
    const DB: &str = "test_server_v4_serial.db";
    let _ = stop_homestar();

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_v4.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if wait_for_socket_connection(9000, 1000).is_err() {
        let _ = kill_homestar(homestar_proc, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg("9000")
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();
    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_homestar();
    remove_db(DB);

    Ok(())
}
