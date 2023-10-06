use crate::utils::{kill_homestar, stop_homestar, BIN_NAME};
use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use retry::{delay::Fixed, retry, OperationResult};
use serial_test::file_serial;
use std::{
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const METRICS_URL: &str = "http://localhost:4004";

#[test]
#[file_serial]
fn test_metrics_serial() -> Result<()> {
    fn sample_metrics() -> Option<prometheus_parse::Value> {
        let body = retry(
            Fixed::from_millis(500).take(2),
            || match reqwest::blocking::get(METRICS_URL) {
                Ok(response) => match response.status() {
                    StatusCode::OK => OperationResult::Ok(response.text()),
                    _ => OperationResult::Err("Metrics server failed to serve metrics"),
                },
                Err(_) => OperationResult::Retry("Metrics server not available"),
            },
        )
        .unwrap()
        .expect("Metrics server failed to serve metrics");

        let lines: Vec<_> = body.lines().map(|s| Ok(s.to_owned())).collect();
        let metrics =
            prometheus_parse::Scrape::parse(lines.into_iter()).expect("Unable to parse metrics");

        metrics
            .samples
            .iter()
            .find(|sample| sample.metric.as_str() == "system_used_memory_bytes")
            .and_then(|sample| Some(sample.value.to_owned()))
    }

    let _ = stop_homestar();

    let mut homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/metrics_node/config/settings.toml")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4004);
    let result = retry(Fixed::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        homestar_proc.kill().unwrap();
        panic!("Homestar server/runtime failed to start in time");
    }

    // Try metrics server until the target metric is available
    let sample1 = retry(Fixed::from_millis(100).take(3), || {
        if let Some(sample) = sample_metrics() {
            OperationResult::Ok(sample)
        } else {
            OperationResult::Retry("Could not find system_used_memory_bytes metric")
        }
    })
    .unwrap();

    let sample2 = retry(Fixed::from_millis(500).take(3), || {
        let sample2 = sample_metrics().unwrap();
        if sample1 != sample2 {
            OperationResult::Ok(sample2)
        } else {
            OperationResult::Retry("Samples are the same")
        }
    })
    .unwrap();

    assert_ne!(sample1, sample2);

    let _ = kill_homestar(homestar_proc, None);
    let _ = stop_homestar();

    Ok(())
}
