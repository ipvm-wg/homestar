#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("pubsub error: {0}")]
    PubSubError(#[from] PubSubError),
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum PubSubError {
    #[error("insufficient peers subscribed to topic {0} for publishing")]
    InsufficientPeers(String),
    #[error("not enabled")]
    NotEnabled,
    #[error(transparent)]
    SubscriptionError(#[from] libp2p::gossipsub::SubscriptionError),
}
