use anyhow::{Context, Result};
use assert_cmd::crate_name;
#[cfg(not(windows))]
use assert_cmd::prelude::*;
#[cfg(not(windows))]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use once_cell::sync::Lazy;
use retry::{delay::Fixed, retry};
use std::{
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
};
#[cfg(not(windows))]
use sysinfo::PidExt;
use sysinfo::{ProcessExt, SystemExt};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(crate_name!()));
const IPFS: &str = "ipfs";

pub(crate) fn startup_ipfs() -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".ipfs");
    println!("starting ipfs daemon...{}", path.to_str().unwrap());
    let mut ipfs_daemon = Command::new(IPFS)
        .args([
            "--repo-dir",
            path.to_str().unwrap(),
            "--offline",
            "daemon",
            "--init",
        ])
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

pub(crate) fn stop_homestar() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("stop")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to stop Homestar server")?;

    Ok(())
}

pub(crate) fn stop_ipfs() -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".ipfs");
    Command::new(IPFS)
        .args(["--repo-dir", path.to_str().unwrap(), "shutdown"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to stop IPFS daemon")?;

    Ok(())
}

pub(crate) fn stop_all_bins() -> Result<()> {
    let _ = stop_ipfs();
    let _ = stop_homestar();

    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn kill_homestar_process() -> Result<()> {
    let system = sysinfo::System::new_all();
    let pid = system
        .processes_by_exact_name("homestar-runtime")
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
    let _result = retry(Fixed::from_millis(500), || {
        Command::new(BIN.as_os_str())
            .arg("ping")
            .assert()
            .try_failure()
    });

    Ok(())
}

#[cfg(windows)]
pub(crate) fn kill_homestar_process() -> Result<()> {
    let system = sysinfo::System::new_all();
    let pid = system
        .processes_by_exact_name("homestar-runtime.exe")
        .collect::<Vec<_>>()
        .first()
        .map(|x| x.pid())
        .unwrap();

    if let Some(process) = system.process(pid) {
        process.kill();
    };

    Ok(())
}
