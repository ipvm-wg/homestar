use anyhow::{Context, Result};
use assert_cmd::{crate_name, prelude::*};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use once_cell::sync::Lazy;
use predicates::prelude::*;
use retry::{delay::Fixed, retry};
use serial_test::serial;
use std::{
    fs,
    net::{IpAddr, Ipv6Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};
use sysinfo::{PidExt, ProcessExt, SystemExt};
use wait_timeout::ChildExt;

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(crate_name!()));

fn stop_bin() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("stop")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to stop Homestar server")?;
    Ok(())
}

#[test]
#[serial]
fn test_help_serial() -> Result<()> {
    let _ = stop_bin();
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
    let _ = stop_bin();

    Ok(())
}

#[test]
#[serial]
fn test_version_serial() -> Result<()> {
    let _ = stop_bin();
    Command::new(BIN.as_os_str())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{} {}",
            crate_name!(),
            env!("CARGO_PKG_VERSION")
        )));
    let _ = stop_bin();

    Ok(())
}

#[test]
#[serial]
fn test_server_not_running_serial() -> Result<()> {
    let _ = stop_bin();

    Command::new(BIN.as_os_str())
        .arg("ping")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Connection refused"));

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Connection refused"));

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::2")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No route to host")
                .or(predicate::str::contains("Network is unreachable")),
        );

    Command::new(BIN.as_os_str())
        .arg("stop")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("server was already shutdown")),
        );
    let _ = stop_bin();

    Ok(())
}

#[test]
#[serial]
fn test_server_serial() -> Result<()> {
    let _ = stop_bin();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-db")
        .arg("homestar.db")
        .assert()
        .failure();

    let mut homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3030);
    let result = retry(Fixed::from_millis(500), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        homestar_proc.kill().unwrap();
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .assert()
        .success()
        .stdout(predicate::str::contains("::1"))
        .stdout(predicate::str::contains("pong"));

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("-p")
        .arg("9999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Connection refused"));

    let _ = Command::new(BIN.as_os_str()).arg("stop").output();

    if let Ok(None) = homestar_proc.try_wait() {
        let _status_code = match homestar_proc.wait_timeout(Duration::from_secs(1)).unwrap() {
            Some(status) => status.code(),
            None => {
                homestar_proc.kill().unwrap();
                homestar_proc.wait().unwrap().code()
            }
        };
    }
    let _ = stop_bin();

    Ok(())
}

#[test]
#[serial]
fn test_daemon_serial() -> Result<()> {
    let _ = stop_bin();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-d")
        .env("DATABASE_URL", "homestar.db")
        .stdout(Stdio::piped())
        .assert()
        .success();

    let system = sysinfo::System::new_all();
    let pid = system
        .processes_by_exact_name("homestar-runtime")
        .collect::<Vec<_>>()
        .first()
        .map(|p| p.pid().as_u32())
        .unwrap_or(
            fs::read_to_string("/tmp/homestar.pid")
                .expect("Should have a PID file")
                .trim()
                .parse::<u32>()
                .unwrap(),
        );

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3030);
    let result = retry(Fixed::from_millis(500), || {
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

    let _result = signal::kill(Pid::from_raw(pid.try_into().unwrap()), Signal::SIGTERM);

    Command::new(BIN.as_os_str()).arg("ping").assert().failure();
    let _ = stop_bin();

    Ok(())
}
