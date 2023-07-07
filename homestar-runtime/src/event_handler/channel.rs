//! Wrapper around [crossbeam::channel] to provide a common interface for
//! bounded and non-tokio "oneshot" channels.

use crossbeam::channel;

/// Sender for a bounded [crossbeam::channel].
pub(crate) type BoundedChannelSender<T> = channel::Sender<T>;

/// Receiver for a bounded [crossbeam::channel].
#[allow(dead_code)]
pub(crate) type BoundedChannelReceiver<T> = channel::Receiver<T>;

/// A bounded [crossbeam::channel] with a sender and receiver.
#[derive(Debug, Clone)]
pub(crate) struct BoundedChannel<T> {
    /// Sender for the channel.
    pub(crate) tx: channel::Sender<T>,
    /// REceiver for the channel.
    pub(crate) rx: channel::Receiver<T>,
}

impl<T> BoundedChannel<T> {
    /// Create a new [BoundedChannel] with a given capacity.
    pub(crate) fn new(capacity: usize) -> Self {
        let (tx, rx) = channel::bounded(capacity);
        Self { tx, rx }
    }

    /// Create a oneshot (1) [BoundedChannel].
    pub(crate) fn oneshot() -> Self {
        let (tx, rx) = channel::bounded(1);
        Self { tx, rx }
    }
}
