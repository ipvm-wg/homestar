//! Event-handler cache for retry events.

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
        Some(value.expiration)
    }
}

/// A cache value, made-up of an expiration and data map.
#[derive(Clone, Debug)]
pub(crate) struct CacheValue {
    expiration: Duration,
    data: HashMap<String, CacheData>,
}

impl CacheValue {
    pub(crate) fn new(expiration: Duration, data: HashMap<String, CacheData>) -> Self {
        Self { expiration, data }
    }
}

/// Kinds of data to be stored in the cache.
#[derive(Clone, Debug)]
pub(crate) enum CacheData {
    Peer(PeerId),
    OnExpiration(DispatchEvent),
}

/// Events to be dispatched on cache expiration.
#[derive(Clone, Debug)]
pub(crate) enum DispatchEvent {
    RegisterPeer,
    DiscoverPeers,
}

/// Setup a cache with an eviction listener.
pub(crate) fn setup_cache(
    sender: Arc<channel::AsyncChannelSender<Event>>,
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
