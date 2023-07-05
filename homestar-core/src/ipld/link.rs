//! Typed cid for custom links.
//!
//! Extracted from [libipld::Link] to allow for custom de/serialization on
//! custom types.

use libipld::{
    codec::{Codec, Decode, Encode},
    error, Cid,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    io::{Read, Seek, Write},
    marker::PhantomData,
    ops::Deref,
};

/// Typed cid.
#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Link<T> {
    cid: Cid,
    _marker: PhantomData<T>,
}

impl<T> Link<T> {
    /// Creates a new `Link`.
    pub fn new(cid: Cid) -> Self {
        Self {
            cid,
            _marker: PhantomData,
        }
    }

    /// Returns a reference to the cid.
    pub fn cid(&self) -> &Cid {
        &self.cid
    }
}

impl<T> fmt::Display for Link<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cid.fmt(f)
    }
}

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Link<T> {}

impl<T> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cid.eq(other.cid())
    }
}

impl<T> Eq for Link<T> {}

impl<T> PartialOrd for Link<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cid.cmp(other.cid()))
    }
}

impl<T> Ord for Link<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cid.cmp(other.cid())
    }
}

impl<T> Hash for Link<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Hash::hash(self.cid(), hasher)
    }
}

impl<C: Codec, T> Encode<C> for Link<T>
where
    Cid: Encode<C>,
{
    fn encode<W: Write>(&self, c: C, w: &mut W) -> error::Result<()> {
        self.cid().encode(c, w)
    }
}

impl<C: Codec, T> Decode<C> for Link<T>
where
    Cid: Decode<C>,
{
    fn decode<R: Read + Seek>(c: C, r: &mut R) -> error::Result<Self> {
        Ok(Self::new(Cid::decode(c, r)?))
    }
}

impl<T> Deref for Link<T> {
    type Target = Cid;

    fn deref(&self) -> &Self::Target {
        self.cid()
    }
}

impl<T> AsRef<Cid> for Link<T> {
    fn as_ref(&self) -> &Cid {
        self.cid()
    }
}

impl<T> From<Cid> for Link<T> {
    fn from(cid: Cid) -> Self {
        Self::new(cid)
    }
}
