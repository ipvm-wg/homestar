//! Traits related to Ipld and DagCbor encoding/decoding.

use crate::{consts::DAG_CBOR, Error, Unit};
use libipld::{
    cbor::DagCborCodec,
    json::DagJsonCodec,
    multihash::{Code, MultihashDigest},
    prelude::{Codec, Decode},
    Cid, Ipld,
};
use std::{
    fs,
    io::{Cursor, Write},
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

    /// Serialize `Self` to JSON bytes.
    fn to_dag_json(self) -> Result<Vec<u8>, Error<Unit>> {
        let ipld: Ipld = self.into();
        Ok(DagJsonCodec.encode(&ipld)?)
    }

    /// Serialize `Self` to JSON [String].
    fn to_dagjson_string(self) -> Result<String, Error<Unit>> {
        let encoded = self.to_dag_json()?;
        // JSON spec requires UTF-8 support
        let s = std::str::from_utf8(&encoded)?;
        Ok(s.to_string())
    }

    /// Serialize `Self` to CBOR bytes.
    fn to_cbor(self) -> Result<Vec<u8>, Error<Unit>> {
        let ipld: Ipld = self.into();
        Ok(DagCborCodec.encode(&ipld)?)
    }

    /// Deserialize `Self` from CBOR bytes.
    fn from_cbor(data: &[u8]) -> Result<Self, Error<Unit>>
    where
        Self: TryFrom<Ipld>,
    {
        let ipld = Ipld::decode(DagCborCodec, &mut Cursor::new(data))?;
        let from_ipld = Self::try_from(ipld).map_err(|_err| {
            Error::<Unit>::UnexpectedIpldType(Ipld::String(
                "Failed to convert Ipld to expected type".to_string(),
            ))
        })?;
        Ok(from_ipld)
    }

    /// Serialize `Self` to a CBOR file.
    fn to_cbor_file(self, filename: String) -> Result<(), Error<Unit>> {
        Ok(fs::File::create(filename)?.write_all(&self.to_cbor()?)?)
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
