//! Sets up a webserver for WebSocket and HTTP interaction with clients.

use crate::{
    db::Database,
    ip, runner,
    runner::{DynamicNodeInfo, StaticNodeInfo, WsSender},
    settings,
};
use anyhow::{anyhow, Result};
use faststr::FastStr;
use futures::future::{self, Either};
use homestar_wasm::io::Arg;
use homestar_workflow::Workflow;
use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    method::Method,
};
use jsonrpsee::server::{
    middleware::http::ProxyGetRequestLayer, stop_channel, RandomStringIdProvider, ServerHandle,
};
use libipld::Cid;
use metrics_exporter_prometheus::PrometheusHandle;
use std::{
    iter::once,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    str::FromStr,
    time::Duration,
};
#[cfg(feature = "websocket-notify")]
use tokio::sync::broadcast;
use tokio::{net::TcpListener, runtime::Handle, select};
use tower_http::{
    cors::{self, CorsLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
};
use tracing::{debug, error, info};

pub(crate) mod listener;
#[cfg(feature = "websocket-notify")]
pub(crate) mod notifier;
mod prom;
mod rpc;

#[cfg(feature = "websocket-notify")]
pub(crate) use notifier::Notifier;
pub use prom::PrometheusData;
#[cfg(feature = "websocket-notify")]
pub(crate) use rpc::SUBSCRIBE_NETWORK_EVENTS_ENDPOINT;
use rpc::{Context, JsonRpc};

/// Message type for messages sent back from the
/// WebSocket server to the [runner] for example.
///
/// [runner]: crate::Runner
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Message {
    RunErr(runner::Error),
    /// Run a workflow, given a tuple of name, and [Workflow].
    RunWorkflow((FastStr, Workflow<'static, Arg>)),
    /// Acknowledgement of a [Workflow] run.
    AckWorkflow((Cid, FastStr)),
    /// Message sent to the [Runner] to gather node information from the [EventHandler].
    ///
    /// [Runner]: crate::Runner
    /// [EventHandler]: crate::EventHandler
    GetNodeInfo,
    /// Acknowledgement of a [Message::GetNodeInfo] request, receiving static and dynamic
    /// node information.
    AckNodeInfo((StaticNodeInfo, DynamicNodeInfo)),
}

/// Server fields.
#[cfg(feature = "websocket-notify")]
#[derive(Clone, Debug)]
pub(crate) struct Server {
    /// V4 Address of the server.
    v4_addr: SocketAddr,
    /// V6 Address of the server.
    v6_addr: SocketAddr,
    /// Message buffer capacity for the server.
    capacity: usize,
    /// Message sender for broadcasting internal events to clients connected to
    /// to the server.
    evt_notifier: Notifier<notifier::Message>,
    /// Message sender for broadcasting workflow-related events to clients
    /// connected to to the server.
    workflow_msg_notifier: Notifier<notifier::Message>,
    /// Sender timeout for the [Sink] messages.
    ///
    /// [Sink]: jsonrpsee::SubscriptionSink
    sender_timeout: Duration,
    /// General timeout for the server.
    webserver_timeout: Duration,
}

/// Server fields.
#[cfg(not(feature = "websocket-notify"))]
#[derive(Clone, Debug)]
pub(crate) struct Server {
    /// Address of the server.
    addr: SocketAddr,
    /// Message buffer capacity for the server.
    capacity: usize,
    /// Sender timeout for the [Sink] messages.
    ///
    /// [Sink]: jsonrpsee::SubscriptionSink
    sender_timeout: Duration,
    /// General timeout for the server.
    webserver_timeout: Duration,
}

impl Server {
    /// Setup bounded, MPMC channel for runtime to send and received messages
    /// through the WebSocket connection(s).
    #[cfg(feature = "websocket-notify")]
    fn setup_channel(
        capacity: usize,
    ) -> (
        broadcast::Sender<notifier::Message>,
        broadcast::Receiver<notifier::Message>,
    ) {
        broadcast::channel(capacity)
    }

    /// Set up a new [Server] instance, which acts as both a
    /// WebSocket and HTTP server.
    #[cfg(feature = "websocket-notify")]
    pub(crate) fn new(settings: &settings::Webserver) -> Result<Self> {
        let (evt_sender, _receiver) = Self::setup_channel(settings.websocket_capacity);
        let (msg_sender, _receiver) = Self::setup_channel(settings.websocket_capacity);
        let v4_host = IpAddr::from_str(&settings.v4_host.to_string())?;
        let v6_host = ip::parse_ip_from_uri_host(&settings.v6_host.to_string())
            .ok_or_else(|| anyhow!("unable to parse URI"))?;

        let port_setting = settings.port;
        let (v4_addr, v6_addr) = if port_available(v4_host, port_setting) {
            (
                SocketAddr::from((v4_host, port_setting)),
                SocketAddr::from((v6_host, port_setting)),
            )
        } else {
            let port = (port_setting..port_setting + 1000)
                .find(|port| port_available(v4_host, *port))
                .ok_or_else(|| anyhow!("no free TCP ports available"))?;
            (
                SocketAddr::from((v6_host, port)),
                SocketAddr::from((v6_host, port)),
            )
        };

        Ok(Self {
            v4_addr,
            v6_addr,
            capacity: settings.websocket_capacity,
            evt_notifier: Notifier::new(evt_sender),
            workflow_msg_notifier: Notifier::new(msg_sender),
            sender_timeout: settings.websocket_sender_timeout,
            webserver_timeout: settings.timeout,
        })
    }

    /// Set up a new [Server] instance, which only acts as an HTTP server.
    #[cfg(not(feature = "websocket-notify"))]
    pub(crate) fn new(settings: &settings::Webserver) -> Result<Self> {
        let v4_host = IpAddr::from_str(&settings.v4_host.to_string())?;
        let v6_host = IpAddr::from_str(&settings.v6_host.to_string())?;
        let port_setting = settings.port;

        let port_setting = settings.port;
        let (v4_addr, v6_addr) = if port_available(v4_host, port_setting) {
            (
                SocketAddr::from((v4_host, port_setting)),
                SocketAddr::from((v6_host, port_setting)),
            )
        } else {
            let port = (port_setting..port_setting + 1000)
                .find(|port| port_available(v4_host, *port))
                .ok_or_else(|| anyhow!("no free TCP ports available"))?;
            (
                SocketAddr::from((v6_host, port)),
                SocketAddr::from((v6_host, port)),
            )
        };

        Ok(Self {
            v4_addr,
            v6_addr,
            capacity: settings.websocket_capacity,
            sender_timeout: settings.websocket_sender_timeout,
            webserver_timeout: settings.timeout,
        })
    }

    /// Instantiates the [JsonRpc] module, and starts the server.
    #[cfg(feature = "websocket-notify")]
    pub(crate) async fn start(
        &self,
        runner_sender: WsSender,
        metrics_hdl: PrometheusHandle,
        db: impl Database + 'static,
    ) -> Result<ServerHandle> {
        let module = JsonRpc::new(Context::new(
            metrics_hdl,
            self.evt_notifier.clone(),
            self.workflow_msg_notifier.clone(),
            runner_sender,
            db,
            self.sender_timeout,
        ))
        .await?;

        self.start_inner(module).await
    }

    /// Instantiates the [JsonRpc] module, and starts the server.
    #[cfg(not(feature = "websocket-notify"))]
    pub(crate) async fn start(
        &self,
        runner_sender: WsSender,
        metrics_hdl: PrometheusHandle,
        db: impl Database + 'static,
    ) -> Result<ServerHandle> {
        let module = JsonRpc::new(Context::new(
            metrics_hdl,
            runner_sender,
            db,
            self.sender_timeout,
        ))
        .await?;
        self.start_inner(module).await
    }

    /// Return the WebSocket event sender for broadcasting messages to connected
    /// clients.
    #[cfg(feature = "websocket-notify")]
    pub(crate) fn evt_notifier(&self) -> Notifier<notifier::Message> {
        self.evt_notifier.clone()
    }

    /// Get WebSocket message sender for broadcasting workflow-related messages
    /// to connected clients.
    #[cfg(feature = "websocket-notify")]
    pub(crate) fn workflow_msg_notifier(&self) -> Notifier<notifier::Message> {
        self.workflow_msg_notifier.clone()
    }

    /// Shared start logic for both WebSocket and HTTP servers.
    async fn start_inner<DB: Database + 'static>(
        &self,
        module: JsonRpc<DB>,
    ) -> Result<ServerHandle> {
        info!(
            subject = "webserver.start",
            category = "webserver",
            "webserver listening on {}/{}",
            self.v4_addr,
            self.v6_addr
        );

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
            .layer(ProxyGetRequestLayer::new("/node", rpc::NODE_INFO_ENDPOINT)?)
            .layer(ProxyGetRequestLayer::new(
                "/rpc_discover",
                rpc::DISCOVER_ENDPOINT,
            )?)
            .layer(cors)
            .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
            .timeout(self.webserver_timeout);

        let runtime_hdl = Handle::current();
        let listener_v4 = TcpListener::bind(&self.v4_addr).await?;
        let listener_v6 = TcpListener::bind(&self.v6_addr).await?;
        let (stop_hdl, server_hdl) = stop_channel();

        let svc = jsonrpsee::server::Server::builder()
            .custom_tokio_runtime(runtime_hdl.clone())
            .set_http_middleware(middleware)
            .set_id_provider(Box::new(RandomStringIdProvider::new(16)))
            .set_message_buffer_capacity(self.capacity as u32)
            .to_service_builder()
            .build(module.into_inner(), stop_hdl.clone());

        runtime_hdl.clone().spawn(async move {
            loop {
                let stream = select! {
                    result = listener_v4.accept() => {
                        if let Ok((stream, _remote_addr)) = result {
                            stream
                        } else {
                            continue
                        }
                    }
                    result = listener_v6.accept() => {
                        if let Ok((stream, _remote_addr)) = result {
                            stream
                        } else {
                            continue
                        }
                    }
                    _ = stop_hdl.clone().shutdown() => break,
                };

                let svc = svc.clone();
                let stop_hdl2 = stop_hdl.clone();
                runtime_hdl.spawn(async move {
                    let conn = hyper::server::conn::Http::new()
                        .serve_connection(stream, svc)
                        .with_upgrades();

                    let stopped = stop_hdl2.shutdown();
                    tokio::pin!(stopped);

                    let res = match future::select(conn, stopped).await {
                        Either::Left((conn, _)) => conn,
                        Either::Right((_, mut conn)) => {
                            debug!("graceful shutdown of HTTP connection");
                            Pin::new(&mut conn).graceful_shutdown();
                            conn.await
                        }
                    };

                    if let Err(err) = res {
                        error!(err=?err, "HTTP connection failed");
                    }
                });
            }
        });

        Ok(server_hdl)
    }
}

fn port_available(host: IpAddr, port: u16) -> bool {
    std::net::TcpListener::bind((host.to_string(), port)).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{channel::AsyncChannel, test_utils::db::MemoryDb};
    #[cfg(feature = "websocket-notify")]
    use crate::{event_handler::notification::ReceiptNotification, test_utils};
    #[cfg(feature = "websocket-notify")]
    use homestar_invocation::ipld::DagJson;
    #[cfg(feature = "websocket-notify")]
    use jsonrpsee::core::client::{error::Error as ClientError, Subscription, SubscriptionClientT};
    #[cfg(feature = "websocket-notify")]
    use jsonrpsee::types::error::ErrorCode;
    use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
    use metrics_exporter_prometheus::PrometheusBuilder;
    #[cfg(feature = "websocket-notify")]
    use notifier::Header;
    use std::net::Ipv4Addr;

    async fn metrics_handle() -> PrometheusHandle {
        let port = port_selector::random_free_tcp_port().unwrap();
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let (recorder, _exporter) = PrometheusBuilder::new()
            .with_http_listener(socket)
            .build()
            .expect("failed to install recorder/exporter");
        recorder.handle()
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn ws_connect_v4() {
        let TestRunner { runner, settings } = TestRunner::start();
        runner.runtime.block_on(async {
            let server = Server::new(settings.node().network().webserver()).unwrap();
            let db = MemoryDb::setup_connection_pool(settings.node(), None).unwrap();
            let metrics_hdl = metrics_handle().await;
            let (runner_tx, _runner_rx) = AsyncChannel::oneshot();
            let _ws_hdl = server.start(runner_tx, metrics_hdl, db).await.unwrap();

            let ws_url = format!("ws://{}", server.v4_addr);
            let http_url = format!("http://{}", server.v4_addr);

            tokio_tungstenite::connect_async(ws_url.clone())
                .await
                .unwrap();

            let client = WsClientBuilder::default().build(ws_url).await.unwrap();
            let ws_resp: serde_json::Value = client
                .request(rpc::HEALTH_ENDPOINT, rpc_params![])
                .await
                .unwrap();
            assert_eq!(ws_resp, serde_json::json!({"healthy": true }));
            let http_resp = reqwest::get(format!("{}/health", http_url)).await.unwrap();
            assert_eq!(http_resp.status(), 200);
            let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();
            assert_eq!(http_resp, serde_json::json!({"healthy": true }));
        });
    }

    #[homestar_runtime_proc_macro::runner_test]
    fn ws_connect_v6() {
        let TestRunner { runner, settings } = TestRunner::start();
        runner.runtime.block_on(async {
            let server = Server::new(settings.node().network().webserver()).unwrap();
            let db = MemoryDb::setup_connection_pool(settings.node(), None).unwrap();
            let metrics_hdl = metrics_handle().await;
            let (runner_tx, _runner_rx) = AsyncChannel::oneshot();
            let _ws_hdl = server.start(runner_tx, metrics_hdl, db).await.unwrap();

            let ws_url = format!("ws://{}", server.v6_addr);
            let http_url = format!("http://{}", server.v6_addr);

            tokio_tungstenite::connect_async(ws_url.clone())
                .await
                .unwrap();

            let client = WsClientBuilder::default().build(ws_url).await.unwrap();
            let ws_resp: serde_json::Value = client
                .request(rpc::HEALTH_ENDPOINT, rpc_params![])
                .await
                .unwrap();
            assert_eq!(ws_resp, serde_json::json!({"healthy": true }));
            let http_resp = reqwest::get(format!("{}/health", http_url)).await.unwrap();
            assert_eq!(http_resp.status(), 200);
            let http_resp = http_resp.json::<serde_json::Value>().await.unwrap();
            assert_eq!(http_resp, serde_json::json!({"healthy": true }));
        });
    }

    #[cfg(feature = "websocket-notify")]
    #[homestar_runtime_proc_macro::runner_test]
    async fn ws_subscribe_unsubscribe_network_events() {
        let TestRunner { runner, settings } = TestRunner::start();
        runner.runtime.block_on(async {
            let server = Server::new(settings.node().network().webserver()).unwrap();
            let db = MemoryDb::setup_connection_pool(settings.node(), None).unwrap();
            let metrics_hdl = metrics_handle().await;
            let (runner_tx, _runner_rx) = AsyncChannel::oneshot();
            let _ws_hdl = server.start(runner_tx, metrics_hdl, db).await.unwrap();

            let ws_url = format!("ws://{}", server.v4_addr);

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
            let (invocation_receipt, runtime_receipt) = test_utils::receipt::receipts();
            let receipt =
                ReceiptNotification::with(invocation_receipt, runtime_receipt.cid(), None);
            server
                .evt_notifier
                .notify(notifier::Message::new(
                    Header::new(
                        notifier::SubscriptionTyp::EventSub(
                            rpc::SUBSCRIBE_NETWORK_EVENTS_ENDPOINT.to_string(),
                        ),
                        None,
                    ),
                    receipt.to_json().unwrap(),
                ))
                .unwrap();

            // send an unknown msg: this should be dropped
            server
                .evt_notifier
                .notify(notifier::Message::new(
                    Header::new(
                        notifier::SubscriptionTyp::EventSub("test".to_string()),
                        None,
                    ),
                    vec![],
                ))
                .unwrap();

            server
                .evt_notifier
                .notify(notifier::Message::new(
                    Header::new(
                        notifier::SubscriptionTyp::EventSub(
                            rpc::SUBSCRIBE_NETWORK_EVENTS_ENDPOINT.to_string(),
                        ),
                        None,
                    ),
                    receipt.to_json().unwrap(),
                ))
                .unwrap();

            let msg1 = sub.next().await.unwrap().unwrap();
            let returned1: ReceiptNotification = DagJson::from_json(&msg1).unwrap();
            assert_eq!(returned1, receipt);

            let msg2 = sub.next().await.unwrap().unwrap();
            let _returned1: ReceiptNotification = DagJson::from_json(&msg2).unwrap();

            assert!(sub.unsubscribe().await.is_ok());
        });
    }

    #[cfg(feature = "websocket-notify")]
    #[homestar_runtime_proc_macro::runner_test]
    async fn ws_subscribe_workflow_incorrect_params() {
        let TestRunner { runner, settings } = TestRunner::start();
        runner.runtime.block_on(async {
            let server = Server::new(settings.node().network().webserver()).unwrap();
            let db = MemoryDb::setup_connection_pool(settings.node(), None).unwrap();
            let metrics_hdl = metrics_handle().await;
            let (runner_tx, _runner_rx) = AsyncChannel::oneshot();
            let _ws_hdl = server.start(runner_tx, metrics_hdl, db).await.unwrap();

            let ws_url = format!("ws://{}", server.v4_addr);

            let client = WsClientBuilder::default().build(ws_url).await.unwrap();
            let sub: Result<Subscription<Vec<u8>>, ClientError> = client
                .subscribe(
                    rpc::SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                    rpc_params![],
                    rpc::UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                )
                .await;

            assert!(sub.is_err());

            if let Err(ClientError::Call(err)) = sub {
                let check = ErrorCode::InvalidParams;
                assert_eq!(err.code(), check.code());
            } else {
                panic!("expected same error code");
            }
        });
    }
}
