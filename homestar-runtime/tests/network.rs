use crate::utils::{kill_homestar, stop_homestar};
use anyhow::Result;
use assert_cmd::crate_name;
use once_cell::sync::Lazy;
use serial_test::file_serial;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(crate_name!()));

#[test]
#[file_serial]
fn test_libp2p_connect() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/test_node1/config/settings.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let homestar_proc2 = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg("tests/test_node2/config/settings.toml")
        .arg("--db")
        .arg("homestar2.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let _ = kill_homestar(homestar_proc1);
    let _ = kill_homestar(homestar_proc2);
    let _ = stop_homestar();

    Ok(())
}
