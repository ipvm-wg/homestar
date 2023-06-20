//! [gossipsub] initializer for PubSub across connected peers.

use anyhow::Result;
use libp2p::{
    gossipsub::{self, ConfigBuilder, Message, MessageAuthenticity, MessageId, ValidationMode},
    identity::Keypair,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};

use crate::settings;

/// Setup [gossipsub] mesh protocol with default configuration.
pub fn new(keypair: Keypair, settings: &settings::Node) -> Result<gossipsub::Behaviour> {
    // To content-address message, we can take the hash of message and use it as an ID.
    let message_id_fn = |message: &Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(settings.network.pubsub_heartbeat_secs))
        .idle_timeout(Duration::from_secs(
            settings.network.pubsub_idle_timeout_secs,
        ))
        // This sets the kind of message validation. The default is Strict (enforce message signing).
        .validation_mode(ValidationMode::Strict)
        .mesh_n_low(1)
        .mesh_outbound_min(1)
        .mesh_n(2)
        // Content-address messages. No two messages of the same content will be propagated.
        .message_id_fn(message_id_fn)
        .duplicate_cache_time(Duration::from_secs(
            settings.network.pubsub_duplication_cache_secs,
        ))
        .support_floodsub()
        .build()
        .map_err(anyhow::Error::msg)?;

    gossipsub::Behaviour::new(MessageAuthenticity::Signed(keypair), gossipsub_config)
        .map_err(anyhow::Error::msg)
}
