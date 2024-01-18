use crate::utils::{wait_for_socket_connection, ChildGuard, FileGuard, BIN_NAME};
use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use retry::{delay::Exponential, retry, OperationResult};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const METRICS_URL: &str = "http://localhost:4020";

#[test]
fn test_metrics_integration() -> Result<()> {
    fn sample_metrics() -> Option<prometheus_parse::Value> {
        let body = retry(
            Exponential::from_millis(500).take(20),
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
            .find(|sample| sample.metric.as_str() == "homestar_system_used_memory_bytes")
            .map(|sample| sample.value.to_owned())
    }

    const DB: &str = "test_metrics_integration.db";
    let _db_guard = FileGuard::new(DB);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_metrics.toml")
        .arg("--db")
        .arg(DB)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection(4020, 100).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Try metrics server until the target metric is available
    let sample1 = retry(Exponential::from_millis(100).take(5), || {
        if let Some(sample) = sample_metrics() {
            OperationResult::Ok(sample)
        } else {
            OperationResult::Retry("Could not find system_used_memory_bytes metric")
        }
    })
    .unwrap();

    let sample2 = retry(Exponential::from_millis(500).take(10), || {
        let sample2 = sample_metrics().unwrap();
        if sample1 != sample2 {
            OperationResult::Ok(sample2)
        } else {
            OperationResult::Retry("Samples are the same")
        }
    });

    if sample2.is_err() {
        panic!("Could not generate a diff in sample(s)");
    }

    assert_ne!(sample1, sample2.unwrap());

    Ok(())
}
