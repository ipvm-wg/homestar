use anyhow::{Context, Result};
#[cfg(not(windows))]
use assert_cmd::prelude::*;
#[cfg(not(windows))]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use once_cell::sync::Lazy;
use predicates::prelude::*;
use retry::{delay::Fixed, retry};
use std::{
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};
use strip_ansi_escapes;
#[cfg(not(windows))]
use sysinfo::PidExt;
use sysinfo::{ProcessExt, SystemExt};
use wait_timeout::ChildExt;

/// Binary name, which is different than the crate name.
pub(crate) const BIN_NAME: &str = "homestar";

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const IPFS: &str = "ipfs";

/// Start-up IPFS daemon for tests with the feature turned-on.
pub(crate) fn startup_ipfs() -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".ipfs");
    println!("starting ipfs daemon...{}", path.to_str().unwrap());
    let mut ipfs_daemon = Command::new(IPFS)
        .args(["--offline", "daemon", "--init"])
        .stdout(Stdio::piped())
        .spawn()?;

    // wait for ipfs daemon to start by testing for a connection
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);
    let result = retry(Fixed::from_millis(500), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if let Err(err) = result {
        ipfs_daemon.kill().unwrap();
        panic!("`ipfs daemon` failed to start: {:?}", err);
    } else {
        Ok(())
    }
}

/// Stop the Homestar server/binary.
pub(crate) fn stop_homestar() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("stop")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to stop Homestar server")?;

    Ok(())
}

/// Stop the IPFS binary.
pub(crate) fn stop_ipfs() -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".ipfs");
    Command::new(IPFS)
        .args(["--repo-dir", path.to_str().unwrap(), "shutdown"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to stop IPFS daemon")?;
    rm_rf::ensure_removed(path).unwrap();

    Ok(())
}

/// Stop all binaries.
pub(crate) fn stop_all_bins() -> Result<()> {
    let _ = stop_ipfs();
    let _ = stop_homestar();

    Ok(())
}

/// Retrieve process output.
pub(crate) fn retrieve_output(proc: Child) -> String {
    let output = proc.wait_with_output().expect("failed to wait on child");
    let plain_stdout_bytes = strip_ansi_escapes::strip(output.stdout);
    String::from_utf8(plain_stdout_bytes).unwrap()
}

/// Check process output for all predicates in any line
pub(crate) fn check_lines_for(output: String, predicates: Vec<&str>) -> bool {
    output
        .split("\n")
        .map(|line| {
            // Line contains all predicates
            predicates
                .iter()
                .map(|pred| predicate::str::contains(*pred).eval(line))
                .fold(true, |acc, curr| acc && curr)
        })
        .fold(false, |acc, curr| acc || curr)
}

/// Wait for process to exit or kill after timeout.
pub(crate) fn kill_homestar(mut homestar_proc: Child, timeout: Option<Duration>) -> Child {
    if let Ok(None) = homestar_proc.try_wait() {
        let _status_code = match homestar_proc
            .wait_timeout(timeout.unwrap_or(Duration::from_secs(1)))
            .unwrap()
        {
            Some(status) => status.code(),
            None => {
                homestar_proc.kill().unwrap();
                homestar_proc.wait().unwrap().code()
            }
        };
    }

    homestar_proc
}

/// Kill the Homestar proc running as a daemon.
#[cfg(not(windows))]
pub(crate) fn kill_homestar_daemon() -> Result<()> {
    let system = sysinfo::System::new_all();
    let pid = system
        .processes_by_exact_name(BIN_NAME)
        .collect::<Vec<_>>()
        .first()
        .map(|p| p.pid().as_u32())
        .unwrap_or(
            std::fs::read_to_string("/tmp/homestar.pid")
                .expect("Should have a PID file")
                .trim()
                .parse::<u32>()
                .unwrap(),
        );

    let _result = signal::kill(Pid::from_raw(pid.try_into().unwrap()), Signal::SIGTERM);
    let _result = retry(Fixed::from_millis(1000).take(5), || {
        Command::new(BIN.as_os_str())
            .arg("ping")
            .assert()
            .try_failure()
    });

    Ok(())
}

/// Kill the Homestar proc running as a daemon.
#[allow(dead_code)]
#[cfg(windows)]
pub(crate) fn kill_homestar_daemon() -> Result<()> {
    let system = sysinfo::System::new_all();
    let pid = system
        .processes_by_exact_name(format!("{}.exe", BIN_NAME).as_str())
        .collect::<Vec<_>>()
        .first()
        .map(|x| x.pid())
        .unwrap();

    if let Some(process) = system.process(pid) {
        process.kill();
    };

    Ok(())
}
