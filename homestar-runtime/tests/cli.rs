use crate::utils::{kill_homestar_process, startup_ipfs, stop_all_bins};
use anyhow::Result;
use assert_cmd::{crate_name, prelude::*};
use once_cell::sync::Lazy;
use predicates::prelude::*;
use retry::{delay::Fixed, retry};
use serial_test::file_serial;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(crate_name!()));

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
            crate_name!(),
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
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

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
        .arg("run")
        .arg("-w")
        .arg("./fixtures/test-workflow-add-one.json")
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
        .arg("-w")
        .arg("./fixtures/test-workflow-add-one.json")
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

    if let Ok(None) = homestar_proc.try_wait() {
        let _status_code = match homestar_proc.wait_timeout(Duration::from_secs(1)).unwrap() {
            Some(status) => status.code(),
            None => {
                homestar_proc.kill().unwrap();
                homestar_proc.wait().unwrap().code()
            }
        };
    }

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
        .arg("-d")
        .env("DATABASE_URL", "homestar.db")
        .stdout(Stdio::piped())
        .assert()
        .success();

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

    let _ = stop_all_bins();
    let _ = kill_homestar_process();

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
        .arg("fixtures/test_v4.toml")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9999);
    let result = retry(Fixed::from_millis(500), || {
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
        .arg("9999")
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

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
        .arg("fixtures/test_v4.toml")
        .arg("-d")
        .env("DATABASE_URL", "homestar.db")
        .stdout(Stdio::piped())
        .assert()
        .success();

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9999);
    let result = retry(Fixed::from_millis(500), || {
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
        .arg("9999")
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    let _ = stop_all_bins();
    let _ = kill_homestar_process();

    Ok(())
}
