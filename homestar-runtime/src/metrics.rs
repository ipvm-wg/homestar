//! Collect metrics and setup recorder and scrape endpoint.

use crate::settings;
use anyhow::Result;
use metrics_exporter_prometheus::PrometheusHandle;
#[cfg(feature = "monitoring")]
use tokio::runtime::Handle;

mod exporter;
#[cfg(feature = "monitoring")]
mod node;

/// Start metrics collection and setup scrape endpoint.
#[cfg(feature = "monitoring")]
pub(crate) async fn start(
    monitor_settings: &settings::Monitoring,
    network_settings: &settings::Network,
) -> Result<PrometheusHandle> {
    let metrics_hdl = exporter::setup_metrics_recorder(network_settings)?;

    // Spawn tick-driven process collection task
    let handle = Handle::current();
    handle.spawn(node::collect_metrics(
        monitor_settings.process_collector_interval,
    ));

    Ok(metrics_hdl)
}

#[cfg(not(feature = "monitoring"))]
pub(crate) async fn start(network_settings: &settings::Network) -> Result<PrometheusHandle> {
    let metrics_hdl = exporter::setup_metrics_recorder(network_settings)?;
    Ok(metrics_hdl)
}
