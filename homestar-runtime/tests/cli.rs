#[cfg(not(windows))]
use crate::utils::kill_homestar_daemon;
use crate::utils::{kill_homestar, startup_ipfs, stop_all_bins, BIN_NAME};
use anyhow::Result;
use assert_cmd::prelude::*;
use once_cell::sync::Lazy;
use predicates::prelude::*;
use retry::{delay::Exponential, retry};
use serial_test::file_serial;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[file_serial]
fn test_help_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

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

    let _ = stop_all_bins();

    Ok(())
}

#[test]
#[file_serial]
fn test_version_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    Command::new(BIN.as_os_str())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{} {}",
            BIN_NAME,
            env!("CARGO_PKG_VERSION")
        )));

    let _ = stop_all_bins();

    Ok(())
}

#[test]
#[file_serial]
fn test_server_not_running_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

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

    let _ = stop_all_bins();

    Ok(())
}

#[test]
#[file_serial]
fn test_server_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-db")
        .arg("homestar.db")
        .assert()
        .failure();

    let mut homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_v6.toml")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 9837);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        homestar_proc.kill().unwrap();
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
    let _ = stop_all_bins();

    Ok(())
}

#[cfg(feature = "test-utils")]
#[test]
#[file_serial]
fn test_workflow_run_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    let mut homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_workflow.toml")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 9840);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        homestar_proc.kill().unwrap();
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
            "bafyrmibcfltf6vhtfdson5z4av4r4wg3rccpt4hxajt54msacojeecazqy",
        ))
        .stdout(predicate::str::contains(
            "ipfs://bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
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
            "bafyrmibcfltf6vhtfdson5z4av4r4wg3rccpt4hxajt54msacojeecazqy",
        ))
        .stdout(predicate::str::contains(
            "ipfs://bafybeiabbxwf2vn4j3zm7bbojr6rt6k7o6cg6xcbhqkllubmsnvocpv7y4",
        ))
        .stdout(predicate::str::contains("num_tasks"))
        .stdout(predicate::str::contains("progress_count"));

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();

    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_all_bins();

    Ok(())
}

#[test]
#[file_serial]
#[cfg(not(windows))]
fn test_daemon_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_v4_alt.toml")
        .arg("-d")
        .env("DATABASE_URL", "homestar.db")
        .stdout(Stdio::piped())
        .assert()
        .success();

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9836);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg("9836")
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    let _ = stop_all_bins();
    let _ = kill_homestar_daemon();

    Ok(())
}

#[test]
#[file_serial]
#[cfg(windows)]
fn test_signal_kill_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    let mut homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3030);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .assert()
        .success()
        .stdout(predicate::str::contains("::1"))
        .stdout(predicate::str::contains("pong"));

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();
    let _ = kill_homestar(homestar_proc, None);

    Command::new(BIN.as_os_str()).arg("ping").assert().failure();

    let _ = stop_all_bins();

    Ok(())
}

#[test]
#[file_serial]
#[cfg(windows)]
fn test_server_v4_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    let mut homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_v4.toml")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9835);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        homestar_proc.kill().unwrap();
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg("9835")
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();

    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_all_bins();

    Ok(())
}

#[test]
#[file_serial]
#[cfg(not(windows))]
fn test_daemon_v4_serial() -> Result<()> {
    let _ = stop_all_bins();

    #[cfg(feature = "ipfs")]
    let _ = startup_ipfs();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_v4.toml")
        .arg("-d")
        .env("DATABASE_URL", "homestar.db")
        .stdout(Stdio::piped())
        .assert()
        .success();

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9835);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg("9835")
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    let _ = stop_all_bins();
    let _ = kill_homestar_daemon();

    Ok(())
}
