//! Ipfs Client container for an [Arc]'ed [IpfsClient].

use anyhow::Result;
use futures::TryStreamExt;
use homestar_core::workflow::Receipt;
use ipfs_api::{
    request::{DagCodec, DagPut},
    response::DagPutResponse,
    IpfsApi, IpfsClient,
};
use libipld::{Cid, Ipld};
use std::{io::Cursor, sync::Arc};
use url::Url;

const SHA3_256: &str = "sha3-256";

/// [IpfsClient]-wrapper.
#[allow(missing_debug_implementations)]
pub struct IpfsCli(Arc<IpfsClient>);

impl Clone for IpfsCli {
    fn clone(&self) -> Self {
        IpfsCli(Arc::clone(&self.0))
    }
}

impl Default for IpfsCli {
    fn default() -> Self {
        Self(Arc::new(IpfsClient::default()))
    }
}

impl IpfsCli {
    /// Retrieve content from a [Url].
    pub async fn get_resource(&self, url: &Url) -> Result<Vec<u8>> {
        let cid = Cid::try_from(url.to_string())?;
        self.get_cid(cid).await
    }

    /// Retrieve content from a [Cid].
    pub async fn get_cid(&self, cid: Cid) -> Result<Vec<u8>> {
        self.0
            .cat(&cid.to_string())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(Into::into)
    }

    /// Put/Write [Receipt] into IPFS.
    pub async fn put_receipt(&self, receipt: Receipt<Ipld>) -> Result<String> {
        let receipt_bytes: Vec<u8> = receipt.try_into()?;
        self.put_receipt_bytes(receipt_bytes).await
    }

    /// Put/Write [Receipt], as bytes, into IPFS.
    pub async fn put_receipt_bytes(&self, receipt_bytes: Vec<u8>) -> Result<String> {
        let dag_builder = DagPut::builder()
            .store_codec(DagCodec::Cbor)
            .input_codec(DagCodec::Cbor)
            .hash(SHA3_256) // sadly no support for blake3-256
            .build();

        let DagPutResponse { cid } = self
            .0
            .dag_put_with_options(Cursor::new(receipt_bytes.clone()), dag_builder)
            .await
            .expect("a CID");

        Ok(cid.cid_string)
    }
}
