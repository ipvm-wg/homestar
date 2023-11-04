use crate::network::webserver::Notifier;
use anyhow::anyhow;
use chrono::prelude::Utc;
use homestar_core::ipld::DagJson;
use libipld::{serde::from_ipld, Ipld};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::warn;

const TYPE_KEY: &str = "type";
const DATA_KEY: &str = "data";
const TIMESTAMP_KEY: &str = "timestamp";

/// Send notification as bytes
pub(crate) fn send(notifier: Notifier, ty: EventNotificationType, data: BTreeMap<&str, String>) {
    let notification = EventNotification::new(ty, data);

    if let Ok(json) = notification.to_json() {
        let _ = notifier.notify(json);
    } else {
        warn!("Unable to serialize notification as bytes: {notification:?}");
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct EventNotification {
    ty: EventNotificationType,
    data: Ipld,
    timestamp: i64,
}

impl EventNotification {
    pub(crate) fn new(ty: EventNotificationType, data: BTreeMap<&str, String>) -> Self {
        let ipld_data = data
            .iter()
            .map(|(key, val)| (key.to_string(), Ipld::String(val.to_owned())))
            .collect();

        Self {
            ty,
            data: Ipld::Map(ipld_data),
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

impl DagJson for EventNotification where Ipld: From<EventNotification> {}

impl From<EventNotification> for Ipld {
    fn from(notification: EventNotification) -> Self {
        Ipld::Map(BTreeMap::from([
            ("type".into(), notification.ty.into()),
            ("data".into(), notification.data),
            ("timestamp".into(), notification.timestamp.into()),
        ]))
    }
}

impl TryFrom<Ipld> for EventNotification {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let ty: EventNotificationType = map
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
            ty,
            data,
            timestamp,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum EventNotificationType {
    SwarmNotification(SwarmNotification),
}

impl DagJson for EventNotificationType where Ipld: From<EventNotificationType> {}

impl From<EventNotificationType> for Ipld {
    fn from(event_type: EventNotificationType) -> Self {
        match event_type {
            EventNotificationType::SwarmNotification(ty) => ty.into(),
        }
    }
}

impl TryFrom<Ipld> for EventNotificationType {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let ty = from_ipld::<String>(ipld)?;

        match ty.as_str() {
            "connectionEstablished" => Ok(EventNotificationType::SwarmNotification(
                SwarmNotification::ConnnectionEstablished,
            )),
            "connectionClosed" => Ok(EventNotificationType::SwarmNotification(
                SwarmNotification::ConnnectionClosed,
            )),
            _ => Err(anyhow!("Missing notification type.")),
        }
    }
}

// Types of swarm notification sent to clients
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum SwarmNotification {
    ConnnectionEstablished,
    ConnnectionClosed,
}

impl DagJson for SwarmNotification where Ipld: From<SwarmNotification> {}

#[allow(unused_variables, non_snake_case)]
impl From<SwarmNotification> for Ipld {
    fn from(notification: SwarmNotification) -> Self {
        match notification {
            SwarmNotification::ConnnectionEstablished => {
                Ipld::String("connectionEstablished".into())
            }
            SwarmNotification::ConnnectionClosed => Ipld::String("connectionClosed".into()),
        }
    }
}

impl TryFrom<Ipld> for SwarmNotification {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let ty = from_ipld::<String>(ipld)?;

        match ty.as_str() {
            "connectionEstablished" => Ok(SwarmNotification::ConnnectionEstablished),
            "connectionClosed" => Ok(SwarmNotification::ConnnectionClosed),
            _ => Err(anyhow!("Missing notification type.")),
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
            EventNotificationType::SwarmNotification(SwarmNotification::ConnnectionEstablished),
            btreemap! {
                "peer_id" => peer_id.clone(),
                "address" => address.clone()
            },
        );
        let bytes = notification.to_json().unwrap();

        let parsed = EventNotification::from_json(bytes.as_ref()).unwrap();
        let data: BTreeMap<String, String> = from_ipld(parsed.data).unwrap();

        assert_eq!(
            parsed.ty,
            EventNotificationType::SwarmNotification(SwarmNotification::ConnnectionEstablished)
        );
        assert_eq!(data.get("peer_id").unwrap(), &peer_id);
        assert_eq!(data.get("address").unwrap(), &address);
    }

    #[test]
    fn notification_json_string_rountrip() {
        let peer_id = PeerId::random().to_string();
        let address: String = "/ip4/127.0.0.1/tcp/7000".to_string();

        let notification = EventNotification::new(
            EventNotificationType::SwarmNotification(SwarmNotification::ConnnectionEstablished),
            btreemap! {
                "peer_id" => peer_id.clone(),
                "address" => address.clone()
            },
        );
        let json_string = notification.to_json_string().unwrap();

        let parsed = EventNotification::from_json_string(json_string).unwrap();
        let data: BTreeMap<String, String> = from_ipld(parsed.data).unwrap();

        assert_eq!(
            parsed.ty,
            EventNotificationType::SwarmNotification(SwarmNotification::ConnnectionEstablished)
        );
        assert_eq!(data.get("peer_id").unwrap(), &peer_id);
        assert_eq!(data.get("address").unwrap(), &address);
    }
}
