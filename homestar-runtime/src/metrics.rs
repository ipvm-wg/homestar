//! Collect metrics and monitoring endpoint for Prometheus

use crate::settings;
use anyhow::Result;
use tokio::runtime::Handle;

mod exporter;
mod node;

pub(crate) async fn start(settings: &settings::Monitoring) -> Result<()> {
    let handle = Handle::current();
    exporter::setup_metrics_recorder(settings)?;

    // Spawn tick-driven process collection task
    #[cfg(feature = "monitoring")]
    handle.spawn(node::collect_metrics(settings.process_collector_interval));

    Ok(())
}
