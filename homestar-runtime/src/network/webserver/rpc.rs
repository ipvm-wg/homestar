//! JSON-RPC module for registering methods and subscriptions.

#[cfg(feature = "websocket-notify")]
use super::notifier::{self, Header, Notifier, SubscriptionTyp};
#[allow(unused_imports)]
use super::{listener, prom::PrometheusData, Message};
#[cfg(feature = "websocket-notify")]
use crate::channel::AsyncChannel;
use crate::runner::WsSender;
#[cfg(feature = "websocket-notify")]
use anyhow::anyhow;
use anyhow::Result;
#[cfg(feature = "websocket-notify")]
use dashmap::DashMap;
#[cfg(feature = "websocket-notify")]
use faststr::FastStr;
#[cfg(feature = "websocket-notify")]
use futures::StreamExt;
#[cfg(feature = "websocket-notify")]
use homestar_core::ipld::DagCbor;
use jsonrpsee::{
    server::RpcModule,
    types::error::{ErrorCode, ErrorObject},
};
#[cfg(feature = "websocket-notify")]
use jsonrpsee::{types::SubscriptionId, SubscriptionMessage, SubscriptionSink, TrySendError};
#[cfg(feature = "websocket-notify")]
use libipld::Cid;
use metrics_exporter_prometheus::PrometheusHandle;
#[cfg(feature = "websocket-notify")]
use std::sync::Arc;
use std::time::Duration;
#[allow(unused_imports)]
use tokio::sync::oneshot;
#[cfg(feature = "websocket-notify")]
use tokio::{runtime::Handle, select};
#[cfg(feature = "websocket-notify")]
use tokio_stream::wrappers::BroadcastStream;
#[cfg(feature = "websocket-notify")]
use tracing::debug;
#[allow(unused_imports)]
use tracing::{error, warn};

/// Health endpoint.
pub(crate) const HEALTH_ENDPOINT: &str = "health";
/// Metrics endpoint for prometheus / openmetrics polling.
pub(crate) const METRICS_ENDPOINT: &str = "metrics";
/// Run a workflow and subscribe to that workflow's events.
#[cfg(feature = "websocket-notify")]
pub(crate) const SUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "subscribe_run_workflow";
/// Unsubscribe from a workflow's events.
#[cfg(feature = "websocket-notify")]
pub(crate) const UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT: &str = "unsubscribe_run_workflow";
/// Subscribe to network events.
#[cfg(feature = "websocket-notify")]
pub(crate) const SUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "subscribe_network_events";
/// Unsubscribe from network events.
#[cfg(feature = "websocket-notify")]
pub(crate) const UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT: &str = "unsubscribe_network_events";

/// Context needed for RPC methods.
#[cfg(feature = "websocket-notify")]
pub(crate) struct Context {
    metrics_hdl: PrometheusHandle,
    evt_notifier: Notifier<notifier::Message>,
    workflow_msg_notifier: Notifier<notifier::Message>,
    runner_sender: WsSender,
    receiver_timeout: Duration,
    workflow_listeners: Arc<DashMap<SubscriptionId<'static>, (Cid, FastStr)>>,
}

/// Context needed for RPC methods.
#[allow(dead_code)]
#[cfg(not(feature = "websocket-notify"))]
pub(crate) struct Context {
    metrics_hdl: PrometheusHandle,
    runner_sender: WsSender,
    receiver_timeout: Duration,
}

impl Context {
    /// Create a new [Context] instance.
    #[cfg(feature = "websocket-notify")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
    pub(crate) fn new(
        metrics_hdl: PrometheusHandle,
        evt_notifier: Notifier<notifier::Message>,
        workflow_msg_notifier: Notifier<notifier::Message>,
        runner_sender: WsSender,
        receiver_timeout: Duration,
    ) -> Self {
        Self {
            metrics_hdl,
            evt_notifier,
            workflow_msg_notifier,
            runner_sender,
            receiver_timeout,
            workflow_listeners: DashMap::new().into(),
        }
    }

    /// Create a new [Context] instance.
    #[cfg(not(feature = "websocket-notify"))]
    pub(crate) fn new(
        metrics_hdl: PrometheusHandle,
        runner_sender: WsSender,
        receiver_timeout: Duration,
    ) -> Self {
        Self {
            metrics_hdl,
            runner_sender,
            receiver_timeout,
        }
    }
}

/// [RpcModule] wrapper.
pub(crate) struct JsonRpc(RpcModule<Context>);

impl JsonRpc {
    /// Create a new [JsonRpc] instance, registering methods on initialization.
    pub(crate) async fn new(ctx: Context) -> Result<Self> {
        let module = Self::register(ctx).await?;
        Ok(Self(module))
    }

