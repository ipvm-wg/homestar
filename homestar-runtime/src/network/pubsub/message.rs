use anyhow::{anyhow, Result};
use homestar_core::workflow::Nonce;
use libipld::{self, cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Ipld};
use std::collections::BTreeMap;

const HEADER_KEY: &str = "header";
const PAYLOAD_KEY: &str = "payload";
const NONCE_KEY: &str = "nonce";

#[derive(Debug)]
pub(crate) struct Message<T> {
    pub(crate) header: Header,
    pub(crate) payload: T,
}

impl<T> Message<T> {
    pub(crate) fn new(payload: T) -> Self {
        let header = Header {
            nonce: Nonce::generate(),
        };

        Self { header, payload }
    }
}

impl<T> TryFrom<Message<T>> for Vec<u8>
where
    Ipld: From<Message<T>> + From<T>,
{
    type Error = anyhow::Error;

    fn try_from(message: Message<T>) -> Result<Self, Self::Error> {
        let message_ipld = Ipld::from(message);
        DagCborCodec.encode(&message_ipld)
    }
}

impl<T> TryFrom<Vec<u8>> for Message<T>
where
    Ipld: TryInto<Message<T>>,
    T: TryFrom<Vec<u8>>,
{
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let ipld: Ipld = DagCborCodec.decode(&bytes)?;
        ipld.try_into()
            .map_err(|_| anyhow!("Could not convert IPLD to pubsub message."))
    }
}

// impl TryFrom<Vec<u8>> for Message<crate::Receipt> {
//     type Error = anyhow::Error;

//     fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
//         let ipld: Ipld = DagCborCodec.decode(&bytes)?;
//         ipld.try_into()
//             .map_err(|_| anyhow!("Could not convert IPLD to pubsub message."))
//     }
// }

impl<T> From<Message<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(message: Message<T>) -> Self {
        Ipld::Map(BTreeMap::from([
            (HEADER_KEY.into(), message.header.into()),
            (PAYLOAD_KEY.into(), message.payload.into()),
        ]))
    }
}

impl<T> TryFrom<Ipld> for Message<T>
where
    T: From<Ipld>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let header = map
            .get(HEADER_KEY)
            .ok_or_else(|| anyhow!("missing {HEADER_KEY}"))?
            .to_owned()
            .try_into()?;

        let payload = map
            .get(PAYLOAD_KEY)
            .ok_or_else(|| anyhow!("missing {PAYLOAD_KEY}"))?
            .to_owned()
            .try_into()?;

        Ok(Message { header, payload })
    }
}

impl TryFrom<Ipld> for Message<crate::Receipt> {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let header = map
            .get(HEADER_KEY)
            .ok_or_else(|| anyhow!("missing {HEADER_KEY}"))?
            .to_owned()
            .try_into()?;

        let payload = map
            .get(PAYLOAD_KEY)
            .ok_or_else(|| anyhow!("missing {PAYLOAD_KEY}"))?
            .to_owned()
            .try_into()?;

        Ok(Message { header, payload })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Header {
    nonce: Nonce,
}

impl From<Header> for Ipld {
    fn from(header: Header) -> Self {
        Ipld::Map(BTreeMap::from([(
            NONCE_KEY.into(),
            header.nonce.to_owned().into(),
        )]))
    }
}

impl TryFrom<Ipld> for Header {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let nonce = map
            .get(NONCE_KEY)
            .ok_or_else(|| anyhow!("Missing {NONCE_KEY}"))?
            .try_into()?;

        Ok(Header { nonce })
    }
}

impl TryFrom<Header> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(header: Header) -> Result<Self, Self::Error> {
        let header_ipld = Ipld::from(header);
        DagCborCodec.encode(&header_ipld)
    }
}

impl TryFrom<Vec<u8>> for Header {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let ipld: Ipld = DagCborCodec.decode(&bytes)?;
        ipld.try_into()
    }
}
