//! Responses for display/return to the user for
//! client requests.

use crate::{
    cli::show::{self, ApplyStyle},
    workflow::{self, IndexedResources},
};
use chrono::NaiveDateTime;
use faststr::FastStr;
use libipld::Cid;
use serde::{Deserialize, Serialize};
use std::{fmt, net::SocketAddr, sync::Arc};
use tabled::{
    col,
    settings::{object::Rows, Format, Modify},
    Table, Tabled,
};

/// Workflow information specified for response / display upon
/// acknowledgement of running a workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tabled)]
pub struct AckWorkflow {
    pub(crate) cid: Cid,
    pub(crate) name: FastStr,
    pub(crate) num_tasks: u32,
    #[tabled(skip)]
    pub(crate) progress: Vec<Cid>,
    pub(crate) progress_count: u32,
    #[tabled(skip)]
    pub(crate) resources: IndexedResources,
    pub(crate) timestamp: String,
}

impl fmt::Display for AckWorkflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cid: {}, progress: {}/{}, timestamp: {}",
            self.cid, self.progress_count, self.num_tasks, self.timestamp
        )
    }
}

impl AckWorkflow {
    /// Workflow information for response / display.
    pub(crate) fn new(
        workflow_info: Arc<workflow::Info>,
        name: FastStr,
        timestamp: NaiveDateTime,
    ) -> Self {
        Self {
            cid: workflow_info.cid,
            name,
            num_tasks: workflow_info.num_tasks,
            progress: workflow_info.progress.clone(),
            progress_count: workflow_info.progress_count,
            resources: workflow_info.resources.clone(),
            timestamp: timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

impl show::ConsoleTable for AckWorkflow {
    fn table(&self) -> show::Output {
        show::Output::new(Table::new(vec![self]).to_string())
    }

    fn echo_table(&self) -> Result<(), std::io::Error> {
        let table = self.table();

        let mut resource_table = Table::new(
            self.resources
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
        );

        resource_table
            .with(Modify::new(Rows::first()).with(Format::content(|_s| "Resources".to_string())));

        let tbl = col![table, resource_table].default();

        tbl.echo()
    }
}

/// Ping response for display.
#[derive(Tabled)]
pub(crate) struct Ping {
    address: SocketAddr,
    response: String,
}

impl Ping {
    /// Create a new [Ping] response.
    pub(crate) fn new(address: SocketAddr, response: String) -> Self {
        Self { address, response }
    }
}

impl show::ConsoleTable for Ping {
    fn table(&self) -> show::Output {
        Table::new(vec![&self]).default()
    }

    fn echo_table(&self) -> Result<(), std::io::Error> {
        self.table().echo()
    }
}
