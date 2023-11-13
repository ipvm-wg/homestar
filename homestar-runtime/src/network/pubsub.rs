//! [gossipsub] initializer for PubSub across connected peers.
//!
//! [gossipsub]: libp2p::gossipsub

use crate::settings;
use anyhow::Result;
use libp2p::{
    gossipsub::{self, ConfigBuilder, Message, MessageAuthenticity, MessageId, ValidationMode},
    identity::Keypair,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// [Receipt]-related topic for pub(gossip)sub.
///
/// [Receipt]: homestar_core::workflow::receipt
pub(crate) const RECEIPTS_TOPIC: &str = "receipts";

/// Setup [gossipsub] mesh protocol with default configuration.
///
/// [gossipsub]: libp2p::gossipsub
pub(crate) fn new(keypair: Keypair, settings: &settings::Node) -> Result<gossipsub::Behaviour> {
    // To content-address message, we can take the hash of message and use it as an ID.
    let message_id_fn = |message: &Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = ConfigBuilder::default()
        .heartbeat_interval(settings.network.pubsub_heartbeat)
        .idle_timeout(settings.network.pubsub_idle_timeout)
        // This sets the kind of message validation. The default is Strict (enforce message signing).
        .validation_mode(ValidationMode::Strict)
        .max_transmit_size(settings.network.pubsub_max_transmit_size)
        .mesh_n_low(settings.network.pubsub_mesh_n_low)
        .mesh_outbound_min(settings.network.pubsub_mesh_outbound_min)
        .mesh_n(settings.network.pubsub_mesh_n)
        .mesh_n_high(settings.network.pubsub_mesh_n_high)
        // Content-address messages. No two messages of the same content will be propagated.
        .message_id_fn(message_id_fn)
        .duplicate_cache_time(settings.network.pubsub_duplication_cache_time)
        .support_floodsub()
        .build()
        .map_err(anyhow::Error::msg)?;

    gossipsub::Behaviour::new(MessageAuthenticity::Signed(keypair), gossipsub_config)
        .map_err(anyhow::Error::msg)
}
