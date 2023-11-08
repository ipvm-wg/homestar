use crate::utils::{kill_homestar, stop_homestar, BIN_NAME};
use anyhow::Result;
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder,
};
use once_cell::sync::Lazy;
use retry::{delay::Exponential, retry};
use std::{
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

static BIN: Lazy<PathBuf> = Lazy::new(|| assert_cmd::cargo::cargo_bin(BIN_NAME));
const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

#[test]
fn test_connection_notifications() -> Result<()> {
    let _ = stop_homestar();

    let homestar_proc1 = Command::new(BIN.as_os_str())
        .env(
            "RUST_LOG",
            "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
        )
        .arg("start")
        .arg("-c")
        .arg("tests/fixtures/test_notification1.toml")
        .arg("--db")
        .arg("homestar1.db")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let ws_port = 8022;
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), ws_port);
    let result = retry(Exponential::from_millis(1000).take(10), || {
        TcpStream::connect(socket).map(|stream| stream.shutdown(Shutdown::Both))
    });

    if result.is_err() {
        let _ = kill_homestar(homestar_proc1, None);
        panic!("Homestar server/runtime failed to start in time");
    }

    let ws_url = format!("ws://{}:{}", Ipv4Addr::LOCALHOST, ws_port);
    tokio_test::block_on(async {
        tokio_tungstenite::connect_async(ws_url.clone())
            .await
            .unwrap();

        let client = WsClientBuilder::default()
            .build(ws_url.clone())
            .await
            .unwrap();
        let mut sub: Subscription<Vec<u8>> = client
            .subscribe(
                SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let homestar_proc2 = Command::new(BIN.as_os_str())
            .env(
                "RUST_LOG",
                "homestar=debug,homestar_runtime=debug,libp2p=debug,libp2p_gossipsub::behaviour=debug",
            )
            .arg("start")
            .arg("-c")
            .arg("tests/fixtures/test_notification2.toml")
            .arg("--db")
            .arg("homestar2.db")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let _ = kill_homestar(homestar_proc2, None);

        tokio::time::sleep(Duration::from_secs(2)).await;

        {
            let msg = sub
                .next()
                .await
                .expect("Subscription did not receive a connection established message");
            let json: serde_json::Value = serde_json::from_slice(&msg.unwrap()).unwrap();
            let typ = json["type"].as_str().unwrap();
            let peer_id = json["data"]["peer_id"].as_str().unwrap();

            assert_eq!(typ, "network:connectionEstablished");
            assert_eq!(
                peer_id,
                "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc"
            );
        }

        {
            let msg = sub
                .next()
                .await
                .expect("Subscription did not receive a connection closed message");
            let json: serde_json::Value = serde_json::from_slice(&msg.unwrap()).unwrap();
            let typ = json["type"].as_str().unwrap();
            let peer_id = json["data"]["peer_id"].as_str().unwrap();

            assert_eq!(typ, "network:connectionClosed");
            assert_eq!(
                peer_id,
                "16Uiu2HAm3g9AomQNeEctL2hPwLapap7AtPSNt8ZrBny4rLx1W5Dc"
            );
        }
    });

    let _ = kill_homestar(homestar_proc1, None);

    Ok(())
}
