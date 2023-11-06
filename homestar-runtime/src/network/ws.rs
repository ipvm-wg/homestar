//! Sets up a websocket server for sending and receiving messages from browser
//! clients.

use crate::{
    channel::AsyncBoundedChannelReceiver,
    runner::{self, WsSender},
    settings,
};
use anyhow::{anyhow, Result};
use axum::{
    extract::{
        ws::{self, Message as AxumMsg, WebSocketUpgrade},
        ConnectInfo, State, TypedHeader,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use faststr::FastStr;
use futures::{stream::StreamExt, SinkExt};
use homestar_core::Workflow;
use homestar_wasm::io::Arg;
use std::{
    net::{IpAddr, SocketAddr, TcpListener},
    ops::ControlFlow,
    str::FromStr,
    time::Duration,
};
use tokio::{
    runtime::Handle,
    select,
    sync::{broadcast, oneshot},
    time::{self, Instant},
};
use tracing::{debug, error, info, warn};

#[cfg(feature = "websocket-notify")]
pub(crate) mod listener;
#[cfg(feature = "websocket-notify")]
pub(crate) mod notifier;
#[cfg(feature = "websocket-notify")]
pub(crate) use notifier::Notifier;

/// Message type for messages sent back from the
/// websocket server to the [runner] for example.
///
/// [runner]: crate::Runner
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Message {
    /// Notify the listener that the websocket server is shutting down
    /// gracefully.
    GracefulShutdown(oneshot::Sender<()>),
    /// Error attempting to run a [Workflow].
    RunErr(runner::Error),
    /// Run a workflow, given a tuple of name, and [Workflow].
    RunWorkflow((FastStr, Workflow<'static, Arg>)),
    /// Acknowledgement of a [Workflow] run.
    ///
    /// TODO: Temporary Ack until we define semantics for JSON-RPC or similar.
    RunWorkflowAck,
}

/// WebSocket server fields.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct Server {
    /// Address of the websocket server.
    addr: SocketAddr,
    /// Message sender for broadcasting to clients connected to the
    /// websocket server.
    notifier: Notifier,
    /// Receiver timeout for the websocket server.
    receiver_timeout: Duration,
}

/// State used for the websocket server routes.
#[derive(Clone, Debug)]
struct ServerState {
    notifier: Notifier,
    runner_sender: WsSender,
    receiver_timeout: Duration,
}

impl Server {
    /// Setup bounded, MPMC channel for runtime to send and received messages
    /// through the websocket connection(s).
    fn setup_channel(
        capacity: usize,
    ) -> (broadcast::Sender<Vec<u8>>, broadcast::Receiver<Vec<u8>>) {
        broadcast::channel(capacity)
    }

    pub(crate) fn new(settings: &settings::Network) -> Result<Self> {
        let (sender, _receiver) = Self::setup_channel(settings.websocket_capacity);

        let host = IpAddr::from_str(&settings.websocket_host.to_string())?;
        let port_setting = settings.websocket_port;
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
            notifier: Notifier::new(sender),
            receiver_timeout: settings.websocket_receiver_timeout,
        })
    }

    /// Start the websocket server given settings.
    pub(crate) async fn start(
        &self,
        rx: AsyncBoundedChannelReceiver<Message>,
        runner_sender: WsSender,
    ) -> Result<()> {
        let addr = self.addr;
        info!("websocket server listening on {}", addr);
        let app = Router::new().route(
            "/",
            get(ws_handler).with_state(ServerState {
                notifier: self.notifier.clone(),
                runner_sender,
                receiver_timeout: self.receiver_timeout,
            }),
        );

        axum::Server::bind(&addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                if let Ok(Message::GracefulShutdown(tx)) = rx.recv_async().await {
                    info!("websocket server shutting down");
                    let _ = tx.send(());
                }
            })
            .await?;

        Ok(())
    }

    /// Get websocket message sender for broadcasting messages to websocket
    /// clients.
    pub(crate) fn notifier(&self) -> Notifier {
        self.notifier.clone()
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(state): State<ServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");

    // Finalize the upgrade process by returning upgrade callback.
    // We can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(mut socket: ws::WebSocket, addr: SocketAddr, state: ServerState) {
    // Send a ping (unsupported by some browsers) just to kick things off and
    // get a response.
    if socket.send(AxumMsg::Ping(vec![1, 2, 3])).await.is_ok() {
        debug!("Pinged {}...", addr);
    } else {
        info!("Could not send ping {}!", addr);
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // Receive single message from a client (we can either receive or send with
    // the socket). This will likely be the Pong for our Ping or a processed
    // message from client.
    // Waiting for message from a client will block this task, but will not
    // block other client's connections.
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, addr, &state).await.is_break() {
                return;
            }
        } else {
            info!("client {} abruptly disconnected", addr);
            return;
        }
    }

    // By splitting socket we can send and receive at the same time.
    let (mut socket_sender, mut socket_receiver) = socket.split();
    let mut subscribed_rx = state.notifier.inner().subscribe();
    let handle = Handle::current();

    let mut send_task = handle.spawn(async move {
        while let Ok(msg) = subscribed_rx.recv().await {
            // In any websocket error, break loop.
            if socket_sender.send(AxumMsg::Binary(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = handle.spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = socket_receiver.next().await {
            cnt += 1;
            if process_message(msg, addr, &state).await.is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("Websocket context {} destroyed", addr);
}

/// Process [messages].
///
/// [messages]: Message
async fn process_message(
    msg: AxumMsg,
    addr: SocketAddr,
    state: &ServerState,
) -> ControlFlow<(), ()> {
    match msg {
        AxumMsg::Text(t) => {
            debug!(">>> {} sent str: {:?}", addr, t);
        }
        AxumMsg::Binary(bytes) => {
            debug!(">>> {} sent {}", addr, bytes.len());
            match serde_json::from_slice::<listener::Run<'_>>(&bytes) {
                Ok(listener::Run {
                    action,
                    name,
                    workflow,
                }) if action.eq("run") => {
                    let (tx, rx) = oneshot::channel();
                    if let Err(err) = state
                        .runner_sender
                        .send((Message::RunWorkflow((name, workflow)), Some(tx)))
                        .await
                    {
                        error!(err=?err, "error sending message to runner");
                    }

                    if (time::timeout_at(Instant::now() + state.receiver_timeout, rx).await)
                        .is_err()
                    {
                        error!("did not acknowledge action=run message in time");
                    }
                }
                Ok(_) => warn!("unknown action or message shape"),
                // another message
                Err(_err) => debug!(
                    "{}",
                    std::str::from_utf8(&bytes).unwrap_or(format!("{:?}", bytes).as_ref())
                ),
            }
        }
        AxumMsg::Close(c) => {
            if let Some(cf) = c {
                info!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );
            } else {
                info!(">>> {} sent close message without CloseFrame", addr);
            }
            return ControlFlow::Break(());
        }

        AxumMsg::Pong(v) => {
            debug!(">>> {} sent pong with {:?}", addr, v);
        }
        // You should never need to manually handle AxumMsg::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        AxumMsg::Ping(v) => {
            debug!(">>> {} sent ping with {:?}", addr, v);
        }
    }
    ControlFlow::Continue(())
}

fn port_available(host: IpAddr, port: u16) -> bool {
    TcpListener::bind((host.to_string(), port)).is_ok()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{channel, settings::Settings};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn ws_connect() {
        let settings = Settings::load().unwrap();
        let server = Server::new(settings.node().network()).unwrap();
        let (_ws_tx, ws_rx) = channel::AsyncBoundedChannel::oneshot();
        let (runner_tx, _runner_rx) = mpsc::channel(1);
        let _ws_hdl = tokio::spawn({
            let ws_server = server.clone();
            async move { ws_server.start(ws_rx, runner_tx).await }
        });

        tokio_tungstenite::connect_async("ws://localhost:1337".to_string())
            .await
            .unwrap();
    }
}
