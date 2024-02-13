use anyhow::{bail, Context, Result};
#[cfg(not(windows))]
use assert_cmd::prelude::*;
use chrono::{DateTime, FixedOffset};
#[cfg(feature = "websocket-notify")]
use jsonrpsee::{
    core::client::{Client, Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
#[cfg(not(windows))]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use once_cell::sync::Lazy;
use port_selector::Selector;
use predicates::prelude::*;
#[cfg(not(windows))]
use retry::delay::Fixed;
use retry::{delay::Exponential, retry};
use std::{
    env, fmt, fs,
    fs::File,
    future::Future,
    io::Write,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    str::FromStr,
    time::Duration,
};
use tokio::time::{timeout, Timeout};
use wait_timeout::ChildExt;
#[cfg(windows)]
use winapi::shared::winerror::ERROR_ACCESS_DENIED;

/// Binary name, which is different than the crate name.
pub(crate) const BIN_NAME: &str = "homestar";

/// Test-default ed25519 multihash.
pub(crate) const ED25519MULTIHASH: &str = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";
/// Test-default ed25519 multihash 2.
#[cfg(feature = "websocket-notify")]
pub(crate) const ED25519MULTIHASH2: &str = "12D3KooWK99VoVxNE7XzyBwXEzW7xhK7Gpv85r9F3V3fyKSUKPH5";
/// Test-default ed25519 multihash 3.
#[cfg(feature = "websocket-notify")]
pub(crate) const ED25519MULTIHASH3: &str = "12D3KooWJWoaqZhDaoEFshF7Rh1bpY9ohihFhzcW6d69Lr2NASuq";
/// Test-default ed25519 multihash 4.
#[cfg(feature = "websocket-notify")]
pub(crate) const ED25519MULTIHASH4: &str = "12D3KooWRndVhVZPCiQwHBBBdg769GyrPUW13zxwqQyf9r3ANaba";
/// Test-default ed25519 multihash 5.
#[cfg(feature = "websocket-notify")]
pub(crate) const ED25519MULTIHASH5: &str = "12D3KooWPT98FXMfDQYavZm66EeVjTqP9Nnehn1gyaydqV8L8BQw";
/// Test-default secp256k1 multihash.
pub(crate) const SECP256K1MULTIHASH: &str = "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc";

/// Return listener address.
pub(crate) fn listen_addr(port: u16) -> String {
    format!("/ip4/127.0.0.1/tcp/{port}")
}

/// Return multiaddr address.
#[cfg(feature = "websocket-notify")]
pub(crate) fn multiaddr(port: u16, hash: &str) -> String {
    format!("/ip4/127.0.0.1/tcp/{port}/p2p/{hash}")
}

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

/// [port_selector::Selector] wrapper for tests.
pub(crate) struct RpcSelector(pub(crate) Selector);

impl RpcSelector {
    pub(crate) fn new() -> Self {
        Self(Selector {
            check_tcp: true,
            check_udp: true,
            port_range: (10000, 15000),
            max_random_times: 100,
        })
    }

    /// Select a free port within the port range.
    pub(crate) fn select_free_port(&mut self) -> Result<u16> {
        port_selector::select_free_port(self.0).context("failed to select free port")
    }
}

/// Guard for a [Child] process.
#[derive(Debug)]
pub(crate) struct ChildGuard {
    guard: Option<Child>,
}

impl fmt::Display for ChildGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.guard {
            Some(child) => write!(f, "{}", child.id()),
            None => write!(f, "None"),
        }
    }
}

#[allow(dead_code)]
impl ChildGuard {
    /// Create a new [ChildGuard] from a [Child] process.
    pub(crate) fn new(child: Child) -> Self {
        Self { guard: Some(child) }
    }

    /// Take the [Child] process from the [ChildGuard].
    pub(crate) fn take(mut self) -> Child {
        self.guard.take().expect("Failed to take the `Child`")
    }

