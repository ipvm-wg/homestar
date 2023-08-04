//! Traits related to [Ipld] and [DagJson] encoding/decoding.
//!
//! [DagJson]: DagJsonCodec

use crate::{workflow::Error, Unit};
use libipld::{codec::Decode, json::DagJsonCodec, prelude::Codec, Ipld};
use std::{
    fs,
    io::{Cursor, Write},
};

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
            Error::<Unit>::UnexpectedIpldType(
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

    /// Write JSON to file.
    fn to_file(&self, filename: String) -> Result<(), Error<Unit>> {
        Ok(fs::File::create(filename)?.write_all(&self.to_json()?)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        test_utils,
        workflow::{config::Resources, instruction::RunInstruction, prf::UcanPrf, Task},
        Workflow,
    };

    #[test]
    fn write_json_to_file_and_read() {
        let config = Resources::default();
        let (instruction1, instruction2, _) =
            test_utils::workflow::related_wasm_instructions::<Unit>();

        let task1 = Task::new(
            RunInstruction::Expanded(instruction1.clone()),
            config.clone().into(),
            UcanPrf::default(),
        );

        let task2 = Task::new(
            RunInstruction::Expanded(instruction2.clone()),
            config.into(),
            UcanPrf::default(),
        );

        let workflow = Workflow::new(vec![task1, task2]);
        let json = workflow.to_json_string().unwrap();
        workflow
            .to_file("./fixtures/test_add.json".to_string())
            .unwrap();
        let read_file = fs::read_to_string("./fixtures/test_add.json").unwrap();
        assert_eq!(json, read_file);

        let workflow_read = Workflow::from_json_string(read_file).unwrap();
        assert_eq!(workflow, workflow_read);
    }
}
