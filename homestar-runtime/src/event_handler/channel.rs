//! Wrapper around [crossbeam::channel] and [flume::bounded] to provide common
//! interfaces for sync/async bounded and non-tokio "oneshot" channels.

use crossbeam::channel;

/// Sender for a bounded [crossbeam::channel].
pub type BoundedChannelSender<T> = channel::Sender<T>;

/// Receiver for a bounded [crossbeam::channel].
pub type BoundedChannelReceiver<T> = channel::Receiver<T>;

/// A bounded [crossbeam::channel] with a sender and receiver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundedChannel<T> {
    /// Sender for the channel.
    tx: channel::Sender<T>,
    /// REceiver for the channel.
    rx: channel::Receiver<T>,
}

impl<T> BoundedChannel<T> {
    /// Create a new [BoundedChannel] with a given capacity.
    pub fn with(capacity: usize) -> (BoundedChannelSender<T>, BoundedChannelReceiver<T>) {
        let (tx, rx) = channel::bounded(capacity);
        (tx, rx)
    }

    /// Create a oneshot (1) [BoundedChannel].
    pub fn oneshot() -> (BoundedChannelSender<T>, BoundedChannelReceiver<T>) {
        let (tx, rx) = channel::bounded(1);
        (tx, rx)
    }
}

/// [flume::Sender] for a bounded [flume::bounded] channel.
pub type AsyncBoundedChannelSender<T> = flume::Sender<T>;

/// [flume::Receiver] for a bounded [flume::bounded] channel.
pub type AsyncBoundedChannelReceiver<T> = flume::Receiver<T>;

/// A bounded [flume] channel with sender and receiver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsyncBoundedChannel<T> {
    /// Sender for the channel.
    tx: flume::Sender<T>,
    /// REceiver for the channel.
    rx: flume::Receiver<T>,
}

impl<T> AsyncBoundedChannel<T> {
    /// Create a new [AsyncBoundedChannel] with a given capacity.
    pub fn with(capacity: usize) -> (AsyncBoundedChannelSender<T>, AsyncBoundedChannelReceiver<T>) {
        let (tx, rx) = flume::bounded(capacity);
        (tx, rx)
    }

    /// Create a oneshot (1) [BoundedChannel].
    pub fn oneshot() -> (AsyncBoundedChannelSender<T>, AsyncBoundedChannelReceiver<T>) {
        let (tx, rx) = flume::bounded(1);
        (tx, rx)
    }
}
