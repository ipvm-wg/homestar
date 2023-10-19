#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("pubsub error: {0}")]
    PubSubError(#[from] PubSubError),
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum PubSubError {
    #[error("not enabled")]
    NotEnabled,
    #[error(transparent)]
    SubscriptionError(#[from] libp2p::gossipsub::SubscriptionError),
}