    /// Get a reference to the inner [RpcModule].
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &RpcModule<Context> {
        &self.0
    }

    /// Get and take ownership of the inner [RpcModule].
    pub(crate) fn into_inner(self) -> RpcModule<Context> {
        self.0
    }

    async fn register(ctx: Context) -> Result<RpcModule<Context>> {
        let mut module = RpcModule::new(ctx);

        #[cfg(not(test))]
        module.register_async_method(HEALTH_ENDPOINT, |_, ctx| async move {
            let (tx, rx) = crate::channel::AsyncChannel::oneshot();
            ctx.runner_sender
                .send_async((Message::GetNodeInfo, Some(tx)))
                .await
                .map_err(|err| internal_err(err.to_string()))?;

            if let Ok(Message::AckNodeInfo((static_info, dyn_info))) =
                rx.recv_deadline(std::time::Instant::now() + ctx.receiver_timeout)
            {
                Ok(serde_json::json!({ "healthy": true, "nodeInfo": {
                    "static": static_info, "dynamic": dyn_info}}))
            } else {
                error!(
                    subject = "call.health",
                    category = "jsonrpc.call",
                    sub = HEALTH_ENDPOINT,
                    "did not acknowledge message in time"
                );
                Err(internal_err("failed to get node information".to_string()))
            }
        })?;

        #[cfg(test)]
        module.register_async_method(HEALTH_ENDPOINT, |_, _| async move {
            use crate::runner::{DynamicNodeInfo, StaticNodeInfo};
            use std::str::FromStr;
            let peer_id =
                libp2p::PeerId::from_str("12D3KooWRNw2pJC9748Fmq4WNV27HoSTcX3r37132FLkQMrbKAiC")
                    .unwrap();
            Ok::<serde_json::Value, ErrorObject<'_>>(serde_json::json!({
                "healthy": true, "nodeInfo": {"static": StaticNodeInfo::new(peer_id), "dynamic": DynamicNodeInfo::new(vec![])},
            }))
        })?;

        module.register_async_method(METRICS_ENDPOINT, |params, ctx| async move {
            let render = ctx.metrics_hdl.render();

            // TODO: Handle prefix specific metrics in parser.
            match params.one::<listener::MetricsPrefix>() {
                Ok(listener::MetricsPrefix { prefix }) => PrometheusData::from_string(&render)
                    .map_err(|err| {
                        internal_err(format!(
                            "failed to render metrics @prefix {} : {:#?}",
                            prefix, err
                        ))
                    }),
                Err(_) => PrometheusData::from_string(&render)
                    .map_err(|err| internal_err(format!("failed to render metrics: {:#?}", err))),
            }
        })?;

        #[cfg(feature = "websocket-notify")]
        module.register_subscription(
            SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            UNSUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
            |_, pending, ctx| async move {
                let sink = pending.accept().await?;
                let rx = ctx.evt_notifier.inner().subscribe();
                let stream = BroadcastStream::new(rx);
                Self::handle_event_subscription(
                    sink,
                    stream,
                    SUBSCRIBE_NETWORK_EVENTS_ENDPOINT.to_string(),
                )
                .await?;
                Ok(())
            },
        )?;

        #[cfg(feature = "websocket-notify")]
        module.register_subscription(
            SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            UNSUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
            |params, pending, ctx| async move {
                match params.one::<listener::Run<'_>>() {
                    Ok(listener::Run { name, workflow }) => {
                        let (tx, rx) = AsyncChannel::oneshot();
                        ctx.runner_sender
                            .send_async((
                                Message::RunWorkflow((name.clone(), workflow.clone())),
                                Some(tx),
                            ))
                            .await?;

                        if let Ok(Message::AckWorkflow((cid, name))) =
                            rx.recv_deadline(std::time::Instant::now() + ctx.receiver_timeout)
                        {
                            let sink = pending.accept().await?;
                            ctx.workflow_listeners
                                .insert(sink.subscription_id(), (cid, name));
                            let rx = ctx.workflow_msg_notifier.inner().subscribe();
                            let stream = BroadcastStream::new(rx);
                            Self::handle_workflow_subscription(sink, stream, ctx).await?;
                        } else {
                            error!(
                                subject = "subscription.workflow.err",
                                category = "jsonrpc.subscription",
                                sub = SUBSCRIBE_RUN_WORKFLOW_ENDPOINT,
                                workflow_name = name.to_string(),
                                "did not acknowledge message in time"
                            );
                            let _ = pending
                                .reject(busy_err(format!(
                                    "not able to run workflow {}",
                                    workflow.to_cid()?
                                )))
                                .await;
                        }
                    }
                    Err(err) => {
                        warn!(subject = "subscription.workflow.err",
                              category = "jsonrpc.subscription",
                              err=?err,
                              "failed to parse run workflow params");
                        let _ = pending.reject(err).await;
                    }
                }
                Ok(())
            },
        )?;

        Ok(module)
    }

    #[cfg(feature = "websocket-notify")]
    async fn handle_event_subscription(
        mut sink: SubscriptionSink,
        mut stream: BroadcastStream<notifier::Message>,
        subscription_type: String,
    ) -> Result<()> {
        let rt_hdl = Handle::current();
        rt_hdl.spawn(async move {
            loop {
                select! {
                    _ = sink.closed() => {
                        break Ok(());
                    }
                    next_msg = stream.next() => {
                        let msg = match next_msg {
                            Some(Ok(notifier::Message {
                                header: Header {
                                    subscription: SubscriptionTyp::EventSub(evt),
                                    ..
                                },
                                payload,
                            })) if evt == subscription_type => payload,
                            Some(Ok(_)) => continue,
                            Some(Err(err)) => {
                                error!(subject = "subscription.event.err",
                                       category = "jsonrpc.subscription",
                                       err=?err,
                                       "subscription stream error");
                                break Err(err.into());
                            }
                            None => break Ok(()),
                        };
                        let sub_msg = SubscriptionMessage::from_json(&msg)?;
                        match sink.try_send(sub_msg) {
                            Ok(()) => (),
                            Err(TrySendError::Closed(_)) => {
                                break Err(anyhow!("subscription sink closed"));
                            }
                            Err(TrySendError::Full(_)) => {
                                error!(subject = "subscription.event.err",
                                      category = "jsonrpc.subscription",
                                      "subscription sink full");
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    #[cfg(feature = "websocket-notify")]
    async fn handle_workflow_subscription(
        mut sink: SubscriptionSink,
        mut stream: BroadcastStream<notifier::Message>,
        ctx: Arc<Context>,
    ) -> Result<()> {
        let rt_hdl = Handle::current();
        rt_hdl.spawn(async move {
        loop {
            select! {
                _ = sink.closed() => {
                    ctx.workflow_listeners.remove(&sink.subscription_id());
                    break Ok(());
                }
                next_msg = stream.next() => {
                    let msg = match next_msg {
                        Some(Ok(notifier::Message {
                            header: Header { subscription: SubscriptionTyp::Cid(cid), ident },
                            payload,
                        })) => {
                            let msg = ctx.workflow_listeners
                                .get(&sink.subscription_id())
                                .and_then(|v| {
                                    let (v_cid, v_name) = v.value();
                                    if v_cid == &cid && (Some(v_name) == ident.as_ref() || ident.is_none()) {
                                        debug!(
                                            subject = "subscription.workflow",
                                            category = "jsonrpc.subscription",
                                            cid = cid.to_string(),
                                            ident = ident.clone().unwrap_or("undefined".into()).to_string(), "received message");
                                        Some(payload)
                                    } else {
                                        None
                                    }
                                });
                            msg
                        }
                        Some(Ok(notifier::Message {
                            header: notifier::Header { subscription: _sub, ..},
                            ..
                        })) => {
                            continue;
                        }
                        Some(Err(err)) => {
                            error!("subscription stream error: {}", err);
                            ctx.workflow_listeners.remove(&sink.subscription_id());
                            break Err(err.into());
                        }
                        None => break Ok(()),
                    };

                    if let Some(msg) = msg {
                        let sub_msg = SubscriptionMessage::from_json(&msg)?;
                        match sink.try_send(sub_msg) {
                            Ok(()) => (),
                            Err(TrySendError::Closed(_)) => {
                                ctx.workflow_listeners.remove(&sink.subscription_id());
                                break Err(anyhow!("subscription sink closed"));
                            }
                            Err(TrySendError::Full(_)) => {
                                error!(subject = "subscription.workflow.err",
                                      category = "jsonrpc.subscription",
                                      "subscription sink full");
                            }
                        }
                    }
                }
            }
        }
    });

        Ok(())
    }
}

fn internal_err<'a, T: ToString>(msg: T) -> ErrorObject<'a> {
    ErrorObject::owned(ErrorCode::InternalError.code(), msg.to_string(), None::<()>)
}

#[allow(dead_code)]
fn busy_err<'a, T: ToString>(msg: T) -> ErrorObject<'a> {
    ErrorObject::owned(ErrorCode::ServerIsBusy.code(), msg.to_string(), None::<()>)
}
