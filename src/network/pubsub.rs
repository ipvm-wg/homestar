use anyhow::Result;
use libp2p::{
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubMessage, MessageAuthenticity, MessageId,
        ValidationMode,
    },
    identity::Keypair,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};

pub fn new(keypair: Keypair) -> Result<Gossipsub> {
    // To content-address message, we can take the hash of message and use it as an ID.
    let message_id_fn = |message: &GossipsubMessage| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = GossipsubConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        // This sets the kind of message validation. The default is Strict (enforce message signing).
        .validation_mode(ValidationMode::Strict)
        .mesh_n_low(1)
        .mesh_outbound_min(1)
        .mesh_n(2)
        // Content-address messages. No two messages of the same content will be propagated.
        .message_id_fn(message_id_fn)
        .build()
        .map_err(anyhow::Error::msg)?;

    Gossipsub::new(MessageAuthenticity::Signed(keypair), gossipsub_config)
        .map_err(anyhow::Error::msg)
}