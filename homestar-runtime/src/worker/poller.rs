//! Poller for workflow execution on pending tasks.

use crate::{channel::AsyncChannelSender, db::Database, event_handler::Event};
use anyhow::Result;
use homestar_invocation::task;
use homestar_wasm::io::Arg;
use std::{future::Future, sync::Arc, time::Duration};
use tokio::{runtime::Handle, sync::RwLock};

type LinkMap = Arc<RwLock<homestar_workflow::LinkMap<task::Result<Arg>>>>;

/// Poller context for working with state.
pub(crate) struct Poller<DB: Database> {
    pub(crate) db: DB,
    pub(crate) event_sender: Arc<AsyncChannelSender<Event>>,
    pub(crate) linkmap: Option<LinkMap>,
}

/// Poll (once) eagerly when called (in the background).
pub(crate) async fn poll<P: Poll<DB> + Send + 'static, DB: Database + 'static>(
    actor: P,
    db: DB,
    event_sender: Arc<AsyncChannelSender<Event>>,
    linkmap: Option<LinkMap>,
) {
    let poller = Poller::new(db, event_sender, linkmap);
    let handle = Handle::current();
    handle.spawn(async move {
        let _ = actor.poll(&poller).await;
    });
}

/// Start a poller at a given interval which runs in the background.
#[allow(dead_code)]
pub(crate) async fn poll_at_interval<
    P: Poll<DB> + Send + Sync + Clone + 'static,
    DB: Database + 'static,
>(
    actor: P,
    db: DB,
    event_sender: Arc<AsyncChannelSender<Event>>,
    interval: Duration,
    linkmap: Option<LinkMap>,
) {
    let mut interval = tokio::time::interval(interval);
    let poller = Arc::new(Poller::new(db, event_sender, linkmap));
    let handle = Handle::current();
    handle.spawn(async move {
        loop {
            interval.tick().await;
            let poller_clone = Arc::clone(&poller);
            let _ = actor.poll(Arc::as_ref(&poller_clone)).await;
        }
    });
}

impl<DB> Poller<DB>
where
    DB: Database,
{
    /// Create a new [Poller].
    fn new(db: DB, event_sender: Arc<AsyncChannelSender<Event>>, linkmap: Option<LinkMap>) -> Self {
        Self {
            db,
            event_sender,
            linkmap,
        }
    }
}

/// Trait for polling a resource.
pub(crate) trait Poll<DB>
where
    DB: Database,
{
    /// Poll for work.
    fn poll(&self, ctx: &Poller<DB>) -> impl Future<Output = Result<()>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        channel::{AsyncChannel, AsyncChannelSender},
        test_utils::db::MemoryDb,
        Settings,
    };

    #[derive(Debug, Clone)]
    struct TestResolver(AsyncChannelSender<u32>);

    impl<DB> Poll<DB> for TestResolver
    where
        DB: Database,
    {
        async fn poll(&self, _ctx: &Poller<DB>) -> Result<()> {
            let _ = self.0.send_async(1).await;
            Ok(())
        }
    }

    #[tokio::test]
    async fn polls_once() {
        let (tx, rx) = AsyncChannel::with(1);
        poll(
            TestResolver(tx),
            MemoryDb::setup_connection_pool(Settings::load().unwrap().node(), None).unwrap(),
            Arc::new(AsyncChannel::with(1).0),
            None,
        )
        .await;

        let msg = rx.recv_async().await.unwrap();
        assert_eq!(msg, 1);
        assert!(rx.try_recv().is_err())
    }

    #[tokio::test]
    async fn polls_at_interval() {
        let (tx, rx) = AsyncChannel::with(1);
        poll_at_interval(
            TestResolver(tx),
            MemoryDb::setup_connection_pool(Settings::load().unwrap().node(), None).unwrap(),
            Arc::new(AsyncChannel::with(1).0),
            Duration::from_millis(10),
            None,
        )
        .await;

        tokio::time::sleep(Duration::from_millis(20)).await;

        let msg1 = rx.recv_async().await.unwrap();
        assert_eq!(msg1, 1);
        let msg2 = rx.recv_async().await.unwrap();
        assert_eq!(msg2, 1);
    }
}
