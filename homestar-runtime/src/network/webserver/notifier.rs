//! Notifier for broadcasting messages to websocket clients.

use anyhow::Result;
use faststr::FastStr;
use libipld::Cid;
use std::{fmt, sync::Arc};

/// Type-wrapper for WebSocket sender.
#[derive(Debug)]
pub(crate) struct Notifier<T: Clone>(
    Arc<async_broadcast::Sender<T>>,
    Arc<async_broadcast::Receiver<T>>,
);

impl<T> Clone for Notifier<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T> Notifier<T>
where
    T: Send + Sync + Clone + fmt::Debug + 'static,
{
    /// Create a new [Notifier].
    pub(crate) fn new(
        sender: async_broadcast::Sender<T>,
        receiver: async_broadcast::Receiver<T>,
    ) -> Self {
        Self(sender.into(), receiver.into())
    }

    pub(crate) fn subscriber(&self) -> Arc<async_broadcast::Receiver<T>> {
        self.1.clone()
    }

    /// Send a message to all connected WebSocket clients.
    pub(crate) fn notify(&self, msg: T) -> Result<()> {
        let _ = self.0.try_broadcast(msg)?;
        Ok(())
    }
}

/// Subscription type: either directed via a Cid or an event subscription string.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum SubscriptionTyp {
    EventSub(String),
    Cid(Cid),
}

/// A header for a message to be sent to a WebSocket client.
#[derive(Debug, Clone)]
pub(crate) struct Header {
    pub(crate) subscription: SubscriptionTyp,
    pub(crate) ident: Option<FastStr>,
}

impl Header {
    /// Create a new [Header].
    pub(crate) fn new(sub: SubscriptionTyp, ident: Option<FastStr>) -> Self {
        Self {
            subscription: sub,
            ident,
        }
    }
}

/// A message to be sent to a WebSocket client, with a header and payload.
#[derive(Debug, Clone)]
pub(crate) struct Message {
    pub(crate) header: Header,
    pub(crate) payload: Vec<u8>,
}

impl Message {
    /// Create a new [Message].
    pub(crate) fn new(header: Header, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    /// Get a reference to the [Header] of a [Message].
    #[allow(dead_code)]
    pub(crate) fn header(&self) -> &Header {
        &self.header
    }

    /// Get a reference to the payload of a [Message].
    #[allow(dead_code)]
    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload
    }
}
