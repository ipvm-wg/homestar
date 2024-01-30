use crate::{
    make_config,
    utils::{wait_for_socket_connection, ChildGuard, ProcInfo, BIN_NAME},
};
use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use retry::{delay::Exponential, retry, OperationResult};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[serial_test::parallel]
fn test_metrics_integration() -> Result<()> {
    fn sample_metrics(port: u16) -> Option<prometheus_parse::Value> {
        let url = format!("http://localhost:{}/metrics", port);
        let body = retry(
            Exponential::from_millis(500).take(20),
            || match reqwest::blocking::get(url.as_str()) {
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

    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);
    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection(metrics_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    // Try metrics server until the target metric is available
    let sample1 = retry(Exponential::from_millis(100).take(5), || {
        if let Some(sample) = sample_metrics(metrics_port) {
            OperationResult::Ok(sample)
        } else {
            OperationResult::Retry("Could not find system_used_memory_bytes metric")
        }
    })
    .unwrap();

    let sample2 = retry(Exponential::from_millis(500).take(10), || {
        let sample2 = sample_metrics(metrics_port).unwrap();
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
