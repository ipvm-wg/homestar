//! Metrics Prometheus recorder.

use crate::{metrics::node, settings};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Set up Prometheus buckets for matched metrics and install recorder.
pub(crate) fn setup_metrics_recorder(settings: &settings::Monitoring) -> anyhow::Result<()> {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    let socket = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        settings.metrics_port,
    );

    #[cfg(feature = "monitoring")]
    node::describe();

    // swarm_events::describe();
    // homestar_events::describe();

    // let mut registry = Registry::default();
    // let metrics = Metrics::new(&mut registry);
    // println!("LIBP2P METRICS {:?}", metrics);

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Suffix("_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )?
        .with_http_listener(socket)
        .install()
        .expect("failed to install recorder/exporter");

    Ok(())
}
