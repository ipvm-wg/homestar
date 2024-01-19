//! Traits related to Ipld and DagCbor encoding/decoding.

use crate::{consts::DAG_CBOR, Error, Unit};
use libipld::{
    cbor::DagCborCodec,
    multihash::{Code, MultihashDigest},
    prelude::Codec,
    Cid, Ipld,
};

/// Trait for DagCbor-related encode/decode.
pub trait DagCbor
where
    Self: Sized,
    Ipld: From<Self>,
{
    /// Performs the conversion from an owned `Self` to Cid.
    fn to_cid(self) -> Result<Cid, Error<Unit>> {
        let ipld: Ipld = self.into();
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(DAG_CBOR, hash))
    }
}

/// Trait for DagCbor-related encode/decode for references.
pub trait DagCborRef
where
    Self: Sized,
    for<'a> Ipld: From<&'a Self>,
{
    /// Performs the conversion from a referenced `Self` to Cid.
    fn to_cid(&self) -> Result<Cid, Error<Unit>> {
        let ipld: Ipld = self.into();
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(DAG_CBOR, hash))
    }
}
