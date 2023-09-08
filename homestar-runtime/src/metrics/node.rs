//! Node metrics, including system, process, network, and database information

use anyhow::Result;
use metrics::{describe_gauge, Unit};
use std::time::Duration;
use sysinfo::{CpuRefreshKind, Disk, DiskExt, ProcessRefreshKind, RefreshKind, System, SystemExt};
use tracing::{info, warn};

/// Create and describe gauges for node metrics.
pub(crate) fn describe() {
    // System metrics
    describe_gauge!(
        "system_available_memory",
        Unit::Bytes,
        "The amount of available memory."
    );
    describe_gauge!(
        "system_used_memory",
        Unit::Bytes,
        "The amount of used memory."
    );
    describe_gauge!(
        "system_free_swap",
        Unit::Bytes,
        "The amount of free swap space."
    );
    describe_gauge!(
        "system_used_swap",
        Unit::Bytes,
        "The amount of used swap space."
    );
    describe_gauge!(
        "system_disk_available_space",
        Unit::Bytes,
        "The total amount of available disk space."
    );
    describe_gauge!("system_uptime", Unit::Seconds, "The total system uptime.");
    describe_gauge!(
        "system_load_average",
        Unit::Percent,
        "The load average over a five minute interval."
    );

    // describe_gauge!(
    //     "process_cpu_usage_percentage",
    //     Unit::Percent,
    //     "The CPU percentage used."
    // );
    // describe_gauge!(
    //     "process_virtual_memory_bytes",
    //     Unit::Bytes,
    //     "The virtual memory size in bytes."
    // );
    // describe_gauge!("process_memory_bytes", Unit::Bytes, "Memory size in bytes.");
    // describe_gauge!(
    //     "process_disk_total_written_bytes",
    //     Unit::Bytes,
    //     "The total bytes written to disk."
    // );
    // describe_gauge!(
    //     "process_disk_written_bytes",
    //     Unit::Bytes,
    //     "The bytes written to disk."
    // );
    // describe_gauge!(
    //     "process_disk_total_read_bytes",
    //     Unit::Bytes,
    //     "Total bytes Read from disk."
    // );
    // describe_gauge!(
    //     "process_disk_read_bytes",
    //     Unit::Bytes,
    //     "The bytes read from disk."
    // );
    // describe_gauge!(
    //     "process_disk_written_bytes",
    //     Unit::Bytes,
    //     "The bytes written to disk."
    // );
    // describe_gauge!(
    //     "process_uptime_seconds",
    //     Unit::Seconds,
    //     "How much time the process has been running in seconds."
    // );
}

/// Collect node metrics on a settings-defined interval.
pub(crate) async fn collect_metrics(interval: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval));

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

    // System metrics
    metrics::gauge!("system_available_memory", sys.available_memory() as f64);
    metrics::gauge!("system_used_memory", sys.used_memory() as f64);
    metrics::gauge!("system_free_swap", sys.free_swap() as f64);
    metrics::gauge!("system_used_swap", sys.used_swap() as f64);
    metrics::gauge!(
        "system_disk_available_space",
        compute_available_disk_space(sys.disks()) as f64
    );
    metrics::gauge!("system_uptime", sys.uptime() as f64);
    metrics::gauge!("system_load_average", sys.load_average().five);

    // Process metrics
    // let pid = get_current_pid().map_err(|e| anyhow!("no process pid found {}", e))?;

    // let is_process_refreshed = sys.refresh_process(pid);
    // sys.refresh_cpu();

    // if is_process_refreshed {
    //     let proc = sys.process(pid).context("no process associated with pid")?;
    //     let cpus = num_cpus::get();
    //     let disk = proc.disk_usage();

    //     // cpu-usage divided by # of cores.
    //     metrics::gauge!(
    //         "process_cpu_usage_percentage",
    //         f64::from(sys.global_cpu_info().cpu_usage() / (cpus as f32))
    //     );

    // The docs for sysinfo indicate that `virtual_memory`
    // returns in KB, but that is incorrect.
    // See this issue: https://github.com/GuillaumeGomez/sysinfo/issues/428#issuecomment-774098021
    // And this PR: https://github.com/GuillaumeGomez/sysinfo/pull/430/files
    //     metrics::gauge!(
    //         "process_virtual_memory_bytes",
    //         (proc.virtual_memory()) as f64
    //     );
    //     metrics::gauge!("process_memory_bytes", (proc.memory() * 1_000) as f64);
    //     metrics::gauge!("process_uptime_seconds", proc.run_time() as f64);
    //     metrics::gauge!(
    //         "process_disk_total_written_bytes",
    //         disk.total_written_bytes as f64,
    //     );
    //     metrics::gauge!("process_disk_written_bytes", disk.written_bytes as f64);
    //     metrics::gauge!(
    //         "process_disk_total_read_bytes",
    //         disk.total_read_bytes as f64,
    //     );
    //     metrics::gauge!("process_disk_read_bytes", disk.read_bytes as f64);
    // } else {
    //     info!(
    //         subject = "metrics.process_collection",
    //         category = "metrics",
    //         "failed to refresh process information, metrics may show old results"
    //     );
    // }

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
