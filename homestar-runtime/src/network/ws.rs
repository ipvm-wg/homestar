//! Sets up a websocket server for sending and receiving messages from browser
//! clients.

use crate::{event_handler::channel::BoundedChannelSender, runner::Runner, settings};
use anyhow::{anyhow, Result};
use axum::{
    extract::{
        ws::{self, Message, WebSocketUpgrade},
        ConnectInfo, State, TypedHeader,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{stream::StreamExt, SinkExt};
use std::{
    net::{IpAddr, SocketAddr, TcpListener},
    ops::ControlFlow,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::broadcast;
use tracing::{debug, info};

/// Type alias for websocket sender.
pub(crate) type WsSender = Arc<broadcast::Sender<String>>;

/// Message type for messages sent back from the
/// websocket server to the [runner] for example.
///
/// [runner]: crate::Runner
#[derive(Debug, Clone, PartialEq)]
pub enum WsMessage {
    /// Notify the listener that the websocket server is shutting down
    /// gracefully.
    GracefulShutdown,
}

/// WebSocket state information.
#[allow(dead_code, missing_debug_implementations)]
#[derive(Clone)]
pub(crate) struct WebSocketServer {
    addr: SocketAddr,
    msg_sender: WsSender,
    runner_sender: Arc<BoundedChannelSender<WsMessage>>,
}

impl WebSocketServer {
    /// Setup bounded, MPMC channel for runtime to send and received messages
    /// through the websocket connection(s).
    pub(crate) fn setup_channel(
        capacity: usize,
    ) -> (broadcast::Sender<String>, broadcast::Receiver<String>) {
        broadcast::channel(capacity)
    }

    /// Start the websocket server given settings.
    pub(crate) async fn start(
        settings: settings::Network,
        ws_sender: WsSender,
        runner_sender: Arc<BoundedChannelSender<WsMessage>>,
    ) -> Result<()> {
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

        let ws_state = Self {
            addr,
            msg_sender: ws_sender,
            runner_sender: runner_sender.clone(),
        };
        let app = Router::new().route("/", get(ws_handler).with_state(ws_state.clone()));

        info!("websocket server listening on {}", addr);

        axum::Server::bind(&addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                let _ = Runner::shutdown_signal().await;
                info!("websocket server shutting down");
                drop(ws_state.msg_sender);
                let _ = runner_sender.send(WsMessage::GracefulShutdown);
            })
            .await?;

        Ok(())
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(state): State<WebSocketServer>,
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
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: ws::WebSocket, state: WebSocketServer) {
    let addr = state.addr;

    // Send a ping (unsupported by some browsers) just to kick things off and
    // get a response.
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
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
            if process_message(msg, addr).await.is_break() {
                return;
            }
        } else {
            info!("client {} abruptly disconnected", state.addr);
            return;
        }
    }

    // By splitting socket we can send and receive at the same time.
    let (mut socket_sender, mut socket_receiver) = socket.split();
    let mut subscribed_rx = state.msg_sender.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = subscribed_rx.recv().await {
            // In any websocket error, break loop.
            if socket_sender
                .send(Message::Binary(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = socket_receiver.next().await {
            cnt += 1;
            if process_message(msg, addr).await.is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("Websocket context {} destroyed", addr);
}

/// Process [messages].
///
/// [messages]: Message
async fn process_message(msg: Message, addr: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            info!(">>> {} sent str: {:?}", addr, t);
        }
        Message::Binary(d) => {
            info!(">>> {} sent {} bytes: {:?}", addr, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );
            } else {
                info!(">>> {} somehow sent close message without CloseFrame", addr);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            info!(">>> {} sent pong with {:?}", addr, v);
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            info!(">>> {} sent ping with {:?}", addr, v);
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
    use crate::{event_handler::channel::BoundedChannel, settings::Settings};

    #[tokio::test]
    async fn ws_connect() {
        let settings = Arc::new(Settings::load().unwrap());
        let (tx, _rx) = WebSocketServer::setup_channel(10);
        let ch = BoundedChannel::oneshot();
        tokio::spawn(WebSocketServer::start(
            settings.node().network().clone(),
            tx.into(),
            ch.tx.into(),
        ));

        tokio_tungstenite::connect_async("ws://localhost:1337".to_string())
            .await
            .unwrap();
    }
}
