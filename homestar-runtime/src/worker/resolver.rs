use super::{poller::Poller, Poll};
use crate::{
    channel::AsyncChannel,
    db::Database,
    event_handler::{
        event::QueryRecord,
        swarm_event::{FoundEvent, ResponseEvent},
        Event,
    },
    network::swarm::CapsuleTag,
    workflow::Resource,
    Db,
};
use anyhow::{bail, Result};
use fnv::FnvHashSet;
use homestar_invocation::{error::ResolveError, task};
use homestar_wasm::io::Arg;
use homestar_workflow::LinkMap;
use indexmap::IndexMap;
use libipld::{Cid, Ipld};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::RwLock,
    time::{timeout_at, Instant},
};
use tracing::{debug, instrument};

pub(crate) trait Resolver {
    async fn resolve(
        self,
        linkmap: Arc<RwLock<LinkMap<task::Result<Arg>>>>,
        resources: Arc<RwLock<IndexMap<Resource, Vec<u8>>>>,
        db: impl Database,
    ) -> Result<task::Result<Arg>, ResolveError>;
}

impl Resolver for Cid {
    #[instrument(level = "debug", name = "cid_resolve", skip_all)]
    async fn resolve(
        self,
        linkmap: Arc<RwLock<LinkMap<task::Result<Arg>>>>,
        resources: Arc<RwLock<IndexMap<Resource, Vec<u8>>>>,
        db: impl Database,
    ) -> Result<task::Result<Arg>, ResolveError> {
        if let Some(result) = linkmap.read().await.get(&self) {
            debug!(
                subject = "worker.resolve_cid",
                category = "worker.run",
                cid = self.to_string(),
                "found CID in in-memory linkmap"
            );

            Ok(result.to_owned())
        } else if let Some(bytes) = resources.read().await.get(&Resource::Cid(self)) {
            debug!(
                subject = "worker.resolve_cid",
                category = "worker.run",
                cid = self.to_string(),
                "found CID in map of resources"
            );

            Ok(task::Result::Ok(Arg::Ipld(Ipld::Bytes(bytes.to_vec()))))
        } else {
            let conn = &mut db.conn()?;
            match Db::find_instruction_by_cid(self, conn) {
                Ok(found) => Ok(found.output_as_arg()),
                Err(_) => {
                    debug!(
                        subject = "worker.resolve_cid",
                        category = "worker.run",
                        cid = self.to_string(),
                        "no related instruction receipt found in the DB"
                    );
                    Err(ResolveError::UnresolvedCid((self).to_string()))
                }
            }
        }
    }
}

/// A resolver for CIDs that may be available on the DHT.
pub(crate) struct DHTResolver {
    cids: Arc<FnvHashSet<Cid>>,
    p2p_receipt_timeout: Duration,
    workflow_cid: Cid,
}

impl DHTResolver {
    /// Create a new [DHTResolver].
    pub(crate) fn new(
        cids: Arc<FnvHashSet<Cid>>,
        p2p_receipt_timeout: Duration,
        workflow_cid: Cid,
    ) -> Self {
        Self {
            cids,
            p2p_receipt_timeout,
            workflow_cid,
        }
    }
}

impl<DB> Poll<DB> for DHTResolver
where
    DB: Database,
{
    async fn poll(&self, ctx: &Poller<DB>) -> Result<()> {
        for cid in self.cids.iter() {
            let (tx, rx) = AsyncChannel::oneshot();

            let _ = ctx
                .event_sender
                .send_async(Event::FindRecord(QueryRecord::with(
                    *cid,
                    CapsuleTag::Receipt,
                    Some(tx),
                )))
                .await;

            let found = match timeout_at(Instant::now() + self.p2p_receipt_timeout, rx.recv_async())
                .await
            {
                Ok(Ok(ResponseEvent::Found(Ok(FoundEvent::Receipt(found))))) => found,
                Ok(Ok(ResponseEvent::Found(Err(err)))) => {
                    bail!(ResolveError::UnresolvedCid(format!(
                        "failure in attempting to find event: {err}"
                    )))
                }
                Ok(Ok(_)) => bail!(ResolveError::UnresolvedCid(
                    "wrong or unexpected event message received".to_string(),
                )),
                Ok(Err(err)) => bail!(ResolveError::UnresolvedCid(format!(
                    "unexpected error while trying to resolve cid: {err}",
                ))),
                Err(_) => bail!(ResolveError::UnresolvedCid(
                    "timed out while trying to resolve cid".to_string(),
                )),
            };

            let conn = &mut ctx.db.conn()?;

            let receipt = Db::commit_receipt(self.workflow_cid, found.clone().receipt, conn)
                .unwrap_or(found.clone().receipt);

            debug!(
                subject = "db.commit_receipt",
                category = "dht.resolver",
                cid_resolved = cid.to_string(),
                receipt_cid = receipt.cid().to_string(),
                "committed to database"
            );

            let found_result = receipt.output_as_arg();

            // Store the result in the linkmap for use in next iterations.
            if let Some(ref m) = ctx.linkmap {
                m.write().await.entry(*cid).or_insert_with(|| found_result);
            }

            // retrieval mechanism.
            #[cfg(feature = "websocket-notify")]
            let _ = ctx
                .event_sender
                .send_async(Event::StoredRecord(FoundEvent::Receipt(found)))
                .await;
        }

        Ok(())
    }
}
