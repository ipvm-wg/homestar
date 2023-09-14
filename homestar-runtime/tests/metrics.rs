use crate::utils::{kill_homestar_process, stop_homestar};
use anyhow::Result;
use assert_cmd::crate_name;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use retry::{delay::Fixed, retry, OperationResult};
use serial_test::file_serial;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread, time,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(crate_name!()));
const METRICS_URL: &str = "http://localhost:4000";

#[test]
#[file_serial]
fn test_metrics_serial() -> Result<()> {
    fn sample_metrics() -> prometheus_parse::Value {
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
            .expect("Could not find system_used_memory_bytes metric")
            .value
            .to_owned()
    }

    let _ = stop_homestar();

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/test_node2/config/settings.toml")
        .arg("--db")
        .arg("homestar.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let sample1 = sample_metrics();
    thread::sleep(time::Duration::from_millis(600));
    let sample2 = sample_metrics();

    assert_ne!(sample1, sample2);

    let _ = stop_homestar();
    let _ = kill_homestar_process();

    Ok(())
}
