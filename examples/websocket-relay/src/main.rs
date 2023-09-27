use homestar_runtime::{db::Database, Db, Logger, Runner, Settings};
use miette::Result;
use retry::{delay::Fixed, retry};
use std::{
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream},
    process::{Child, Command, Stdio},
};
use tracing::info;

fn main() -> Result<()> {
    let settings = Settings::load().expect("runtime settings to be loaded");
    let _guard = Logger::init(&settings.monitoring());

    // Just for example purposes, we're going to start the ipfs
    // daemon. Typically, these would be started separately.
    let mut ipfs_daemon = ipfs_setup();

    info!(
        subject = "settings",
        category = "homestar_init",
        "starting with settings: {:?}",
        settings,
    );

    let db = Db::setup_connection_pool(settings.node(), None).expect("to setup database pool");

    info!(
        subject = "database",
        category = "homestar_init",
        "starting with database: {}",
        Db::url().expect("database url to be provided"),
    );

    info!("starting Homestar runtime...");
    Runner::start(settings, db).expect("Failed to start runtime");

    // ipfs cleanup after runtime is stopped
    match ipfs_daemon.try_wait() {
        Ok(Some(status)) => info!("exited with: {status}"),
        Ok(None) => ipfs_daemon.kill().unwrap(),
        Err(e) => panic!("error attempting to wait: {e}"),
    }

    Ok(())
}

fn ipfs_setup() -> Child {
    let mut ipfs_daemon = Command::new("ipfs")
        .args(["--repo-dir", "./tmp/.ipfs", "--offline", "daemon", "--init"])
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawning of `ipfs daemon` process");

    // wait for ipfs daemon to start by testing for a connection
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5001);
    let result = retry(Fixed::from_millis(500), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if let Err(err) = result {
        ipfs_daemon.kill().unwrap();
        panic!("`ipfs daemon` failed to start: {:?}", err);
    }

    info!("`ipfs daemon` started");

    let ipfs_add_img = Command::new("ipfs")
        .args([
            "--repo-dir",
            "./tmp/.ipfs",
            "add",
            "--cid-version",
            "1",
            "./synthcat.png",
        ])
        .stderr(Stdio::piped())
        .output()
        .expect("`ipfs add` of synthcat.png");

    info!("synthcat.png added to local IPFS instance");

    let ipfs_add_wasm = Command::new("ipfs")
        .args([
            "--repo-dir",
            "./tmp/.ipfs",
            "add",
            "--cid-version",
            "1",
            "./example_test.wasm",
        ])
        .stderr(Stdio::piped())
        .output()
        .expect("`ipfs add` of synthcat.png");

    info!("wasm module added to local IPFS instance");

    info!("ipfs: {:?}", ipfs_add_img);
    info!("ipfs: {:?}", ipfs_add_wasm);
    if !ipfs_add_img.status.success() || !ipfs_add_wasm.status.success() {
        panic!("`ipfs add` failed");
    }

    ipfs_daemon
}
