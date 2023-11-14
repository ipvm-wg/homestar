use anyhow::anyhow;
use anyhow::Result;
use homestar_core::workflow::Nonce;
use libipld::{self, cbor::DagCborCodec, prelude::Codec, serde::from_ipld, Ipld};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct Message<T> {
    header: Header,
    payload: T,
}

const HEADER_KEY: &str = "header";
const PAYLOAD_KEY: &str = "payload";
const NONCE_KEY: &str = "nonce";

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
    Ipld: From<Message<T>>,
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

impl From<Message<Ipld>> for Ipld {
    fn from(message: Message<Ipld>) -> Self {
        From::from(&message)
    }
}

impl From<&Message<Ipld>> for Ipld {
    fn from(message: &Message<Ipld>) -> Self {
        Ipld::Map(BTreeMap::from([
            (HEADER_KEY.into(), message.header.to_owned().into()),
            (PAYLOAD_KEY.into(), message.payload.to_owned().into()),
        ]))
    }
}

impl<'a, T> TryFrom<Ipld> for Message<T>
where
    T: From<&'a Ipld>,
    Ipld: From<T>,
{
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let header = map
            .get(HEADER_KEY)
            .ok_or_else(|| anyhow!("missing {HEADER_KEY}"))?
            .try_into()?;

        let payload = map
            .get(PAYLOAD_KEY)
            .ok_or_else(|| anyhow!("missing {PAYLOAD_KEY}"))?
            .try_into()?;

        Ok(Message { header, payload })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Header {
    nonce: Nonce,
}

impl From<&Header> for Ipld {
    fn from(header: &Header) -> Self {
        Ipld::Map(BTreeMap::from([(
            NONCE_KEY.into(),
            header.nonce.to_owned().into(),
        )]))
    }
}

impl From<Header> for Ipld {
    fn from(header: Header) -> Self {
        From::from(&header)
    }
}

impl TryFrom<&Ipld> for Header {
    type Error = anyhow::Error;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
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
