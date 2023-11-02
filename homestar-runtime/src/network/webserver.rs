//! Sets up a websocket server for sending and receiving messages from browser
//! clients.

use crate::{runner, runner::WsSender, settings};
use anyhow::{anyhow, Result};
use faststr::FastStr;
use homestar_core::Workflow;
use homestar_wasm::io::Arg;
use http::{header::CONTENT_TYPE, Method};
use jsonrpsee::{
    self,
    server::{middleware::ProxyGetRequestLayer, ServerHandle},
};
use metrics_exporter_prometheus::PrometheusHandle;
use std::{
    net::{IpAddr, SocketAddr, TcpListener},
    str::FromStr,
    time::Duration,
};
use tokio::runtime::Handle;
#[cfg(feature = "websocket-notify")]
use tokio::sync::broadcast;
use tower_http::cors::{self, CorsLayer};
use tracing::info;

pub(crate) mod listener;
#[cfg(feature = "websocket-notify")]
pub(crate) mod notifier;
mod prom;
mod rpc;

#[cfg(feature = "websocket-notify")]
pub(crate) use notifier::Notifier;
use rpc::{Context, JsonRpc};

/// Message type for messages sent back from the
/// websocket server to the [runner] for example.
///
/// [runner]: crate::Runner
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Message {
    /// Error attempting to run a [Workflow].
    RunErr(runner::Error),
    /// Run a workflow, given a tuple of name, and [Workflow].
    RunWorkflow((FastStr, Workflow<'static, Arg>)),
    /// Acknowledgement of a [Workflow] run.
    AckWorkflow,
}

/// WebSocket server fields.
#[cfg(feature = "websocket-notify")]
#[derive(Clone, Debug)]
pub(crate) struct Server {
    /// Address of the websocket server.
    addr: SocketAddr,
    /// TODO
    capacity: usize,
    /// Message sender for broadcasting to clients connected to the
    /// websocket server.
    notifier: Notifier,
    /// Receiver timeout for the websocket server.
    receiver_timeout: Duration,
    /// TODO
    webserver_timeout: Duration,
}

#[cfg(not(feature = "websocket-notify"))]
#[derive(Clone, Debug)]
pub(crate) struct Server {
    /// Address of the websocket server.
    addr: SocketAddr,
    /// TODO
    capacity: usize,
    /// Receiver timeout for the websocket server.
    receiver_timeout: Duration,
    /// TODO
    webserver_timeout: Duration,
}

impl Server {
    /// Setup bounded, MPMC channel for runtime to send and received messages
    /// through the websocket connection(s).
    #[cfg(feature = "websocket-notify")]
    fn setup_channel(
        capacity: usize,
    ) -> (broadcast::Sender<Vec<u8>>, broadcast::Receiver<Vec<u8>>) {
        broadcast::channel(capacity)
    }

    #[cfg(feature = "websocket-notify")]
    pub(crate) fn new(settings: &settings::Network) -> Result<Self> {
        let (sender, _receiver) = Self::setup_channel(settings.websocket_capacity);
        let host = IpAddr::from_str(&settings.webserver_host.to_string())?;
        let port_setting = settings.webserver_port;
        let addr = if port_available(host, port_setting) {
            SocketAddr::from((host, port_setting))
        } else {
            let port = (port_setting..port_setting + 1000)
                .find(|port| port_available(host, *port))
                .ok_or_else(|| anyhow!("no free TCP ports available"))?;
            SocketAddr::from((host, port))
        };

        Ok(Self {
            addr,
            capacity: settings.websocket_capacity,
            notifier: Notifier::new(sender),
            receiver_timeout: settings.websocket_receiver_timeout,
            webserver_timeout: settings.webserver_timeout,
        })
    }

    #[cfg(not(feature = "websocket-notify"))]
    pub(crate) fn new(settings: &settings::Network) -> Result<Self> {
        let host = IpAddr::from_str(&settings.webserver_host.to_string())?;
        let port_setting = settings.webserver_port;
        let addr = if port_available(host, port_setting) {
            SocketAddr::from((host, port_setting))
        } else {
            let port = (port_setting..port_setting + 1000)
                .find(|port| port_available(host, *port))
                .ok_or_else(|| anyhow!("no free TCP ports available"))?;
            SocketAddr::from((host, port))
        };

        Ok(Self {
            addr,
            capacity: settings.websocket_capacity,
            receiver_timeout: settings.websocket_receiver_timeout,
            webserver_timeout: settings.webserver_timeout,
        })
    }

    /// Start the websocket server.
    #[cfg(feature = "websocket-notify")]
    pub(crate) async fn start(
        &self,
        runner_sender: WsSender,
        metrics_hdl: PrometheusHandle,
    ) -> Result<ServerHandle> {
        let module = JsonRpc::new(Context::new(
            metrics_hdl,
            self.notifier.clone(),
            runner_sender,
            self.receiver_timeout,
        ))
        .await?;

        self.start_inner(module).await
    }

    /// Start the websocket server.
    #[cfg(not(feature = "websocket-notify"))]
    pub(crate) async fn start(
        &self,
        runner_sender: WsSender,
        metrics_hdl: PrometheusHandle,
    ) -> Result<ServerHandle> {
        let module = JsonRpc::new(Context::new(
            metrics_hdl,
            runner_sender,
            self.receiver_timeout,
        ))
        .await?;
        self.start_inner(module).await
    }

    /// Get websocket message sender for broadcasting messages to websocket
    /// clients.
    #[cfg(feature = "websocket-notify")]
    pub(crate) fn notifier(&self) -> Notifier {
        self.notifier.clone()
    }

    async fn start_inner(&self, module: JsonRpc) -> Result<ServerHandle> {
        let addr = self.addr;
        info!("webserver listening on {}", addr);

        let cors = CorsLayer::new()
            // Allow `POST` when accessing the resource
            .allow_methods([Method::GET, Method::POST])
            // Allow requests from any origin
            .allow_origin(cors::Any)
            .allow_headers([CONTENT_TYPE]);

        let middleware = tower::ServiceBuilder::new()
            .layer(ProxyGetRequestLayer::new("/health", rpc::HEALTH_ENDPOINT)?)
            .layer(ProxyGetRequestLayer::new(
                "/metrics",
                rpc::METRICS_ENDPOINT,
            )?)
            .layer(cors)
            .timeout(self.webserver_timeout);

        let runtime_hdl = Handle::current();

        let server = jsonrpsee::server::Server::builder()
            .custom_tokio_runtime(runtime_hdl.clone())
            .set_middleware(middleware)
            .set_message_buffer_capacity(self.capacity as u32)
            .build(addr)
            .await
            .expect("Webserver to startup");

        let hdl = server.start(module.into_inner());
        runtime_hdl.spawn(hdl.clone().stopped());

        Ok(hdl)
    }
}

fn port_available(host: IpAddr, port: u16) -> bool {
    TcpListener::bind((host.to_string(), port)).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::settings::Settings;
    use homestar_core::test_utils;
    #[cfg(feature = "websocket-notify")]
    use homestar_core::{
        ipld::DagJson,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
    };
    #[cfg(feature = "websocket-notify")]
    use jsonrpsee::core::client::{Subscription, SubscriptionClientT};
    #[cfg(feature = "websocket-notify")]
    use jsonrpsee::types::error::ErrorCode;
    use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
    #[cfg(feature = "websocket-notify")]
    use notifier::NotifyReceipt;
    use serial_test::file_serial;
    use tokio::sync::mpsc;

    fn set_ports(settings: &mut Settings) {
        settings.node.network.metrics_port = test_utils::ports::get_port() as u16;
        settings.node.network.webserver_port = test_utils::ports::get_port() as u16;
    }

    async fn metrics_handle(settings: Settings) -> PrometheusHandle {
        #[cfg(feature = "monitoring")]
        let metrics_hdl = crate::metrics::start(settings.monitoring(), settings.node.network())
            .await
            .unwrap();

        #[cfg(not(feature = "monitoring"))]
        let metrics_hdl = crate::metrics::start(settings.node.network())
            .await
            .unwrap();

        metrics_hdl
    }

    #[tokio::test]
    #[file_serial]
    async fn ws_connect() {
        let mut settings = Settings::load().unwrap();
        set_ports(&mut settings);
        let server = Server::new(settings.node().network()).unwrap();
        let metrics_hdl = metrics_handle(settings).await;
        let (runner_tx, _runner_rx) = mpsc::channel(1);
        server.start(runner_tx, metrics_hdl).await.unwrap();

        let ws_url = format!("ws://{}", server.addr);
        let http_url = format!("http://{}", server.addr);

        tokio_tungstenite::connect_async(ws_url.clone())
            .await
            .unwrap();

        let client = WsClientBuilder::default().build(ws_url).await.unwrap();
        let ws_resp: serde_json::Value = client
            .request(rpc::HEALTH_ENDPOINT, rpc_params![])
            .await
            .unwrap();

        assert_eq!(ws_resp, serde_json::json!({"healthy": true}));
        let http_resp = reqwest::get(format!("{}/health", http_url)).await.unwrap();
        assert_eq!(http_resp.status(), 200);
        let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();
        assert_eq!(http_resp, serde_json::json!({"healthy": true}));

        unsafe { metrics::clear_recorder() }
    }

    #[cfg(feature = "monitoring")]
    #[tokio::test]
    #[file_serial]
    async fn ws_metrics_no_prefix() {
        let mut settings = Settings::load().unwrap();
        set_ports(&mut settings);
        settings.monitoring.process_collector_interval = Duration::from_millis(100);
        let server = Server::new(settings.node().network()).unwrap();
        let metrics_hdl = metrics_handle(settings).await;
        let (runner_tx, _runner_rx) = mpsc::channel(1);
        server.start(runner_tx, metrics_hdl).await.unwrap();

        let ws_url = format!("ws://{}", server.addr);

        // wait for interval to pass
        std::thread::sleep(Duration::from_millis(100));

        let client = WsClientBuilder::default().build(ws_url).await.unwrap();
        let ws_resp1: serde_json::Value = client
            .request(rpc::METRICS_ENDPOINT, rpc_params![])
            .await
            .unwrap();

        let len = if let serde_json::Value::Array(array) = &ws_resp1["metrics"] {
            array.len()
        } else {
            panic!("expected array");
        };

        assert!(len > 0);

        unsafe { metrics::clear_recorder() }
    }

    #[cfg(feature = "websocket-notify")]
    #[tokio::test]
    #[file_serial]
    async fn ws_subscribe_unsubscribe_network_events() {
        let mut settings = Settings::load().unwrap();
        set_ports(&mut settings);
        let server = Server::new(settings.node().network()).unwrap();
        let metrics_hdl = metrics_handle(settings).await;
        let (runner_tx, _runner_rx) = mpsc::channel(1);
        server.start(runner_tx, metrics_hdl).await.unwrap();

        let ws_url = format!("ws://{}", server.addr);

        let client1 = WsClientBuilder::default().build(ws_url).await.unwrap();
        let mut sub: Subscription<Vec<u8>> = client1
            .subscribe(
                rpc::SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
                rpc_params![],
                rpc::UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            )
            .await
            .unwrap();

        // send any bytes through (Vec<u8>)
        let (invocation_receipt, runtime_receipt) = crate::test_utils::receipt::receipts();
        let receipt = NotifyReceipt::with(invocation_receipt, runtime_receipt.cid(), None);
        server.notifier.notify(receipt.to_json().unwrap()).unwrap();
        let msg = sub.next().await.unwrap().unwrap();
        let returned: NotifyReceipt = DagJson::from_json(&msg).unwrap();

        assert_eq!(returned, receipt);
        assert!(sub.unsubscribe().await.is_ok());

        unsafe { metrics::clear_recorder() }
    }

    #[cfg(feature = "websocket-notify")]
    #[tokio::test]
    #[file_serial]
    async fn ws_subscribe_workflow_incorrect_params() {
        let mut settings = Settings::load().unwrap();
        set_ports(&mut settings);
        let server = Server::new(settings.node().network()).unwrap();
        let metrics_hdl = metrics_handle(settings).await;
        let (runner_tx, _runner_rx) = mpsc::channel(1);
        server.start(runner_tx, metrics_hdl).await.unwrap();

        let ws_url = format!("ws://{}", server.addr);

        let client = WsClientBuilder::default().build(ws_url).await.unwrap();
        let sub: Result<Subscription<Vec<u8>>, jsonrpsee::core::error::Error> = client
            .subscribe(
                rpc::SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![],
                rpc::UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await;

        assert!(sub.is_err());

        if let Err(jsonrpsee::core::error::Error::Call(err)) = sub {
            let check = ErrorCode::InvalidParams;
            assert_eq!(err.code(), check.code());
        } else {
            panic!("expected same error code");
        }

        unsafe { metrics::clear_recorder() }
    }

    #[cfg(feature = "websocket-notify")]
    #[tokio::test]
    #[file_serial]
    async fn ws_subscribe_workflow_runner_timeout() {
        let mut settings = Settings::load().unwrap();
        set_ports(&mut settings);
        let server = Server::new(settings.node().network()).unwrap();
        let metrics_hdl = metrics_handle(settings).await;
        let (runner_tx, _runner_rx) = mpsc::channel(1);
        server.start(runner_tx, metrics_hdl).await.unwrap();

        let ws_url = format!("ws://{}", server.addr);

        let config = Resources::default();
        let instruction1 = test_utils::workflow::instruction::<Arg>();
        let (instruction2, _) = test_utils::workflow::wasm_instruction_with_nonce::<Arg>();

        let task1 = Task::new(
            RunInstruction::Expanded(instruction1),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.into(),
            UcanPrf::default(),
        );

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let run_str = format!(
            r#"{{"name": "test","workflow": {}}}"#,
            workflow.to_json_string().unwrap()
        );

        let run: serde_json::Value = serde_json::from_str(&run_str).unwrap();
        let client = WsClientBuilder::default().build(ws_url).await.unwrap();
        let sub: Result<Subscription<Vec<u8>>, jsonrpsee::core::error::Error> = client
            .subscribe(
                rpc::SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                rpc_params![run],
                rpc::UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            )
            .await;

        assert!(sub.is_err());

        // Assure error is not on parse of params, but due to runner
        // timeout (as runner is not available).
        if let Err(jsonrpsee::core::error::Error::Call(err)) = sub {
            let check = ErrorCode::InternalError;
            assert_eq!(err.code(), check.code());
        } else {
            panic!("expected same error code");
        }

        unsafe { metrics::clear_recorder() }
    }
}
