//! Node metrics, including system, process, network, and database information

use crate::Db;
use anyhow::{anyhow, Context, Result};
use metrics::{describe_counter, describe_gauge, Unit};
use std::time::Duration;
use sysinfo::{
    get_current_pid, CpuRefreshKind, Disk, DiskExt, NetworkExt, Networks, NetworksExt, ProcessExt,
    ProcessRefreshKind, RefreshKind, System, SystemExt,
};
use tracing::{info, warn};

/// Create and describe gauges for node metrics.
pub(crate) fn describe() {
    // System metrics
    describe_gauge!(
        "system_available_memory_bytes",
        Unit::Bytes,
        "The amount of available memory."
    );
    describe_gauge!(
        "system_used_memory_bytes",
        Unit::Bytes,
        "The amount of used memory."
    );
    describe_gauge!(
        "system_free_swap_bytes",
        Unit::Bytes,
        "The amount of free swap space."
    );
    describe_gauge!(
        "system_used_swap_bytes",
        Unit::Bytes,
        "The amount of used swap space."
    );
    describe_gauge!(
        "system_disk_available_space_bytes",
        Unit::Bytes,
        "The total amount of available disk space."
    );
    describe_gauge!(
        "system_uptime_seconds",
        Unit::Seconds,
        "The total system uptime."
    );
    describe_gauge!(
        "system_load_average_percentage",
        Unit::Percent,
        "The load average over a five minute interval."
    );

    // Process metrics
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
        "The bytes written to disk since last refresh."
    );
    describe_gauge!(
        "process_disk_total_read_bytes",
        Unit::Bytes,
        "Total bytes read from disk."
    );
    describe_gauge!(
        "process_disk_read_bytes",
        Unit::Bytes,
        "The bytes read from disk since last refresh."
    );
    describe_counter!(
        "process_uptime_seconds",
        Unit::Seconds,
        "How much time the process has been running in seconds."
    );

    // Network metrics
    describe_counter!(
        "network_transmitted_bytes",
        Unit::Bytes,
        "The bytes transmitted since last refresh."
    );
    describe_counter!(
        "network_received_bytes",
        Unit::Bytes,
        "The bytes received since last refresh."
    );

    // Databsae metrics
    describe_counter!(
        "database_size_bytes",
        Unit::Bytes,
        "The sqlite database size."
    );
}

/// Collect node metrics on a settings-defined interval.
pub(crate) async fn collect_metrics(interval: u64) {
    let mut interval = tokio::time::interval(Duration::from_millis(interval));

    // Log static system info
    log_static_info();

    loop {
        interval.tick().await;
        let sys_info = System::new_with_specifics(
            RefreshKind::new()
                .with_components()
                .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                .with_memory()
                .with_disks()
                .with_processes(ProcessRefreshKind::everything().without_user()),
        );
        if let Err(err) = collect_stats(sys_info).await {
            warn!(
                subject = "metrics.process_collection",
                category = "metrics",
                "failure to get process statistics {:#?}",
                err
            );
        }
    }
}

async fn collect_stats(sys: System) -> Result<()> {
    fn compute_available_disk_space(disks: &[Disk]) -> u64 {
        disks
            .iter()
            .fold(0, |acc, disk| acc + disk.available_space())
    }
    fn compute_network_transmitted(networks: &Networks) -> u64 {
        networks
            .iter()
            .fold(0, |acc, interface| acc + interface.1.transmitted())
    }
    fn compute_network_received(networks: &Networks) -> u64 {
        networks
            .iter()
            .fold(0, |acc, interface| acc + interface.1.received())
    }
    async fn compute_database_size() -> Option<f64> {
        if let Ok(size) = Db::size().await {
            Some(size.get_value())
        } else {
            None
        }
    }

    // System metrics
    metrics::gauge!(
        "system_available_memory_bytes",
        sys.available_memory() as f64
    );
    metrics::gauge!("system_used_memory_bytes", sys.used_memory() as f64);
    metrics::gauge!("system_free_swap_bytes", sys.free_swap() as f64);
    metrics::gauge!("system_used_swap_bytes", sys.used_swap() as f64);
    metrics::gauge!(
        "system_disk_available_space_bytes",
        compute_available_disk_space(sys.disks()) as f64
    );
    metrics::gauge!("system_uptime_seconds", sys.uptime() as f64);
    metrics::gauge!("system_load_average_percentage", sys.load_average().five);

    // Process metrics
    let pid = get_current_pid().map_err(|e| anyhow!("no process pid found {}", e))?;
    let proc = sys.process(pid).context("no process associated with pid")?;

    let cpus = sys.physical_core_count().unwrap_or(1);
    metrics::gauge!(
        "process_cpu_usage_percentage",
        f64::from(proc.cpu_usage() / (cpus as f32))
    );
    metrics::gauge!(
        "process_virtual_memory_bytes",
        (proc.virtual_memory()) as f64
    );
    metrics::gauge!("process_memory_bytes", (proc.memory()) as f64);

    let process_disk_usage = proc.disk_usage();
    metrics::gauge!(
        "process_disk_total_written_bytes",
        process_disk_usage.total_written_bytes as f64,
    );
    metrics::gauge!(
        "process_disk_written_bytes",
        process_disk_usage.written_bytes as f64
    );
    metrics::gauge!(
        "process_disk_total_read_bytes",
        process_disk_usage.total_read_bytes as f64,
    );
    metrics::gauge!(
        "process_disk_read_bytes",
        process_disk_usage.read_bytes as f64
    );

    metrics::gauge!("process_uptime_seconds", proc.run_time() as f64);

    // Network metrics
    let networks = sys.networks();
    metrics::gauge!(
        "network_transmitted_bytes",
        compute_network_transmitted(networks) as f64
    );
    metrics::gauge!(
        "network_received_bytes",
        compute_network_received(networks) as f64
    );

    // Database metrics
    if let Some(database_size) = compute_database_size().await {
        metrics::gauge!("database_size_bytes", database_size);
    }

    Ok(())
}

// Log static system information
fn log_static_info() {
    fn compute_total_disk_space(disks: &[Disk]) -> u64 {
        disks.iter().fold(0, |acc, disk| acc + disk.total_space())
    }

    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_components()
            .with_cpu(CpuRefreshKind::new())
            .with_memory()
            .with_disks(),
    );

    info!(
        subject = "metrics",
        category = "homestar_runtime",
        "Running on {} at kernel version {}",
        sys.long_os_version().unwrap_or(String::from("Unknown")),
        sys.kernel_version().unwrap_or(String::from("Unknown")),
    );
    info!(
        subject = "metrics",
        category = "homestar_runtime",
        "System booted at UNIX time {}",
        sys.boot_time(),
    );
    info!(
        subject = "metrics",
        category = "homestar_runtime",
        "Physical core count of all the CPUs: {}",
        sys.physical_core_count().unwrap_or(0),
    );
    info!(
        subject = "metrics",
        category = "homestar_runtime",
        "Total memory on machine: {}",
        sys.total_memory(),
    );
    info!(
        subject = "metrics",
        category = "homestar_runtime",
        "Total swap on machine: {}",
        sys.total_swap(),
    );
    info!(
        subject = "metrics",
        category = "homestar_runtime",
        "Total disk space on machine: {}",
        compute_total_disk_space(sys.disks()) as f64
    );
}
