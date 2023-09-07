//! Node metrics, including system, process, network, and database information

use anyhow::{anyhow, Context, Result};
use metrics::{describe_gauge, Unit};
use std::time::Duration;
use sysinfo::{get_current_pid, CpuExt, ProcessExt, System, SystemExt};
use tracing::{info, warn};

/// Create and describe gauges for node metrics.
pub(crate) fn describe() {
    describe_gauge!(
        "process_cpu_usage_percentage",
        Unit::Percent,
        "The CPU percentage used."
    );
    describe_gauge!(
        "process_virtual_memory_bytes",
        Unit::Bytes,
        "The virtual memory size in bytes."
    );
    describe_gauge!("process_memory_bytes", Unit::Bytes, "Memory size in bytes.");
    describe_gauge!(
        "process_disk_total_written_bytes",
        Unit::Bytes,
        "The total bytes written to disk."
    );
    describe_gauge!(
        "process_disk_written_bytes",
        Unit::Bytes,
        "The bytes written to disk."
    );
    describe_gauge!(
        "process_disk_total_read_bytes",
        Unit::Bytes,
        "Total bytes Read from disk."
    );
    describe_gauge!(
        "process_disk_read_bytes",
        Unit::Bytes,
        "The bytes read from disk."
    );
    describe_gauge!(
        "process_disk_written_bytes",
        Unit::Bytes,
        "The bytes written to disk."
    );
    describe_gauge!(
        "process_uptime_seconds",
        Unit::Seconds,
        "How much time the process has been running in seconds."
    );
}

/// Collect node metrics on a settings-defined interval.
pub(crate) async fn collect_metrics(interval: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval));

    loop {
        interval.tick().await;
        let sys_info = System::new();
        if let Err(err) = get_stats(sys_info).await {
            warn!(
                subject = "metrics.process_collection",
                category = "metrics",
                "failure to get process statistics {:#?}",
                err
            );
        }
    }
}

async fn get_stats(mut sys: System) -> Result<()> {
    let pid = get_current_pid().map_err(|e| anyhow!("no process pid found {}", e))?;

    let is_process_refreshed = sys.refresh_process(pid);
    sys.refresh_cpu();

    if is_process_refreshed {
        let proc = sys.process(pid).context("no process associated with pid")?;
        let cpus = num_cpus::get();
        let disk = proc.disk_usage();

        // cpu-usage divided by # of cores.
        metrics::gauge!(
            "process_cpu_usage_percentage",
            f64::from(sys.global_cpu_info().cpu_usage() / (cpus as f32))
        );

        // The docs for sysinfo indicate that `virtual_memory`
        // returns in KB, but that is incorrect.
        // See this issue: https://github.com/GuillaumeGomez/sysinfo/issues/428#issuecomment-774098021
        // And this PR: https://github.com/GuillaumeGomez/sysinfo/pull/430/files
        metrics::gauge!(
            "process_virtual_memory_bytes",
            (proc.virtual_memory()) as f64
        );
        metrics::gauge!("process_memory_bytes", (proc.memory() * 1_000) as f64);
        metrics::gauge!("process_uptime_seconds", proc.run_time() as f64);
        metrics::gauge!(
            "process_disk_total_written_bytes",
            disk.total_written_bytes as f64,
        );
        metrics::gauge!("process_disk_written_bytes", disk.written_bytes as f64);
        metrics::gauge!(
            "process_disk_total_read_bytes",
            disk.total_read_bytes as f64,
        );
        metrics::gauge!("process_disk_read_bytes", disk.read_bytes as f64);
    } else {
        info!(
            subject = "metrics.process_collection",
            category = "metrics",
            "failed to refresh process information, metrics may show old results"
        );
    }

    Ok(())
}
