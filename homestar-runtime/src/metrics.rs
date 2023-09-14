//! Collect metrics and setup recorder and scrape endpoint.

use crate::settings;
use anyhow::Result;
use tokio::runtime::Handle;

mod exporter;
mod node;

/// Start metrics collection and setup scrape endpoint.
pub(crate) async fn start(settings: &settings::Monitoring) -> Result<()> {
    let handle = Handle::current();
    exporter::setup_metrics_recorder(settings)?;

    // Spawn tick-driven process collection task
    #[cfg(feature = "monitoring")]
    handle.spawn(node::collect_metrics(settings.process_collector_interval));

    Ok(())
}
