//! Metrics Prometheus recorder.

#[cfg(feature = "monitoring")]
use crate::metrics::node;
use crate::settings;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use metrics_util::layers::{PrefixLayer, Stack};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::runtime::Handle;

/// Set up Prometheus buckets for matched metrics and install recorder.
#[cfg(not(test))]
pub(crate) fn setup_metrics_recorder(
    settings: &settings::Network,
) -> anyhow::Result<PrometheusHandle> {
    setup_metrics_recorder_inner(settings.metrics.port)
}

/// Set up Prometheus buckets for matched metrics and install recorder.
#[cfg(test)]
pub(crate) fn setup_metrics_recorder(
    _settings: &settings::Network,
) -> anyhow::Result<PrometheusHandle> {
    let port = crate::test_utils::ports::get_port() as u16;
    setup_metrics_recorder_inner(port)
}

fn setup_metrics_recorder_inner(port: u16) -> anyhow::Result<PrometheusHandle> {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);

    let (recorder, exporter) = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Suffix("_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )?
        .with_http_listener(socket)
        .build()
        .expect("failed to install recorder/exporter");

    let hdl = recorder.handle();
    let rt_hdl = Handle::current();
    rt_hdl.spawn(exporter);

    Stack::new(recorder)
        .push(PrefixLayer::new("homestar"))
        .install()?;

    #[cfg(feature = "monitoring")]
    node::describe();

    Ok(hdl)
}
