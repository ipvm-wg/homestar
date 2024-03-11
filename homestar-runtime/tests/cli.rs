#[cfg(not(windows))]
use crate::utils::kill_homestar_daemon;
#[cfg(feature = "test-utils")]
use crate::utils::wait_for_asserts;
use crate::{
    make_config,
    utils::{
        wait_for_socket_connection, wait_for_socket_connection_v6, ChildGuard, ProcInfo, BIN_NAME,
        ED25519MULTIHASH,
    },
};
use anyhow::Result;
use assert_cmd::prelude::*;
use homestar_runtime::Settings;
#[cfg(feature = "test-utils")]
use homestar_runtime::{db::Database, Db};
#[cfg(feature = "test-utils")]
use libipld::Cid;
use once_cell::sync::Lazy;
use predicates::prelude::*;
#[cfg(feature = "test-utils")]
use std::str::FromStr;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));

#[test]
#[serial_test::parallel]
fn test_help_integration() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("ping"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("help"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("init"));

    Command::new(BIN.as_os_str())
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("ping"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("help"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("init"));

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_version_integration() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "{} {}",
            BIN_NAME,
            env!("CARGO_PKG_VERSION")
        )));

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_server_not_running_integration() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("ping")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

    Command::new(BIN.as_os_str())
        .arg("stop")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("server was already shutdown")
                    .or(predicate::str::contains("No connection could be made"))),
        );

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_server_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.keypair_config]
        existing = {{ key_type = "ed25519", path = "./fixtures/__testkey_ed25519.pem" }}
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        host = "::1"
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );

    let config = make_config!(toml);

    Command::new(BIN.as_os_str())
        .env("RUST_BACKTRACE", "0")
        .arg("start")
        .arg("-db")
        .arg(&proc_info.db_path)
        .assert()
        .failure();

    let homestar_proc = Command::new(BIN.as_os_str())
        .env("RUST_BACKTRACE", "0")
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .arg("-p")
        .arg(rpc_port.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("::1"))
        .stdout(predicate::str::contains("pong"));

    Command::new(BIN.as_os_str())
        .arg("node")
        .arg("--host")
        .arg("::1")
        .arg("-p")
        .arg(rpc_port.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains(ED25519MULTIHASH.to_string()));

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("::1")
        .arg("-p")
        .arg(port_selector::random_free_port().unwrap().to_string())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Connection refused")
                .or(predicate::str::contains("No connection could be made")),
        );

    Ok(())
}

#[test]
#[serial_test::parallel]
#[cfg(feature = "test-utils")]
fn test_workflow_run_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );
    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg(rpc_port.to_string())
        .arg("tests/fixtures/test-workflow-add-one.json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ipfs://bafybeia32q3oy6u47x624rmsmgrrlpn7ulruissmz5z2ap6alv7goe7h3q",
        ))
        .stdout(predicate::str::contains("num_tasks"))
        .stdout(predicate::str::contains("progress_count"));

    // run another one of the same!
    Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg(rpc_port.to_string())
        .arg("tests/fixtures/test-workflow-add-one.json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ipfs://bafybeia32q3oy6u47x624rmsmgrrlpn7ulruissmz5z2ap6alv7goe7h3q",
        ))
        .stdout(predicate::str::contains("num_tasks"))
        .stdout(predicate::str::contains("progress_count"));

    Ok(())
}

#[test]
#[serial_test::parallel]
#[cfg(feature = "test-utils")]
fn test_workflow_run_integration_nonced() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let workflow_cid = "bafyrmicbtl7g4zrbarazdbjnk2gxxbbzin3iaf6y2zs6va5auqiyhu5m2e";
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );
    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection_v6(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("run")
        .arg("-p")
        .arg(rpc_port.to_string())
        .arg("tests/fixtures/test-workflow-add-one-nonced.json")
        .assert()
        .success();

    let settings = Settings::load_from_file(PathBuf::from(config.filename())).unwrap();
    let cid = Cid::from_str(workflow_cid).unwrap();
    let db = Db::setup_connection_pool(
        settings.node(),
        Some(proc_info.db_path.display().to_string()),
    )
    .expect("Failed to connect to node two database");

    wait_for_asserts(500, || {
        let (name, info) = Db::get_workflow_info(cid, &mut db.conn().unwrap()).unwrap();
        name.unwrap().as_str() == workflow_cid
            && info.progress().len() == 2
            && info.progress_count() == 2
    })
    .unwrap();

    Ok(())
}

