use crate::{
    network::webserver::{
        notifier::{self, Header, Message, Notifier, SubscriptionTyp},
        SUBSCRIBE_NETWORK_EVENTS_ENDPOINT,
    },
    Receipt,
};
use anyhow::anyhow;
use chrono::prelude::Utc;
use homestar_core::{
    ipld::DagJson,
    workflow::{
        receipt::metadata::{WORKFLOW_KEY, WORKFLOW_NAME_KEY},
        Receipt as InvocationReceipt,
    },
};
use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};
use tracing::{info, warn};

pub(crate) mod receipt;
pub(crate) mod swarm;
pub(crate) use receipt::ReceiptNotification;
pub(crate) use swarm::SwarmNotification;

const TYPE_KEY: &str = "type";
const DATA_KEY: &str = "data";
const TIMESTAMP_KEY: &str = "timestamp";

/// Send receipt notification as bytes.
pub(crate) fn emit_receipt(
    notifier: Notifier<notifier::Message>,
    receipt: Receipt,
    metadata: Option<Ipld>,
) {
    let invocation_receipt = InvocationReceipt::from(&receipt);
    let receipt_cid = receipt.cid();
    let notification = ReceiptNotification::with(invocation_receipt, receipt_cid, metadata.clone());

    if let Ok(json) = notification.to_json() {
        info!(
            cid = receipt_cid.to_string(),
            "Sending receipt to websocket"
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
        warn!("Unable to serialize receipt as bytes: {receipt:?}");
    }
}

/// Send event notification as bytes.
pub(crate) fn emit_event(
    notifier: Notifier<notifier::Message>,
    ty: EventNotificationTyp,
    data: BTreeMap<&str, String>,
) {
    let header = Header::new(
        SubscriptionTyp::EventSub(SUBSCRIBE_NETWORK_EVENTS_ENDPOINT.to_string()),
        None,
    );
    let notification = EventNotification::new(ty, data);

    if let Ok(json) = notification.to_json() {
        let _ = notifier.notify(Message::new(header, json));
    } else {
        warn!("Unable to serialize notification as bytes: {notification:?}");
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
    pub(crate) fn new(typ: EventNotificationTyp, data: BTreeMap<&str, String>) -> Self {
        let ipld_data = data
            .iter()
            .map(|(key, val)| (key.to_string(), Ipld::String(val.to_owned())))
            .collect();

        Self {
            typ,
            data: Ipld::Map(ipld_data),
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

impl DagJson for EventNotification where Ipld: From<EventNotification> {}

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

impl DagJson for EventNotificationTyp where Ipld: From<EventNotificationTyp> {}

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

#[cfg(test)]
mod test {
    use super::*;
    use libp2p::PeerId;
    use maplit::btreemap;

    #[test]
    fn notification_bytes_rountrip() {
        let peer_id = PeerId::random().to_string();
        let address: String = "/ip4/127.0.0.1/tcp/7000".to_string();

        let notification = EventNotification::new(
            EventNotificationTyp::SwarmNotification(SwarmNotification::ConnnectionEstablished),
            btreemap! {
                "peer_id" => peer_id.clone(),
                "address" => address.clone()
            },
        );
        let bytes = notification.to_json().unwrap();

        let parsed = EventNotification::from_json(bytes.as_ref()).unwrap();
        let data: BTreeMap<String, String> = from_ipld(parsed.data).unwrap();

        assert_eq!(
            parsed.typ,
            EventNotificationTyp::SwarmNotification(SwarmNotification::ConnnectionEstablished)
        );
        assert_eq!(data.get("peer_id").unwrap(), &peer_id);
        assert_eq!(data.get("address").unwrap(), &address);
    }

    #[test]
    fn notification_json_string_rountrip() {
        let peer_id = PeerId::random().to_string();
        let address: String = "/ip4/127.0.0.1/tcp/7000".to_string();

        let notification = EventNotification::new(
            EventNotificationTyp::SwarmNotification(SwarmNotification::ConnnectionEstablished),
            btreemap! {
                "peer_id" => peer_id.clone(),
                "address" => address.clone()
            },
        );
        let json_string = notification.to_json_string().unwrap();

        let parsed = EventNotification::from_json_string(json_string).unwrap();
        let data: BTreeMap<String, String> = from_ipld(parsed.data).unwrap();

        assert_eq!(
            parsed.typ,
            EventNotificationTyp::SwarmNotification(SwarmNotification::ConnnectionEstablished)
        );
        assert_eq!(data.get("peer_id").unwrap(), &peer_id);
        assert_eq!(data.get("address").unwrap(), &address);
    }
}
