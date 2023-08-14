//! Daemonize the Homestar runtime.

use anyhow::Result;
use std::path::PathBuf;

#[cfg(not(windows))]
const PID_FILE: &str = "homestar.pid";

/// Start the Homestar runtime as a daemon.
#[cfg(not(windows))]
pub fn start(dir: PathBuf) -> Result<()> {
    daemonize::Daemonize::new()
        .working_directory(dir.canonicalize()?)
        .pid_file(PID_FILE)
        .start()?;

    Ok(())
}

/// Start the Homestar runtime as a daemon.
#[cfg(windows)]
pub fn start(_dir: PathBuf) -> Result<()> {
    Err(anyhow::anyhow!("Daemonizing is not supported on Windows"))
}
