//! File configuration for [Workflow]s.
//!
//! [Workflow]: homestar_workflow::Workflow

use super::Error;
use crate::workflow;
use homestar_invocation::ipld::DagJson;
use homestar_wasm::io::Arg;
use homestar_workflow::Workflow;
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, fmt, path::PathBuf, str::FromStr};
use tokio::fs;

/// Data structure for a workflow file path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadWorkflow {
    /// Workflow file to run.
    file: PathBuf,
}

impl FromStr for ReadWorkflow {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            file: s.parse().map_err(|e| format!("{e}"))?,
        })
    }
}

impl fmt::Display for ReadWorkflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.file)
    }
}

impl ReadWorkflow {
    /// Validate and parse the workflow file.
    ///
    /// Validation is currently limited to checking the file extension,
    /// or attempting to treat the file as JSON if no extension is provided.
    pub(crate) async fn validate_and_parse<'a>(
        &self,
    ) -> Result<(Workflow<'a, Arg>, workflow::Settings), Error> {
        match self.file.extension().and_then(OsStr::to_str) {
            None | Some("json") => {
                let data = fs::read_to_string(&self.file.canonicalize()?).await?;
                // TODO: Parse this from the workflow data/file itself.
                let workflow_settings = workflow::Settings::default();
                Ok((
                    DagJson::from_json_string(data).map_err(anyhow::Error::new)?,
                    workflow_settings,
                ))
            }

            Some(ext) => Err(Error::UnsupportedWorkflow(ext.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use homestar_invocation::{
        authority::UcanPrf,
        task::{instruction::RunInstruction, Resources},
        test_utils, Task,
    };

    #[tokio::test]
    async fn validate_and_parse_workflow() {
        let path = PathBuf::from("./fixtures/test.json");
        let config = Resources::default();
        let (instruction1, instruction2, _) = test_utils::related_wasm_instructions::<Arg>();

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

        workflow.to_file(path.display().to_string()).unwrap();
        let workflow_file = ReadWorkflow { file: path.clone() };

        let (validated_workflow, _settings) = workflow_file.validate_and_parse().await.unwrap();

        assert_eq!(workflow, validated_workflow);

        // rename file extension
        fs::rename(path, "./fixtures/test.txt").await.unwrap();
        let new_path = PathBuf::from("./fixtures/test.txt");
        let workflow_file = ReadWorkflow {
            file: new_path.clone(),
        };
        let error = workflow_file.validate_and_parse().await;
        assert_eq!(
            error.unwrap_err().to_string(),
            "unsupported workflow file type: txt"
        );

        // rename to no file extension
        fs::rename(new_path, "./fixtures/test_fam").await.unwrap();
        let new_path = PathBuf::from("./fixtures/test_fam");
        let workflow_file = ReadWorkflow {
            file: new_path.clone(),
        };
        let (newly_validated_workflow, _settings) =
            workflow_file.validate_and_parse().await.unwrap();
        assert_eq!(workflow, newly_validated_workflow);
    }
}
