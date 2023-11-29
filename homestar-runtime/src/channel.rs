//! Wrapper around [crossbeam::channel] and [flume] to provide common
//! interfaces for sync/async (un)bounded and non-tokio "oneshot" channels.

use crossbeam::channel;

/// Sender for a bounded [crossbeam::channel].
pub type BoundedChannelSender<T> = channel::Sender<T>;

/// Receiver for a bounded [crossbeam::channel].
pub type BoundedChannelReceiver<T> = channel::Receiver<T>;

/// A bounded [crossbeam::channel] with a sender and receiver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Channel<T> {
    /// Sender for the channel.
    tx: channel::Sender<T>,
    /// REceiver for the channel.
    rx: channel::Receiver<T>,
}

impl<T> Channel<T> {
    /// Create a new [Channel] with a given capacity.
    pub fn with(capacity: usize) -> (BoundedChannelSender<T>, BoundedChannelReceiver<T>) {
        let (tx, rx) = channel::bounded(capacity);
        (tx, rx)
    }

    /// Create a oneshot (1) [Channel].
    pub fn oneshot() -> (BoundedChannelSender<T>, BoundedChannelReceiver<T>) {
        let (tx, rx) = channel::bounded(1);
        (tx, rx)
    }
}

/// [flume::Sender] for a bounded [flume::bounded] channel.
pub type AsyncChannelSender<T> = flume::Sender<T>;

/// [flume::Receiver] for a bounded [flume::bounded] channel.
pub type AsyncChannelReceiver<T> = flume::Receiver<T>;

/// A bounded [flume] channel with sender and receiver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsyncChannel<T> {
    /// Sender for the channel.
    tx: flume::Sender<T>,
    /// REceiver for the channel.
    rx: flume::Receiver<T>,
}

impl<T> AsyncChannel<T> {
    /// Create a new [AsyncChannel] with a given capacity.
    pub fn with(capacity: usize) -> (AsyncChannelSender<T>, AsyncChannelReceiver<T>) {
        let (tx, rx) = flume::bounded(capacity);
        (tx, rx)
    }

    /// Create an unbounded [AsyncChannel].
    pub fn unbounded() -> (AsyncChannelSender<T>, AsyncChannelReceiver<T>) {
        let (tx, rx) = flume::unbounded();
        (tx, rx)
    }

    /// Create a oneshot (1) [Channel].
    pub fn oneshot() -> (AsyncChannelSender<T>, AsyncChannelReceiver<T>) {
        let (tx, rx) = flume::bounded(1);
        (tx, rx)
    }
}
