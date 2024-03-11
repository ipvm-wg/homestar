//! [Instruction] nonce parameter.
//!
//! [Instruction]: super::Instruction

use crate::{ipld::schema, Error, Unit};
use const_format::formatcp;
use enum_as_inner::EnumAsInner;
use generic_array::{
    typenum::consts::{U12, U16},
    GenericArray,
};
use libipld::{multibase::Base::Base32HexLower, Ipld};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, Schema, SchemaObject, SingleOrVec, StringValidation},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{borrow::Cow, fmt, module_path};
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
        match ipld {
            Ipld::String(s) if s.is_empty() => Ok(Nonce::Empty),
            Ipld::String(s) => {
                let bytes = Base32HexLower.decode(s)?;
                match bytes.len() {
                    12 => Ok(Nonce::Nonce96(*GenericArray::from_slice(&bytes))),
                    16 => Ok(Nonce::Nonce128(*GenericArray::from_slice(&bytes))),
                    other => Err(Error::unexpected_ipld(other.to_owned().into())),
                }
            }
            Ipld::Bytes(v) => match v.len() {
                12 => Ok(Nonce::Nonce96(*GenericArray::from_slice(&v))),
                16 => Ok(Nonce::Nonce128(*GenericArray::from_slice(&v))),
                other_ipld => Err(Error::unexpected_ipld(other_ipld.to_owned().into())),
            },
            _ => Ok(Nonce::Empty),
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
        Cow::Borrowed(formatcp!("{}::Nonce", module_path!()))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject {
            instance_type: None,
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A 12-byte or 16-byte nonce encoded as IPLD bytes. Use empty string for no nonce.".to_string(),
                ),
                ..Default::default()
            })),
            ..Default::default()
        };

        let empty_string = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
            const_value: Some(json!("")),
            ..Default::default()
        };

        let non_empty_string = SchemaObject {
            instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
            metadata: Some(Box::new(Metadata {
                description: Some("A 12-byte or 16-byte nonce encoded as a string, which expects to be decoded with Base32hex lower".to_string()),
                ..Default::default()
            })),
            string: Some(Box::new(StringValidation {
                min_length: Some(1),
                ..Default::default()
            })),
            ..Default::default()
        };

        schema.subschemas().one_of = Some(vec![
            gen.subschema_for::<schema::IpldBytesStub>(),
            Schema::Object(empty_string),
            Schema::Object(non_empty_string),
        ]);

        schema.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libipld::{json::DagJsonCodec, multibase::Base, prelude::Codec};

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

    #[test]
    fn json_nonce_roundtrip() {
        let b = b"LSS3Ftv+Gtq9965M";
        let bytes = Base::Base64.encode(b);
        let json = json!({
            "/": {"bytes": format!("{}", bytes)}
        });

        let ipld: Ipld = DagJsonCodec.decode(json.to_string().as_bytes()).unwrap();
        let nonce: Nonce = Nonce::try_from(ipld.clone()).unwrap();

        let Ipld::Bytes(bytes) = ipld.clone() else {
            panic!("IPLD is not bytes");
        };

        assert_eq!(bytes, b);
        assert_eq!(ipld, Ipld::Bytes(b.to_vec()));
        assert_eq!(nonce, Nonce::Nonce128(*GenericArray::from_slice(b)));
        assert_eq!(nonce, Nonce::try_from(ipld.clone()).unwrap());

        let nonce: Nonce = ipld.clone().try_into().unwrap();
        let ipld = Ipld::from(nonce.clone());
        assert_eq!(ipld, Ipld::Bytes(b.to_vec()));
    }

    #[test]
    fn nonce_as_string_roundtrip() {
        let nonce = Nonce::generate();
        let string = nonce.to_string();
        let from_string = Nonce::try_from(Ipld::String(string.clone())).unwrap();

        assert_eq!(nonce, from_string);
        assert_eq!(string, nonce.to_string());
    }

    #[test]
    fn json_nonce_string_roundtrip() {
        let in_nnc = "1sod60ml6g26mfhsrsa0";
        let json = json!({
            "nnc": in_nnc
        });

        let ipld: Ipld = DagJsonCodec.decode(json.to_string().as_bytes()).unwrap();
        let Ipld::Map(map) = ipld.clone() else {
            panic!("IPLD is not a map");
        };
        let nnc = map.get("nnc").unwrap();
        let nnc: Nonce = Nonce::try_from(nnc.clone()).unwrap();
        assert_eq!(nnc.to_string(), in_nnc);
        let nonce = Nonce::Nonce96(*GenericArray::from_slice(
            Base32HexLower.decode(in_nnc).unwrap().as_slice(),
        ));
        assert_eq!(nnc, nonce);
    }
}
