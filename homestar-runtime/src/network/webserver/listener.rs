//! Listener for incoming requests types.

use faststr::FastStr;
use homestar_invocation::ipld::DagJson;
use homestar_wasm::io::Arg;
use homestar_workflow::Workflow;
use names::{Generator, Name};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue;

/// A [Workflow] run command via a WebSocket channel.
///
/// Note: We leverage the [RawValue] type in order to use our [DagJson]
/// implementation, which is not a direct [Deserialize] implementation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Run<'a> {
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
    use std::assert_eq;

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
        let run = Run {
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
}