#[test]
#[serial_test::parallel]
#[cfg(not(windows))]
fn test_daemon_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        host = "127.0.0.1"
        port = {rpc_port}
        server_timeout = 300
        [node.network.webserver]
        port = {ws_port}
        "#
    );
    let config = make_config!(toml);

    Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("-d")
        .env("DATABASE_URL", &proc_info.db_path)
        .stdout(Stdio::piped())
        .assert()
        .success();

    if wait_for_socket_connection(rpc_port, 1000).is_err() {
        kill_homestar_daemon().unwrap();
        panic!("Homestar server/runtime failed to start in time");
    }

    kill_homestar_daemon().unwrap();
    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_server_v4_integration() -> Result<()> {
    let proc_info = ProcInfo::new().unwrap();
    let rpc_port = proc_info.rpc_port;
    let metrics_port = proc_info.metrics_port;
    let ws_port = proc_info.ws_port;
    let toml = format!(
        r#"
        [node]
        [node.network.libp2p.mdns]
        enable = false
        [node.network.metrics]
        port = {metrics_port}
        [node.network.rpc]
        host = "127.0.0.1"
        port = {rpc_port}
        [node.network.webserver]
        port = {ws_port}
        "#
    );
    let config = make_config!(toml);

    let homestar_proc = Command::new(BIN.as_os_str())
        .arg("start")
        .arg("-c")
        .arg(config.filename())
        .arg("--db")
        .arg(&proc_info.db_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let _proc_guard = ChildGuard::new(homestar_proc);

    if wait_for_socket_connection(rpc_port, 1000).is_err() {
        panic!("Homestar server/runtime failed to start in time");
    }

    Command::new(BIN.as_os_str())
        .arg("ping")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(rpc_port.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1"))
        .stdout(predicate::str::contains("pong"));

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_init_dry_run() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("init")
        .arg("--dry-run")
        .arg("--no-input")
        .arg("--key-type")
        .arg("ed25519")
        .arg("--key-seed")
        .arg("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
        .assert()
        .success()
        .stdout(predicate::str::contains("key_type = \"ed25519\""))
        .stdout(predicate::str::contains(
            "seed = \"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\"",
        ));

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_init_interactive_no_tty() -> Result<()> {
    Command::new(BIN.as_os_str())
        .arg("init")
        .arg("--dry-run")
        .arg("--key-type")
        .arg("ed25519")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot prompt for key in non-interactive mode.",
        ));

    Command::new(BIN.as_os_str())
        .arg("init")
        .arg("--dry-run")
        .arg("--key-seed")
        .arg("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot prompt for key type in non-interactive mode.",
        ));

    Ok(())
}

#[test]
#[serial_test::parallel]
fn test_init_writes_config() -> Result<()> {
    let uuid = uuid::Uuid::new_v4();
    let config_name = format!("tests/fixtures/{}_config.toml", uuid);
    let key_name = format!("tests/fixtures/{}_key.pem", uuid);

    Command::new(BIN.as_os_str())
        .arg("init")
        .arg("--output")
        .arg(config_name.clone())
        .arg("--no-input")
        .arg("--key-type")
        .arg("ed25519")
        .arg("--key-file")
        .arg(key_name.clone())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Writing settings to \"{}\"",
            config_name.clone()
        )));

    let config = Settings::load_from_file(config_name.clone().into());
    let secret_key =
        ed25519_compact::SecretKey::from_pem(&std::fs::read_to_string(key_name.clone()).unwrap());

    std::fs::remove_file(config_name).unwrap();
    std::fs::remove_file(key_name).unwrap();

    assert!(config.is_ok());
    assert!(secret_key.is_ok());

    Ok(())
}
