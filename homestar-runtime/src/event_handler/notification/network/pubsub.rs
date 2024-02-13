//! Notification types for [swarm] gossipsub events.
//!
//! [swarm]: libp2p::swarm::Swarm

use anyhow::anyhow;
use chrono::prelude::Utc;
use derive_getters::Getters;
use homestar_invocation::ipld::DagJson;
use libipld::{serde::from_ipld, Cid, Ipld};
use libp2p::PeerId;
use schemars::JsonSchema;
use std::collections::BTreeMap;

const CID_KEY: &str = "cid";
const PUBLISHER_KEY: &str = "publisher";
const RAN_KEY: &str = "ran";
const TIMESTAMP_KEY: &str = "timestamp";

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "published_receipt_pubsub")]
pub struct PublishedReceiptPubsub {
    timestamp: i64,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Ran receipt CID")]
    ran: String,
}

impl PublishedReceiptPubsub {
    pub(crate) fn new(cid: Cid, ran: String) -> PublishedReceiptPubsub {
        PublishedReceiptPubsub {
            timestamp: Utc::now().timestamp_millis(),
            cid: cid.to_string(),
            ran,
        }
    }
}

impl DagJson for PublishedReceiptPubsub {}

impl From<PublishedReceiptPubsub> for Ipld {
    fn from(notification: PublishedReceiptPubsub) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (CID_KEY.into(), notification.cid.into()),
            (RAN_KEY.into(), notification.ran.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for PublishedReceiptPubsub {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let ran = from_ipld(
            map.get(RAN_KEY)
                .ok_or_else(|| anyhow!("missing {RAN_KEY}"))?
                .to_owned(),
        )?;

        Ok(PublishedReceiptPubsub {
            timestamp,
            cid,
            ran,
        })
    }
}

#[derive(Debug, Clone, Getters, JsonSchema)]
#[schemars(rename = "received_receipt_pubsub")]
pub struct ReceivedReceiptPubsub {
    timestamp: i64,
    #[schemars(description = "Receipt publisher peer ID")]
    publisher: String,
    #[schemars(description = "Receipt CID")]
    cid: String,
    #[schemars(description = "Ran receipt CID")]
    ran: String,
}

impl ReceivedReceiptPubsub {
    pub(crate) fn new(publisher: PeerId, cid: Cid, ran: String) -> ReceivedReceiptPubsub {
        ReceivedReceiptPubsub {
            timestamp: Utc::now().timestamp_millis(),
            publisher: publisher.to_string(),
            cid: cid.to_string(),
            ran,
        }
    }
}

impl DagJson for ReceivedReceiptPubsub {}

impl From<ReceivedReceiptPubsub> for Ipld {
    fn from(notification: ReceivedReceiptPubsub) -> Self {
        let map: BTreeMap<String, Ipld> = BTreeMap::from([
            (TIMESTAMP_KEY.into(), notification.timestamp.into()),
            (PUBLISHER_KEY.into(), notification.publisher.into()),
            (CID_KEY.into(), notification.cid.into()),
            (RAN_KEY.into(), notification.ran.into()),
        ]);

        Ipld::Map(map)
    }
}

impl TryFrom<Ipld> for ReceivedReceiptPubsub {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let timestamp = from_ipld(
            map.get(TIMESTAMP_KEY)
                .ok_or_else(|| anyhow!("missing {TIMESTAMP_KEY}"))?
                .to_owned(),
        )?;

        let publisher = from_ipld(
            map.get(PUBLISHER_KEY)
                .ok_or_else(|| anyhow!("missing {PUBLISHER_KEY}"))?
                .to_owned(),
        )?;

        let cid = from_ipld(
            map.get(CID_KEY)
                .ok_or_else(|| anyhow!("missing {CID_KEY}"))?
                .to_owned(),
        )?;

        let ran = from_ipld(
            map.get(RAN_KEY)
                .ok_or_else(|| anyhow!("missing {RAN_KEY}"))?
                .to_owned(),
        )?;

        Ok(ReceivedReceiptPubsub {
            timestamp,
            publisher,
            cid,
            ran,
        })
    }
}
