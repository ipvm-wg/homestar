//! Sets up a websocket server for sending and receiving messages from browser
//! clients.

use crate::settings;
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

/// WebSocket state information.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct WebSocket {
    addr: SocketAddr,
    sender: Arc<broadcast::Sender<String>>,
    receiver: Arc<broadcast::Receiver<String>>,
}

impl WebSocket {
    /// Setup bounded, MPMC channel for runtime to send and received messages
    /// through the websocket connection(s).
    pub fn setup_channel(
        settings: &settings::Node,
    ) -> (broadcast::Sender<String>, broadcast::Receiver<String>) {
        broadcast::channel(settings.network.websocket_capacity)
    }

    /// Start the websocket server given settings.
    pub async fn start_server(
        sender: Arc<broadcast::Sender<String>>,
        receiver: Arc<broadcast::Receiver<String>>,
        settings: &settings::Node,
    ) -> Result<()> {
        let host = IpAddr::from_str(&settings.network.websocket_host.to_string())?;
        let addr = if port_available(host, settings.network.websocket_port) {
            SocketAddr::from((host, settings.network.websocket_port))
        } else {
            let port = (settings.network.websocket_port..settings.network.websocket_port + 1000)
                .find(|port| port_available(host, *port))
                .ok_or_else(|| anyhow!("no free TCP ports available"))?;
            SocketAddr::from((host, port))
        };

        let ws_state = Self {
            addr,
            sender,
            receiver,
        };
        let app = Router::new().route("/", get(ws_handler).with_state(ws_state));

        tokio::spawn(async move {
            axum::Server::bind(&addr)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await
                .expect("Websocket server to start");
        });

        tracing::info!("websocket server starting on {addr}");

        Ok(())
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(state): State<WebSocket>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected.");

    // Finalize the upgrade process by returning upgrade callback.
    // We can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: ws::WebSocket, state: WebSocket) {
    let addr = state.addr;

    // Send a ping (unsupported by some browsers) just to kick things off and
    // get a response.
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        tracing::debug!("Pinged {}...", addr);
    } else {
        tracing::info!("Could not send ping {}!", addr);
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
            tracing::info!("client {} abruptly disconnected", state.addr);
            return;
        }
    }

    // By splitting socket we can send and receive at the same time.
    let (mut socket_sender, mut socket_receiver) = socket.split();
    let mut subscribed_rx = state.sender.subscribe();

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

    tracing::info!("Websocket context {} destroyed", addr);
}

/// Process [messages].
///
/// [messages]: Message
async fn process_message(msg: Message, addr: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            tracing::info!(">>> {} sent str: {:?}", addr, t);
        }
        Message::Binary(d) => {
            tracing::info!(">>> {} sent {} bytes: {:?}", addr, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                tracing::info!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr,
                    cf.code,
                    cf.reason
                );
            } else {
                tracing::info!(">>> {} somehow sent close message without CloseFrame", addr);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            tracing::info!(">>> {} sent pong with {:?}", addr, v);
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            tracing::info!(">>> {} sent ping with {:?}", addr, v);
        }
    }
    ControlFlow::Continue(())
}

fn port_available(host: IpAddr, port: u16) -> bool {
    TcpListener::bind((host.to_string(), port)).is_ok()
}

#[cfg(test)]
mod test {
    use crate::settings::Settings;

    use super::*;

    #[tokio::test]
    async fn ws_connect() {
        let (tx, rx) = broadcast::channel(1);
        let sender = Arc::new(tx);
        let receiver = Arc::new(rx);
        let settings = Settings::load().unwrap();

        WebSocket::start_server(Arc::clone(&sender), Arc::clone(&receiver), settings.node())
            .await
            .unwrap();

        tokio_tungstenite::connect_async("ws://localhost:1337".to_string())
            .await
            .unwrap();
    }
}
