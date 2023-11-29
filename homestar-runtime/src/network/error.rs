//! # Error types centered around the networking.

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
    Publish(#[from] libp2p::gossipsub::PublishError),
    #[error(transparent)]
    Subscription(#[from] libp2p::gossipsub::SubscriptionError),
    // TODO: We may be able to remove this error type once we clean-up erroring
    // through the runtime.
    #[error("error on conversion: {0}")]
    Conversion(#[from] anyhow::Error),
}
