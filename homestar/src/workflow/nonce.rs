//! [Task] Nonce parameter.
//!
//! [Task]: super::Task

use anyhow::anyhow;
use enum_as_inner::EnumAsInner;
use generic_array::{
    typenum::consts::{U12, U16},
    GenericArray,
};
use libipld::Ipld;

type Nonce96 = GenericArray<u8, U12>;
type Nonce128 = GenericArray<u8, U16>;

/// Enumeration over allowed `nonce` types.
#[derive(Clone, Debug, PartialEq, EnumAsInner)]
pub enum Nonce {
    /// 96-bit, 12-byte nonce, e.g. [xid].
    Nonce96(Nonce96),
    /// 129-bit, 16-byte nonce.
    Nonce128(Nonce128),
}

impl Nonce {
    /// Default generator, outputting a [xid] nonce.
    pub fn generate() -> Nonce {
        Nonce::Nonce96(*GenericArray::from_slice(xid::new().as_bytes()))
    }
}

impl From<Nonce> for Ipld {
    fn from(nonce: Nonce) -> Self {
        match nonce {
            Nonce::Nonce96(nonce) => {
                Ipld::List(vec![Ipld::Integer(0), Ipld::Bytes(nonce.to_vec())])
            }
            Nonce::Nonce128(nonce) => {
                Ipld::List(vec![Ipld::Integer(1), Ipld::Bytes(nonce.to_vec())])
            }
        }
    }
}

impl TryFrom<Ipld> for Nonce {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::List(v) = ipld {
            match &v[..] {
                [Ipld::Integer(0), Ipld::Bytes(nonce)] => {
                    Ok(Nonce::Nonce96(*GenericArray::from_slice(nonce)))
                }
                [Ipld::Integer(1), Ipld::Bytes(nonce)] => {
                    Ok(Nonce::Nonce128(*GenericArray::from_slice(nonce)))
                }
                _ => Err(anyhow!("unexpected conversion type")),
            }
        } else {
            Err(anyhow!("mismatched conversion type: {ipld:?}"))
        }
    }
}

impl TryFrom<&Ipld> for Nonce {
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ipld_roundtrip() {
        let gen = Nonce::generate();
        let ipld = Ipld::from(gen.clone());

        let inner = if let Nonce::Nonce96(nonce) = gen {
            Ipld::Bytes(nonce.to_vec())
        } else {
            panic!("No conversion!")
        };

        assert_eq!(ipld, Ipld::List(vec![Ipld::Integer(0), inner]));
        assert_eq!(gen, ipld.try_into().unwrap());
    }
}
