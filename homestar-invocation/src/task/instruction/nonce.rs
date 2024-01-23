//! [Instruction] nonce parameter.
//!
//! [Instruction]: super::Instruction

use crate::{Error, Unit};
use enum_as_inner::EnumAsInner;
use generic_array::{
    typenum::consts::{U12, U16},
    GenericArray,
};
use libipld::{multibase::Base::Base32HexLower, Ipld};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};
use uuid::Uuid;

type Nonce96 = GenericArray<u8, U12>;
type Nonce128 = GenericArray<u8, U16>;

/// Enumeration over allowed `nonce` types.
#[derive(Clone, Debug, PartialEq, EnumAsInner, Serialize, Deserialize)]
pub enum Nonce {
    /// 96-bit, 12-byte nonce, e.g. [xid].
    Nonce96(Nonce96),
    /// 128-bit, 16-byte nonce.
    Nonce128(Nonce128),
    /// No Nonce attributed.
    Empty,
}

impl Nonce {
    /// Default generator, outputting a [xid] nonce, which is a 96-bit, 12-byte
    /// nonce.
    pub fn generate() -> Self {
        Nonce::Nonce96(*GenericArray::from_slice(xid::new().as_bytes()))
    }

    /// Generate a default, 128-bit, 16-byte nonce via [Uuid::new_v4()].
    pub fn generate_128() -> Self {
        Nonce::Nonce128(*GenericArray::from_slice(Uuid::new_v4().as_bytes()))
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Nonce::Nonce96(nonce) => {
                write!(f, "{}", Base32HexLower.encode(nonce.as_slice()))
            }
            Nonce::Nonce128(nonce) => {
                write!(f, "{}", Base32HexLower.encode(nonce.as_slice()))
            }
            Nonce::Empty => write!(f, ""),
        }
    }
}

impl From<Nonce> for Ipld {
    fn from(nonce: Nonce) -> Self {
        match nonce {
            Nonce::Nonce96(nonce) => Ipld::Bytes(nonce.to_vec()),
            Nonce::Nonce128(nonce) => Ipld::Bytes(nonce.to_vec()),
            Nonce::Empty => Ipld::String("".to_string()),
        }
    }
}

impl TryFrom<Ipld> for Nonce {
    type Error = Error<Unit>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Bytes(v) = ipld {
            match v.len() {
                12 => Ok(Nonce::Nonce96(*GenericArray::from_slice(&v))),
                16 => Ok(Nonce::Nonce128(*GenericArray::from_slice(&v))),
                other_ipld => Err(Error::unexpected_ipld(other_ipld.to_owned().into())),
            }
        } else {
            Ok(Nonce::Empty)
        }
    }
}

impl TryFrom<&Ipld> for Nonce {
    type Error = Error<Unit>;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

impl JsonSchema for Nonce {
    fn schema_name() -> String {
        "nonce".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("homestar-invocation::task::instruction::Nonce")
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let schema = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A 12-byte or 16-byte nonce. Use empty string for no nonce.".to_string(),
                ),
                ..Default::default()
            })),
            ..Default::default()
        };
        schema.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ipld_roundtrip_12() {
        let gen = Nonce::generate();
        let ipld = Ipld::from(gen.clone());

        let inner = if let Nonce::Nonce96(nonce) = gen {
            Ipld::Bytes(nonce.to_vec())
        } else {
            panic!("No conversion!")
        };

        assert_eq!(ipld, inner);
        assert_eq!(gen, ipld.try_into().unwrap());
    }

    #[test]
    fn ipld_roundtrip_16() {
        let gen = Nonce::generate_128();
        let ipld = Ipld::from(gen.clone());

        let inner = if let Nonce::Nonce128(nonce) = gen {
            Ipld::Bytes(nonce.to_vec())
        } else {
            panic!("No conversion!")
        };

        assert_eq!(ipld, inner);
        assert_eq!(gen, ipld.try_into().unwrap());
    }

    #[test]
    fn ser_de() {
        let gen = Nonce::generate_128();
        let ser = serde_json::to_string(&gen).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(gen, de);
    }
}