    /// Take the [Child] process from the [ChildGuard] and return the [ChildStdin].
    pub(crate) fn stdin(&mut self) -> Option<ChildStdin> {
        match &mut self.guard {
            Some(child) => child.stdin.take(),
            None => None,
        }
    }

    /// Take the [Child] process from the [ChildGuard] and return the [ChildStdout].
    pub(crate) fn stdout(&mut self) -> Option<ChildStdout> {
        match &mut self.guard {
            Some(child) => child.stdout.take(),
            None => None,
        }
    }

    /// Wait for the [Child] process to exit and return the [std::process::Output].
    pub(crate) fn wait_with_output(self) -> std::io::Result<std::process::Output> {
        self.take().wait_with_output()
    }

    /// Wait for the [Child] process to exit and return if successful.
    pub(crate) fn wait_with_result(self) -> Result<()> {
        let out = self.wait_with_output()?;

        if out.status.success() {
            Ok(())
        } else {
            bail!("{}", String::from_utf8_lossy(&out.stderr))
        }
    }
}

impl Drop for ChildGuard {
    #[cfg(windows)]
    fn drop(&mut self) {
        if let Some(mut child) = self.guard.take() {
            if matches!(child.try_wait(), Ok(None)) {
                if let Err(err) = child.kill() {
                    const ACCESS_DENIED: Option<i32> = Some(ERROR_ACCESS_DENIED as i32);
                    if !matches!(err.raw_os_error(), ACCESS_DENIED) {
                        eprintln!("Failed to clean up child process {}: {}", self, err);
                    }
                }

                // Sending a kill signal does NOT imply the process has exited. Wait for it to exit.
                let wait_res = child.wait();
                if let Ok(code) = wait_res.as_ref() {
                    eprintln!("Child process {} killed, exited with code {:?}", self, code);
                } else {
                    eprintln!(
                        "Failed to wait for child process {} that was terminated: {:?}",
                        self, wait_res
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    fn drop(&mut self) {
        if let Some(mut child) = self.guard.take() {
            // attempt to stop gracefully
            let pid = child.id();
            unsafe {
                libc::kill(libc::pid_t::from_ne_bytes(pid.to_ne_bytes()), libc::SIGTERM);
            }

            for _ in 0..10 {
                if child.try_wait().ok().flatten().is_some() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            if child.try_wait().ok().flatten().is_none() {
                // still alive? kill it with fire
                let _ = child.kill();
            }

            let _ = child.wait();
        }
    }
}

/// Stop the Homestar server/binary.
#[allow(dead_code)]
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
pub(crate) fn check_for_line_with(output: String, predicates: Vec<&str>) -> bool {
    output
        .split('\n')
        .map(|line| line_contains(line, &predicates))
        .any(|curr| curr)
}

#[cfg(feature = "websocket-notify")]
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
    let pid = std::fs::read_to_string("/tmp/homestar.pid")
        .expect("Should have a PID file")
        .trim()
        .parse::<u32>()
        .unwrap();

    rm_rf::ensure_removed("/tmp/homestar.pid").unwrap();
    signal::kill(Pid::from_raw(pid.try_into().unwrap()), Signal::SIGTERM).unwrap();
    retry(Fixed::from_millis(1000).take(5), || {
        Command::new(BIN.as_os_str())
            .arg("ping")
            .assert()
            .try_failure()
    })
    .unwrap();

    Ok(())
}

/// Remove sqlite database and associated temporary files
pub(crate) fn remove_db(name: &str) {
    let _ = fs::remove_file(name);
    let _ = fs::remove_file(format!("{name}-shm"));
    let _ = fs::remove_file(format!("{name}-wal"));
}

/// ProcInfo struct for tests, filled with randomized ports and DB name.
pub(crate) struct ProcInfo {
    pub(crate) metrics_port: u16,
    pub(crate) rpc_port: u16,
    pub(crate) ws_port: u16,
    pub(crate) db_path: PathBuf,
    pub(crate) listen_port: u16,
}

impl ProcInfo {
    pub(crate) fn new() -> Result<Self> {
        let uuid = &uuid::Uuid::new_v4();
        let name = format!("tests/fixtures/{}.db", uuid);

        let proc_info = Self {
            metrics_port: port_selector::random_free_tcp_port()
                .ok_or(anyhow::anyhow!("failed to select free port for metrics"))?,
            rpc_port: RpcSelector::new().select_free_port()?,
            ws_port: port_selector::random_free_port().ok_or(anyhow::anyhow!(
                "failed to select free port for JSON-RPC webserver"
            ))?,
            db_path: PathBuf::from_str(&name)?,
            listen_port: port_selector::random_free_port().ok_or(anyhow::anyhow!(
                "failed to select free port for listen address"
            ))?,
        };

        Ok(proc_info)
    }
}

impl Drop for ProcInfo {
    fn drop(&mut self) {
        if let Some(path) = self.db_path.to_str() {
            remove_db(path);
        };
    }
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

/// Client and subscription.
#[cfg(feature = "websocket-notify")]
pub(crate) struct WsClientSub {
    #[allow(dead_code)]
    client: Client,
    sub: Subscription<Vec<u8>>,
}

#[cfg(feature = "websocket-notify")]
impl WsClientSub {
    pub(crate) fn sub(&mut self) -> &mut Subscription<Vec<u8>> {
        &mut self.sub
    }
}

/// Helper function to subscribe to network events
/// Note that the client must not be dropped of the sub will return only None.
#[cfg(feature = "websocket-notify")]
pub(crate) async fn subscribe_network_events(ws_port: u16) -> WsClientSub {
    const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
    const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

    let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port);
    tokio_tungstenite::connect_async(ws_url.clone())
        .await
        .unwrap();

    let client = WsClientBuilder::default()
        .build(ws_url.clone())
        .await
        .unwrap();

    let sub: Subscription<Vec<u8>> = client
        .subscribe(
            SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            rpc_params![],
            UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
        )
        .await
        .unwrap();

    WsClientSub { client, sub }
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

/// Config wrapper for tests.
#[derive(Debug)]
pub(crate) struct TestConfig {
    filename: String,
    toml_config: toml::Table,
}

#[allow(dead_code)]
impl TestConfig {
    /// Save the [TestConfig] to the config directory.
    pub(crate) fn save_fixture(&self) -> Result<()> {
        let file_path = env::current_dir()?;
        let mut write_file = File::create(file_path.join(&self.filename))?;
        write_file.write_all(toml::to_string(&self.toml_config).unwrap().as_bytes())?;
        Ok(())
    }

    /// Get the name of the [TestConfig].
    pub(crate) fn filename(&self) -> &String {
        &self.filename
    }

    /// Get the toml config of the [TestConfig].
    pub(crate) fn config(&self) -> &toml::Table {
        &self.toml_config
    }

    /// Create a new [TestConfig].
    pub(crate) fn new(filename: String, toml: toml::Table) -> Self {
        TestConfig {
            filename: filename.to_string(),
            toml_config: toml,
        }
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        let file_path = env::current_dir().unwrap();
        fs::remove_file(file_path.join(&self.filename)).unwrap();
    }
}
#[macro_export]
macro_rules! make_config {
    // For tests where all you want to do is write toml.
    ($toml:expr) => {{
        let uuid = uuid::Uuid::new_v4();
        let name = format!("tests/fixtures/{}.toml", uuid);
        let toml = $toml.parse::<toml::Table>().unwrap();
        let test_config = $crate::utils::TestConfig::new(name.to_string(), toml);
        test_config.save_fixture().unwrap();
        test_config
    }};
    // For some finer control over the config.
    ($name:expr, $toml:expr) => {{
        let name = format!("tests/fixtures/{}.toml", $name);
        let toml = $toml.parse::<toml::Table>().unwrap();
        let test_config = $crate::utils::TestConfig::new(name.to_string(), toml);
        test_config.save_fixture().unwrap();
        test_config
    }};
}
