use crate::{channel, event_handler::Event};
use libp2p::PeerId;
use moka::{
    future::Cache,
    notification::RemovalCause::{self, Expired},
    Expiry as ExpiryBase,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

struct Expiry;

impl ExpiryBase<String, CacheValue> for Expiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &CacheValue,
        _current_time: Instant,
    ) -> Option<Duration> {
        value.expiration.as_duration()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CacheValue {
    expiration: Expiration,
    data: HashMap<String, CacheData>,
}

impl CacheValue {
    pub(crate) fn new(expiration: Expiration, data: HashMap<String, CacheData>) -> Self {
        Self { expiration, data }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CacheData {
    Peer(PeerId),
    OnExpiration(DispatchEvent),
}

#[derive(Clone, Debug)]
pub(crate) enum DispatchEvent {
    RegisterPeer,
    DiscoverPeers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Expiration {
    Registration(Duration),
    Discovery(Duration),
}

impl Expiration {
    pub(crate) fn as_duration(&self) -> Option<Duration> {
        match self {
            Expiration::Registration(ttl) => Some(*ttl),
            Expiration::Discovery(interval) => Some(*interval),
        }
    }
}

pub(crate) fn setup_cache(
    sender: Arc<channel::AsyncBoundedChannelSender<Event>>,
) -> Cache<String, CacheValue> {
    let eviction_listener = move |_key: Arc<String>, val: CacheValue, cause: RemovalCause| {
        let tx = Arc::clone(&sender);

        if let Some(CacheData::OnExpiration(event)) = val.data.get("on_expiration") {
            if cause != Expired {
                return;
            }

            match event {
                DispatchEvent::RegisterPeer => {
                    if let Some(CacheData::Peer(rendezvous_node)) = val.data.get("rendezvous_node")
                    {
                        let _ = tx.send(Event::RegisterPeer(rendezvous_node.to_owned()));
                    };
                }
                DispatchEvent::DiscoverPeers => {
                    if let Some(CacheData::Peer(rendezvous_node)) = val.data.get("rendezvous_node")
                    {
                        let _ = tx.send(Event::DiscoverPeers(rendezvous_node.to_owned()));
                    };
                }
            }
        }
    };

    Cache::builder()
        .expire_after(Expiry)
        .eviction_listener(eviction_listener)
        .build()
}
