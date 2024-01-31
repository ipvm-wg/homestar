//! Evented notifications emitted to clients.

use crate::{
    network::webserver::{
        notifier::{self, Header, Message, Notifier, SubscriptionTyp},
        SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
    },
    receipt::metadata::{WORKFLOW_KEY, WORKFLOW_NAME_KEY},
    Receipt,
};
use anyhow::anyhow;
use chrono::prelude::Utc;
use homestar_invocation::{ipld::DagJson, Receipt as InvocationReceipt};
use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, str::FromStr};
use tracing::{debug, warn};

pub(crate) mod receipt;
pub(crate) mod swarm;
pub(crate) use receipt::ReceiptNotification;
pub(crate) use swarm::*;

const TYPE_KEY: &str = "type";
const DATA_KEY: &str = "data";
const TIMESTAMP_KEY: &str = "timestamp";

/// Send receipt notification as bytes.
pub(crate) fn emit_receipt(
    notifier: Notifier<notifier::Message>,
    receipt: &Receipt,
    metadata: Option<Ipld>,
) {
    let invocation_receipt = InvocationReceipt::from(receipt);
    let receipt_cid = receipt.cid();
    let notification = ReceiptNotification::with(invocation_receipt, receipt_cid, metadata.clone());

    if let Ok(json) = notification.to_json() {
        debug!(
            subject = "notification.receipt",
            category = "notification",
            cid = receipt_cid.to_string(),
            "emitting receipt to WebSocket"
        );
        if let Some(ipld) = metadata {
            match (ipld.get(WORKFLOW_KEY), ipld.get(WORKFLOW_NAME_KEY)) {
                (Ok(Ipld::Link(cid)), Ok(Ipld::String(name))) => {
                    let header =
                        Header::new(SubscriptionTyp::Cid(*cid), Some((name.to_string()).into()));
                    let _ = notifier.notify(Message::new(header, json));
                }
                (Ok(Ipld::Link(cid)), Err(_err)) => {
                    let header = Header::new(SubscriptionTyp::Cid(*cid), None);
                    let _ = notifier.notify(Message::new(header, json));
                }
                _ => (),
            }
        }
    } else {
        warn!(
            subject = "notification.err",
            category = "notification",
            cid = receipt_cid.to_string(),
            "unable to serialize receipt notification as bytes"
        );
    }
}

/// Send event notification as bytes.
pub(crate) fn emit_event(
    notifier: Notifier<notifier::Message>,
    ty: EventNotificationTyp,
    data: BTreeMap<&str, Ipld>,
) {
    let header = Header::new(
        SubscriptionTyp::EventSub(SUBSCRIBE_NETWORK_EVENTS_ENDPOINT.to_string()),
        None,
    );
    let notification = EventNotification::new(ty, data);

    if let Ok(json) = notification.to_json() {
        let _ = notifier.notify(Message::new(header, json));
    } else {
        warn!(
            subject = "notification.err",
            category = "notification",
            "unable to serialize event notification as bytes: {}",
            notification.typ
        );
    }
}

/// Send network event notification as bytes.
pub(crate) fn emit_network_event(
    notifier: Notifier<notifier::Message>,
    notification: NetworkNotification,
) {
    let header = Header::new(
        SubscriptionTyp::EventSub(SUBSCRIBE_NETWORK_EVENTS_ENDPOINT.to_string()),
        None,
    );

    if let Ok(json) = notification.to_json() {
        if let Err(err) = notifier.notify(Message::new(header, json)) {
            // TODO Check on why this causes connection closed log errors
            debug!(
                subject = "notification.err",
                category = "notification",
                err=?err,
                "unable to send notification {:?}",
                notification,
            )
        };
    } else {
        debug!(
            subject = "notification.err",
            category = "notification",
            "unable to serialize event notification as bytes: {:?}",
            notification
        );
    }
}

/// Notification sent to clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct EventNotification {
    typ: EventNotificationTyp,
    data: Ipld,
    timestamp: i64,
}

impl EventNotification {
    pub(crate) fn new(typ: EventNotificationTyp, data: BTreeMap<&str, Ipld>) -> Self {
        let data = data
            .iter()
            .map(|(key, val)| (key.to_string(), val.to_owned()))
            .collect();

        Self {
            typ,
            data: Ipld::Map(data),
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

impl DagJson for EventNotification {}

impl From<EventNotification> for Ipld {
    fn from(notification: EventNotification) -> Self {
        Ipld::Map(BTreeMap::from([
            ("type".into(), notification.typ.into()),
            ("data".into(), notification.data),
            ("timestamp".into(), notification.timestamp.into()),
        ]))
    }
}

impl TryFrom<Ipld> for EventNotification {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let typ: EventNotificationTyp = map
            .get(TYPE_KEY)
            .ok_or_else(|| anyhow!("missing {TYPE_KEY}"))?
            .to_owned()
            .try_into()?;

        let data = map
            .get(DATA_KEY)
            .ok_or_else(|| anyhow!("missing {DATA_KEY}"))?
            .to_owned();

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        Ok(EventNotification {
            typ,
            data,
            timestamp,
        })
    }
}

/// Types of notification sent to clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum EventNotificationTyp {
    SwarmNotification(SwarmNotification),
}

impl EventNotificationTyp {
    pub(crate) fn workflow_info_source_label<'a>(&self) -> Option<&'a str> {
        match &self {
            EventNotificationTyp::SwarmNotification(SwarmNotification::ReceivedWorkflowInfo) => {
                Some("provider")
            }
            EventNotificationTyp::SwarmNotification(SwarmNotification::GotWorkflowInfoDht) => {
                Some("publisher")
            }
            _ => None,
        }
    }
}

impl fmt::Display for EventNotificationTyp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventNotificationTyp::SwarmNotification(subtype) => {
                write!(f, "swarm notification: {}", subtype)
            }
        }
    }
}

impl DagJson for EventNotificationTyp {}

impl From<EventNotificationTyp> for Ipld {
    fn from(typ: EventNotificationTyp) -> Self {
        match typ {
            EventNotificationTyp::SwarmNotification(subtype) => {
                Ipld::String(format!("network:{}", subtype))
            }
        }
    }
}

impl TryFrom<Ipld> for EventNotificationTyp {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Some((ty, subtype)) = from_ipld::<String>(ipld)?.split_once(':') {
            match ty {
                "network" => Ok(EventNotificationTyp::SwarmNotification(
                    SwarmNotification::from_str(subtype)?,
                )),
                _ => Err(anyhow!("Missing event notification type: {}", ty)),
            }
        } else {
            Err(anyhow!(
                "Event notification type missing colon delimiter between type and subtype."
            ))
        }
    }
}
