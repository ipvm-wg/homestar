//! Sets up a websocket server for sending and receiving messages from browser
//! clients.

use crate::settings;
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
use futures::{stream::StreamExt, SinkExt};
use std::{
    net::{IpAddr, SocketAddr, TcpListener},
    ops::ControlFlow,
    str::FromStr,
    sync::Arc,
};
use tokio::{
    runtime::Handle,
    select,
    sync::{broadcast, mpsc, oneshot},
};
use tracing::{debug, info};

/// Type alias for websocket sender.
pub type Sender = Arc<broadcast::Sender<String>>;

/// Message type for messages sent back from the
/// websocket server to the [runner] for example.
///
/// [runner]: crate::Runner
#[derive(Debug)]
pub(crate) enum Message {
    /// Notify the listener that the websocket server is shutting down
    /// gracefully.
    GracefulShutdown(oneshot::Sender<()>),
}

/// WebSocket server state information.
#[allow(dead_code, missing_debug_implementations)]
#[derive(Clone)]
pub(crate) struct Server {
    addr: SocketAddr,
    msg_sender: Arc<Sender>,
}

impl Server {
    /// Setup bounded, MPMC channel for runtime to send and received messages
    /// through the websocket connection(s).
    fn setup_channel(capacity: usize) -> (broadcast::Sender<String>, broadcast::Receiver<String>) {
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
            msg_sender: Arc::new(sender.into()),
        })
    }

    /// Start the websocket server given settings.
    pub(crate) async fn start(self, mut rx: mpsc::Receiver<Message>) -> Result<()> {
        let app = Router::new().route("/", get(ws_handler).with_state(self.clone()));
        info!("websocket server listening on {}", self.addr);

        axum::Server::bind(&self.addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                if let Some(Message::GracefulShutdown(tx)) = rx.recv().await {
                    info!("websocket server shutting down");
                    let _ = tx.send(());
                }
            })
            .await?;

        Ok(())
    }

    /// Get websocket message sender for broadcasting messages to websocket
    /// clients.
    pub(crate) fn sender(&self) -> Arc<Sender> {
        self.msg_sender.clone()
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(state): State<Server>,
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

async fn handle_socket(mut socket: ws::WebSocket, state: Server) {
    let addr = state.addr;

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
    let handle = Handle::current();

    let mut send_task = handle.spawn(async move {
        while let Ok(msg) = subscribed_rx.recv().await {
            // In any websocket error, break loop.
            if socket_sender
                .send(AxumMsg::Binary(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let mut recv_task = handle.spawn(async move {
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
    select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("Websocket context {} destroyed", addr);
}

/// Process [messages].
///
/// [messages]: Message
async fn process_message(msg: AxumMsg, addr: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        AxumMsg::Text(t) => {
            info!(">>> {} sent str: {:?}", addr, t);
        }
        AxumMsg::Binary(d) => {
            info!(">>> {} sent {} bytes: {:?}", addr, d.len(), d);
        }
        AxumMsg::Close(c) => {
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

        AxumMsg::Pong(v) => {
            info!(">>> {} sent pong with {:?}", addr, v);
        }
        // You should never need to manually handle AxumMsg::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        AxumMsg::Ping(v) => {
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
    use crate::settings::Settings;

    #[tokio::test]
    async fn ws_connect() {
        let settings = Settings::load().unwrap();
        let state = Server::new(settings.node().network()).unwrap();
        let (_ws_tx, ws_rx) = mpsc::channel(1);
        tokio::spawn(state.start(ws_rx));

        tokio_tungstenite::connect_async("ws://localhost:1337".to_string())
            .await
            .unwrap();
    }
}
