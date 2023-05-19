//! Output of an invocation, referenced by its invocation pointer.

use super::{prf::UcanPrf, InstructionResult, Issuer, Pointer};
use anyhow::anyhow;
use libipld::{
    cbor::DagCborCodec,
    cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    },
    prelude::Codec,
    serde::from_ipld,
    Ipld,
};
use std::collections::BTreeMap;

const RAN_KEY: &str = "ran";
const OUT_KEY: &str = "out";
const ISSUER_KEY: &str = "iss";
const METADATA_KEY: &str = "meta";
const PROOF_KEY: &str = "prf";

/// A Receipt is a cryptographically signed description of the [Invocation]
/// and its [resulting output] and requested effects.
///
/// TODO: Effects et al.
///
/// [resulting output]: InstructionResult
/// [Invocation]: super::Invocation
#[derive(Debug, Clone, PartialEq)]
pub struct Receipt<T> {
    ran: Pointer,
    out: InstructionResult<T>,
    meta: Ipld,
    issuer: Option<Issuer>,
    prf: UcanPrf,
}

impl<T> Receipt<T> {
    ///
    pub fn new(
        ran: Pointer,
        result: InstructionResult<T>,
        metadata: Ipld,
        issuer: Option<Issuer>,
        proof: UcanPrf,
    ) -> Self {
        Self {
            ran,
            out: result,
            meta: metadata,
            issuer,
            prf: proof,
        }
    }
}

impl<T> Receipt<T> {
    /// [Pointer] for [Invocation] ran.
    ///
    /// [Invocation]: super::Invocation
    pub fn ran(&self) -> &Pointer {
        &self.ran
    }

    /// [InstructionResult] output from invocation/execution.
    pub fn out(&self) -> &InstructionResult<T> {
        &self.out
    }

    /// [Ipld] metadata.
    pub fn meta(&self) -> &Ipld {
        &self.meta
    }

    /// Optional [Issuer] for [Receipt].
    pub fn issuer(&self) -> &Option<Issuer> {
        &self.issuer
    }

    /// [UcanPrf] delegation chain.
    pub fn prf(&self) -> &UcanPrf {
        &self.prf
    }
}

impl TryFrom<Receipt<Ipld>> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt<Ipld>) -> Result<Self, Self::Error> {
        let receipt_ipld = Ipld::from(&receipt);
        DagCborCodec.encode(&receipt_ipld)
    }
}

impl TryFrom<Vec<u8>> for Receipt<Ipld> {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let ipld: Ipld = DagCborCodec.decode(&bytes)?;
        ipld.try_into()
    }
}

impl TryFrom<Receipt<Ipld>> for Cid {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt<Ipld>) -> Result<Self, Self::Error> {
        TryFrom::try_from(&receipt)
    }
}

impl TryFrom<&Receipt<Ipld>> for Cid {
    type Error = anyhow::Error;

    fn try_from(receipt: &Receipt<Ipld>) -> Result<Self, Self::Error> {
        let ipld = Ipld::from(receipt);
        let bytes = DagCborCodec.encode(&ipld)?;
        let hash = Code::Sha3_256.digest(&bytes);
        Ok(Cid::new_v1(0x71, hash))
    }
}

impl From<&Receipt<Ipld>> for Ipld {
    fn from(receipt: &Receipt<Ipld>) -> Self {
        Ipld::Map(BTreeMap::from([
            (RAN_KEY.into(), receipt.ran.to_owned().into()),
            (OUT_KEY.into(), receipt.out.to_owned().into()),
            (METADATA_KEY.into(), receipt.meta.to_owned()),
            (
                ISSUER_KEY.into(),
                receipt
                    .issuer
                    .as_ref()
                    .map(|issuer| issuer.to_string().into())
                    .unwrap_or(Ipld::Null),
            ),
            (PROOF_KEY.into(), receipt.prf.to_owned().into()),
        ]))
    }
}

impl From<Receipt<Ipld>> for Ipld {
    fn from(receipt: Receipt<Ipld>) -> Self {
        From::from(&receipt)
    }
}

impl TryFrom<Ipld> for Receipt<Ipld> {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;

        let ran = map
            .get(RAN_KEY)
            .ok_or_else(|| anyhow!("missing {RAN_KEY}"))?
            .try_into()?;

        let out = map
            .get(OUT_KEY)
            .ok_or_else(|| anyhow!("missing {OUT_KEY}"))?;

        let meta = map
            .get(METADATA_KEY)
            .ok_or_else(|| anyhow!("missing {METADATA_KEY}"))?;

        let issuer = map
            .get(ISSUER_KEY)
            .and_then(|ipld| match ipld {
                Ipld::Null => None,
                ipld => Some(ipld),
            })
            .and_then(|ipld| from_ipld(ipld.to_owned()).ok())
            .map(Issuer::new);

        let prf = map
            .get(PROOF_KEY)
            .ok_or_else(|| anyhow!("missing {PROOF_KEY}"))?;

        Ok(Receipt {
            ran,
            out: InstructionResult::try_from(out)?,
            meta: meta.to_owned(),
            issuer,
            prf: UcanPrf::try_from(prf)?,
        })
    }
}

impl TryFrom<Receipt<Ipld>> for Pointer {
    type Error = anyhow::Error;

    fn try_from(receipt: Receipt<Ipld>) -> Result<Self, Self::Error> {
        Ok(Pointer::new(Cid::try_from(receipt)?))
    }
}
