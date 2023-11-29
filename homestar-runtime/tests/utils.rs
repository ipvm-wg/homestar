use anyhow::{Context, Result};
#[cfg(not(windows))]
use assert_cmd::prelude::*;
use chrono::{DateTime, FixedOffset};
#[cfg(not(windows))]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use once_cell::sync::Lazy;
use predicates::prelude::*;
#[cfg(not(windows))]
use retry::delay::Fixed;
use retry::{delay::Exponential, retry};
use std::{
    fs,
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::Duration,
};
#[cfg(not(windows))]
use sysinfo::PidExt;
use sysinfo::{ProcessExt, SystemExt};
use tokio::time::{timeout, Timeout};
use wait_timeout::ChildExt;

/// Binary name, which is different than the crate name.
pub(crate) const BIN_NAME: &str = "homestar";

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

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

/// Retrieve process output.
pub(crate) fn retrieve_output(proc: Child) -> String {
    let output = proc.wait_with_output().expect("failed to wait on child");
    let plain_stdout_bytes = strip_ansi_escapes::strip(output.stdout);
    String::from_utf8(plain_stdout_bytes).unwrap()
}

/// Check process output for all predicates in any line
pub(crate) fn check_lines_for(output: String, predicates: Vec<&str>) -> bool {
    output
        .split('\n')
        .map(|line| line_contains(line, &predicates))
        .any(|curr| curr)
}

pub(crate) fn count_lines_where(output: String, predicates: Vec<&str>) -> i32 {
    output.split('\n').fold(0, |count, line| {
        if line_contains(line, &predicates) {
            count + 1
        } else {
            count
        }
    })
}

/// Extract timestamps for process output lines with matching predicates
#[allow(dead_code)]
pub(crate) fn extract_timestamps_where(
    output: String,
    predicates: Vec<&str>,
) -> Vec<DateTime<FixedOffset>> {
    output.split('\n').fold(vec![], |mut timestamps, line| {
        if line_contains(line, &predicates) {
            match extract_label(line, "ts").and_then(|val| DateTime::parse_from_rfc3339(val).ok()) {
                Some(datetime) => {
                    timestamps.push(datetime);
                    timestamps
                }
                None => {
                    println!("Encountered a log entry that was missing a timestamp label: {line}");
                    timestamps
                }
            }
        } else {
            timestamps
        }
    })
}

/// Check process output line for all predicates
fn line_contains(line: &str, predicates: &[&str]) -> bool {
    predicates
        .iter()
        .map(|pred| predicate::str::contains(*pred).eval(line))
        .all(|curr| curr)
}

/// Extract label value from process output line
#[allow(dead_code)]
fn extract_label<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.split(' ')
        .find(|label| predicate::str::contains(format!("{key}=")).eval(label))
        .and_then(|label| label.split('=').next_back())
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

/// Remove sqlite database and associated temporary files
pub(crate) fn remove_db(name: &str) {
    let _ = fs::remove_file(name);
    let _ = fs::remove_file(format!("{name}-shm"));
    let _ = fs::remove_file(format!("{name}-wal"));
}

/// Wait for socket connection or timeout
pub(crate) fn wait_for_socket_connection(port: u16, exp_retry_base: u64) -> Result<(), ()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let result = retry(Exponential::from_millis(exp_retry_base).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    result.map_or_else(|_| Err(()), |_| Ok(()))
}

/// Wait for socket connection or timeout (ipv6)
pub(crate) fn wait_for_socket_connection_v6(port: u16, exp_retry_base: u64) -> Result<(), ()> {
    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port);
    let result = retry(Exponential::from_millis(exp_retry_base).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    result.map_or_else(|_| Err(()), |_| Ok(()))
}

/// Helper extension trait which allows to limit execution time for the futures.
/// It is helpful in tests to ensure that no future will ever get stuck forever.
pub(crate) trait TimeoutFutureExt<T>: Future<Output = T> + Sized {
    /// Returns a reasonable value that can be used as a future timeout with a certain
    /// degree of confidence that timeout won't be triggered by the test specifics.
    fn default_timeout() -> Duration {
        // If some future wasn't done in 60 seconds, it's either a poorly written test
        // or (most likely) a bug related to some future never actually being completed.
        const TIMEOUT_SECONDS: u64 = 60;
        Duration::from_secs(TIMEOUT_SECONDS)
    }

    /// Adds a fixed timeout to the future.
    fn with_default_timeout(self) -> Timeout<Self> {
        self.with_timeout(Self::default_timeout())
    }

    /// Adds a custom timeout to the future.
    fn with_timeout(self, timeout_value: Duration) -> Timeout<Self> {
        timeout(timeout_value, self)
    }
}

impl<T, U> TimeoutFutureExt<T> for U where U: Future<Output = T> + Sized {}
