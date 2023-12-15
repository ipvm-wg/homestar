//! [super::SwarmEvent] [PeerRecord] traits and decoding implementation.

use crate::{
    event_handler::RequestResponseError,
    receipt::{RECEIPT_TAG, VERSION_KEY},
    workflow,
    workflow::WORKFLOW_TAG,
    Receipt,
};
use anyhow::{anyhow, Result};
use homestar_core::{
    consts,
    workflow::{Pointer, Receipt as InvocationReceipt},
};
use libipld::{Cid, Ipld};
use libp2p::{kad::PeerRecord, PeerId};

/// Trait for handling [PeerRecord]s found on the DHT.
pub(crate) trait FoundRecord {
    fn found_record(&self) -> Result<DecodedRecord>;
}

impl FoundRecord for PeerRecord {
    fn found_record(&self) -> Result<DecodedRecord> {
        let key_cid = Cid::try_from(self.record.key.as_ref())?;

        println!("<< KEY_CID {key_cid} >>");

        decode_capsule(key_cid, self.record.publisher, &self.record.value)
    }
}

/// Internal records decoded from [PeerRecord] found on DHT.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DecodedRecord {
    Receipt(ReceiptRecord),
    Workflow(WorkflowInfoRecord),
}

/// [DecodedRecord] variant for receipts found on DHT.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReceiptRecord {
    pub(crate) peer_id: Option<PeerId>,
    pub(crate) receipt: Receipt,
}

/// [DecodedRecord] variant for workflow info found on DHT.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WorkflowInfoRecord {
    pub(crate) peer_id: Option<PeerId>,
    pub(crate) workflow_info: workflow::Info,
}

pub(crate) fn decode_capsule(
    key_cid: Cid,
    peer_id: Option<PeerId>,
    value: &Vec<u8>,
) -> Result<DecodedRecord> {
    // If it decodes to an error, return the error.
    if let Ok((decoded_error, _)) = RequestResponseError::decode(value) {
        return Err(anyhow!("value returns an error: {decoded_error}"));
    };

    match serde_ipld_dagcbor::de::from_reader(&**value) {
        Ok(Ipld::Map(mut map)) => match map.pop_first() {
            Some((code, Ipld::Map(mut rest))) if code == RECEIPT_TAG => {
                if rest.remove(VERSION_KEY)
                    == Some(Ipld::String(consts::INVOCATION_VERSION.to_string()))
                {
                    let invocation_receipt = InvocationReceipt::try_from(Ipld::Map(rest))?;
                    let receipt = Receipt::try_with(Pointer::new(key_cid), &invocation_receipt)?;

                    println!("<< DESERIALIZED A RECEIPT >>");

                    Ok(DecodedRecord::Receipt(ReceiptRecord { peer_id, receipt }))
                } else {
                    Err(anyhow!(
                        "record version mismatch, current version: {}",
                        consts::INVOCATION_VERSION
                    ))
                }
            }
            Some((code, Ipld::Map(rest))) if code == WORKFLOW_TAG => {
                let workflow_info = workflow::Info::try_from(Ipld::Map(rest))?;

                println!("<< DESERIALIZED A WORKFLOW INFO >>");

                Ok(DecodedRecord::Workflow(WorkflowInfoRecord {
                    peer_id,
                    workflow_info,
                }))
            }
            Some((code, _)) => Err(anyhow!("decode mismatch: {code} is not known")),
            None => Err(anyhow!("invalid record value")),
        },
        Ok(ipld) => Err(anyhow!(
            "decode mismatch: expected an Ipld map, got {ipld:#?}",
        )),
        Err(err) => Err(anyhow!("error deserializing record value: {err}")),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils, workflow};
    use homestar_core::{
        ipld::DagCbor,
        test_utils::workflow as workflow_test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
        Workflow,
    };
    use homestar_wasm::io::Arg;
    use libp2p::{kad::Record, PeerId};

    #[test]
    fn found_receipt_record() {
        let (invocation_receipt, receipt) = test_utils::receipt::receipts();
        let instruction_bytes = receipt.instruction_cid_as_bytes();
        let bytes = Receipt::invocation_capsule(&invocation_receipt).unwrap();
        let record = Record::new(instruction_bytes, bytes);
        let peer_record = PeerRecord {
            record,
            peer: Some(PeerId::random()),
        };
        if let DecodedRecord::Receipt(decoded) = peer_record.found_record().unwrap() {
            assert_eq!(decoded.receipt, receipt);
        } else {
            panic!("Incorrect record type")
        }
    }

    #[test]
    fn found_workflow_record() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            workflow_test_utils::related_wasm_instructions::<Arg>();
        let task1 = Task::new(
            RunInstruction::Expanded(instruction1.clone()),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.into(),
            UcanPrf::default(),
        );

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let stored_info = workflow::Stored::default(
            Pointer::new(workflow.clone().to_cid().unwrap()),
            workflow.len() as i32,
        );
        let workflow_info = workflow::Info::default(stored_info);
        let workflow_cid_bytes = workflow_info.cid_as_bytes();
        let bytes = workflow_info.capsule().unwrap();
        let record = Record::new(workflow_cid_bytes, bytes);
        let peer_record = PeerRecord {
            record,
            peer: Some(PeerId::random()),
        };
        if let DecodedRecord::Workflow(decoded) = peer_record.found_record().unwrap() {
            assert_eq!(decoded.workflow_info, workflow_info);
        } else {
            panic!("Incorrect record type")
        }
    }
}
