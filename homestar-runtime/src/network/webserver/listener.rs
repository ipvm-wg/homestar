//! Listener for incoming requests types.

use anyhow::anyhow;
use faststr::FastStr;
use homestar_invocation::ipld::{DagCbor, DagJson};
use homestar_wasm::io::Arg;
use homestar_workflow::Workflow;
use libipld::{serde::from_ipld, Ipld};
use names::{Generator, Name};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue;
use std::collections::BTreeMap;

const NAME_KEY: &str = "name";
const WORKFLOW_KEY: &str = "workflow";

/// A [Workflow] run command via a WebSocket channel for JSON inputs.
///
/// Note: We leverage the [RawValue] type in order to use our DagJson
/// implementation, which is not a direct [Deserialize] implementation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRun<'a> {
    #[serde(default = "default_name")]
    pub(crate) name: FastStr,
    #[serde(deserialize_with = "from_raw_value")]
    pub(crate) workflow: Workflow<'a, Arg>,
}

fn default_name() -> FastStr {
    let mut name_gen = Generator::with_naming(Name::Numbered);
    name_gen
        .next()
        .unwrap_or_else(|| "workflow".to_string())
        .into()
}

fn from_raw_value<'a, 'de, D>(deserializer: D) -> Result<Workflow<'a, Arg>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_value: &RawValue = Deserialize::deserialize(deserializer)?;
    Workflow::from_json(raw_value.get().as_bytes()).map_err(de::Error::custom)
}

/// A [Workflow] run command via a WebSocket channel for CBOR inputs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CborRun<'a> {
    pub(crate) name: FastStr,
    pub(crate) workflow: Workflow<'a, Arg>,
}

impl<'a> From<CborRun<'a>> for Ipld {
    fn from(run: CborRun<'a>) -> Self {
        Ipld::Map(BTreeMap::from([
            ("name".into(), Ipld::String(run.name.as_str().to_string())),
            ("workflow".into(), run.workflow.into()),
        ]))
    }
}

impl<'a> TryFrom<Ipld> for CborRun<'a> {
    type Error = anyhow::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        let map = from_ipld::<BTreeMap<String, Ipld>>(ipld)?;
        let name: String = from_ipld(
            map.get(NAME_KEY)
                .ok_or_else(|| anyhow!("missing {NAME_KEY}"))?
                .to_owned(),
        )?;
        let workflow = Workflow::try_from(
            map.get(WORKFLOW_KEY)
                .ok_or_else(|| anyhow!("missing {WORKFLOW_KEY}"))?
                .to_owned(),
        )?;
        Ok(CborRun {
            name: FastStr::from(name),
            workflow,
        })
    }
}

impl DagCbor for CborRun<'_> {}
impl DagJson for CborRun<'_> {}

impl<'a, 'de> Deserialize<'de> for CborRun<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Vec::<u8>::deserialize(deserializer)?;
        let ipld: Ipld = serde_ipld_dagcbor::from_slice(&value).map_err(de::Error::custom)?;
        let run = CborRun::try_from(ipld).map_err(de::Error::custom)?;
        Ok(run)
    }
}

/// Filter metrics by prefix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MetricsPrefix {
    pub(crate) prefix: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use homestar_invocation::{
        authority::UcanPrf,
        task::{instruction::RunInstruction, Resources},
        test_utils, Task,
    };
    use std::{fs, path::PathBuf};

    #[test]
    fn run_json() {
        let config = Resources::default();
        let instruction1 = test_utils::instruction::<Arg>();
        let (instruction2, _) = test_utils::wasm_instruction_with_nonce::<Arg>();

        let task1 = Task::new(
            RunInstruction::Expanded(instruction1),
            config.clone().into(),
            UcanPrf::default(),
        );
        let task2 = Task::new(
            RunInstruction::Expanded(instruction2),
            config.into(),
            UcanPrf::default(),
        );

        let workflow = Workflow::new(vec![task1.clone(), task2.clone()]);
        let run = JsonRun {
            name: "test".into(),
            workflow: workflow.clone(),
        };

        let run_str = format!(
            r#"{{"name": "test","workflow": {}}}"#,
            workflow.to_json_string().unwrap()
        );

        let post_run = serde_json::from_str(&run_str).unwrap();
        assert_eq!(run, post_run);
    }

    #[test]
    fn write_cbor_to_file_and_read() {
        let workflow_str =
            fs::read_to_string("tests/fixtures/test-workflow-image-pipeline.json").unwrap();
        let json: serde_json::Value = serde_json::from_str(&workflow_str).unwrap();
        let json_string = serde_json::to_string(&json).unwrap();
        let run_str = format!(r#"{{"name": "test","workflow": {}}}"#, json_string);
        let run1: CborRun<'_> = DagJson::from_json_string(run_str).unwrap();

        let path = PathBuf::from("./fixtures/test.cbor");
        assert!(run1
            .clone()
            .to_cbor_file(path.display().to_string())
            .is_ok());

        let cbor_file = fs::read(path).unwrap();
        let run2: CborRun<'_> = DagCbor::from_cbor(&cbor_file).unwrap();
        assert_eq!(run1, run2);
    }
}
