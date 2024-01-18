//! Ipfs Client container for an [Arc]'ed [IpfsClient].
//!
//! [IpfsClient]: ipfs_api::IpfsClient

use crate::settings;
use anyhow::Result;
use futures::TryStreamExt;
use homestar_invocation::Receipt;
use http::uri::Scheme;
use ipfs_api::{
    request::{DagCodec, DagPut},
    response::DagPutResponse,
    IpfsApi, IpfsClient,
};
use ipfs_api_backend_hyper::TryFromUri;
use libipld::{Cid, Ipld};
use std::{io::Cursor, sync::Arc};
use url::Url;

const SHA3_256: &str = "sha3-256";

/// [IpfsClient]-wrapper.
#[allow(missing_debug_implementations)]
pub(crate) struct IpfsCli(Arc<IpfsClient>);

impl IpfsCli {
    /// Create a new [IpfsCli] from a [IpfsClient].
    pub(crate) fn new(settings: &settings::Ipfs) -> Result<Self> {
        let cli = Self(Arc::new(IpfsClient::from_host_and_port(
            Scheme::HTTP,
            settings.host.as_str(),
            settings.port,
        )?));
        Ok(cli)
    }
}

impl Clone for IpfsCli {
    fn clone(&self) -> Self {
        IpfsCli(Arc::clone(&self.0))
    }
}

impl IpfsCli {
    /// Retrieve content from a IPFS [Url].
    #[allow(dead_code)]
    pub(crate) async fn get_resource(&self, url: &Url) -> Result<Vec<u8>> {
        let cid = Cid::try_from(url.to_string())?;
        self.get_cid(cid).await
    }

    /// Retrieve content from a [Cid].
    #[allow(dead_code)]
    pub(crate) async fn get_cid(&self, cid: Cid) -> Result<Vec<u8>> {
        self.0
            .cat(&cid.to_string())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(Into::into)
    }

    /// Put/Write [Receipt] into IPFS.
    #[allow(dead_code)]
    pub(crate) async fn put_receipt(&self, receipt: Receipt<Ipld>) -> Result<String> {
        let receipt_bytes: Vec<u8> = receipt.try_into()?;
        self.put_receipt_bytes(receipt_bytes).await
    }

    /// Put/Write [Receipt], as bytes, into IPFS.
    pub(crate) async fn put_receipt_bytes(&self, receipt_bytes: Vec<u8>) -> Result<String> {
        let dag_builder = DagPut::builder()
            .store_codec(DagCodec::Cbor)
            .input_codec(DagCodec::Cbor)
            .hash(SHA3_256) // sadly no support for blake3-256
            .build();

        let DagPutResponse { cid } = self
            .0
            .dag_put_with_options(Cursor::new(receipt_bytes), dag_builder)
            .await?;

        Ok(cid.cid_string)
    }
}
