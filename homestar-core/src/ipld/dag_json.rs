//! Traits related to [Ipld] and [DagJson] encoding/decoding.
//!
//! [DagJson]: DagJsonCodec

use crate::{workflow::Error, Unit};
use libipld::{codec::Decode, json::DagJsonCodec, prelude::Codec, Ipld};
use std::io::Cursor;

/// Trait for serializing and deserializing types to and from JSON.
pub trait DagJson
where
    Self: TryFrom<Ipld> + Clone,
    Ipld: From<Self>,
{
    /// Serialize `Self` type to JSON bytes.
    fn to_json(&self) -> Result<Vec<u8>, Error<Unit>> {
        let ipld: Ipld = self.to_owned().into();
        Ok(DagJsonCodec.encode(&ipld)?)
    }

    /// Serialize `Self` type to JSON [String].
    fn to_json_string(&self) -> Result<String, Error<Unit>> {
        let encoded = self.to_json()?;
        // JSON spec requires UTF-8 support
        let s = std::str::from_utf8(&encoded)?;
        Ok(s.to_string())
    }

    /// Deserialize `Self` type from JSON bytes.
    fn from_json(data: &[u8]) -> Result<Self, Error<Unit>> {
        let ipld = Ipld::decode(DagJsonCodec, &mut Cursor::new(data))?;
        let from_ipld = Self::try_from(ipld).map_err(|_err| {
            // re-decode with an unwrap, without a clone, as we know the data is
            // valid JSON.
            Error::<Unit>::UnexpectedIpldTypeError(
                Ipld::decode(DagJsonCodec, &mut Cursor::new(data)).unwrap(),
            )
        })?;
        Ok(from_ipld)
    }

    /// Deserialize `Self` type from JSON [String].
    fn from_json_string(json: String) -> Result<Self, Error<Unit>> {
        let data = json.as_bytes();
        Self::from_json(data)
    }
}
