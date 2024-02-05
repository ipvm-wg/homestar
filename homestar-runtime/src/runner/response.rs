//! Responses for display/return to the user for
//! client requests.

use crate::{
    cli::show::{self, ApplyStyle},
    runner::WorkflowReceiptInfo,
    workflow::{self, IndexedResources},
};
use chrono::NaiveDateTime;
use faststr::FastStr;
use libipld::Cid;
use serde::{Deserialize, Serialize};
use std::{fmt, net::SocketAddr, sync::Arc};
use tabled::{
    builder::Builder,
    col,
    settings::{object::Rows, Format, Modify},
    Table, Tabled,
};

use super::{DynamicNodeInfo, StaticNodeInfo};

/// Workflow information specified for response / display upon
/// acknowledgement of running a workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tabled)]
pub struct AckWorkflow {
    pub(crate) cid: Cid,
    pub(crate) name: FastStr,
    pub(crate) num_tasks: u32,
    pub(crate) progress_count: u32,
    #[tabled(skip)]
    pub(crate) resources: IndexedResources,
    #[tabled(skip)]
    pub(crate) replayed_receipt_info: Vec<WorkflowReceiptInfo>,
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
        replayed_receipt_info: Vec<WorkflowReceiptInfo>,
        name: FastStr,
        timestamp: NaiveDateTime,
    ) -> Self {
        Self {
            cid: workflow_info.cid,
            name,
            num_tasks: workflow_info.num_tasks,
            progress_count: workflow_info.progress_count,
            resources: workflow_info.resources.clone(),
            replayed_receipt_info,
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

        let mut receipt_table_builder = Builder::default();
        receipt_table_builder.push_record([
            "Replayed Receipt".to_string(),
            "Invocation Ran".to_string(),
            "Instruction".to_string(),
        ]);

        for (cid, info) in &self.replayed_receipt_info {
            if let Some((ran, instruction)) = info {
                receipt_table_builder.push_record([
                    cid.to_string(),
                    ran.to_string(),
                    instruction.to_string(),
                ]);
            }
        }

        // If there are no replayed receipts, add a placeholder row.
        if receipt_table_builder.count_rows() == 1 {
            receipt_table_builder.push_record([
                "<none>".to_string(),
                "".to_string(),
                "".to_string(),
            ]);
        };

        let receipt_table = receipt_table_builder.build();

        let tbl = col![table, resource_table, receipt_table].default_with_title("run");

        tbl.echo()
    }
}

/// Ping response for display.
#[derive(Debug, Tabled)]
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
        Table::new(vec![&self]).default_with_title("ping/pong")
    }

    fn echo_table(&self) -> Result<(), std::io::Error> {
        self.table().echo()
    }
}

/// Node identity response for display.
#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct AckNodeInfo {
    /// Static node information.
    static_info: StaticNodeInfo,
    /// Dynamic node information.
    dyn_info: DynamicNodeInfo,
}

impl fmt::Display for AckNodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl AckNodeInfo {
    /// Create a new [AckNodeInfo] response.
    pub(crate) fn new(static_info: StaticNodeInfo, dyn_info: DynamicNodeInfo) -> Self {
        Self {
            static_info,
            dyn_info,
        }
    }
}

impl show::ConsoleTable for AckNodeInfo {
    fn table(&self) -> show::Output {
        show::Output::new(Table::new(vec![self]).to_string())
    }

    fn echo_table(&self) -> Result<(), std::io::Error> {
        let static_info_table = Table::new(vec![&self.static_info]);

        let mut listeners_table = Table::new(
            self.dyn_info
                .listeners
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
        );

        let conns = self
            .dyn_info
            .connections
            .iter()
            .map(|(k, v)| vec![k.to_string(), v.to_string()])
            .collect::<Vec<Vec<String>>>();

        let mut conns_table_builder = tabled::builder::Builder::from_iter(conns);

        // If there are no connections, add a placeholder row.
        if conns_table_builder.count_rows() == 0 {
            conns_table_builder.push_record([
                "Connections".to_string(),
                "".to_string(),
                "".to_string(),
            ]);
            conns_table_builder.push_record(["<none>".to_string(), "".to_string(), "".to_string()]);
        } else {
            conns_table_builder.insert_record(
                0,
                ["Connections".to_string(), "".to_string(), "".to_string()],
            );
        }

        listeners_table.with(
            Modify::new(Rows::first()).with(Format::content(|_s| "Listen Addresses".to_string())),
        );
        let conns_table = conns_table_builder.build();

        let tbl = col![static_info_table, listeners_table, conns_table].default_with_title("node");

        tbl.echo()
    }
}

/// Info response for display.
#[derive(Debug, Tabled)]
pub struct Info {
    version: String,
    git_sha: String,
    features: String,
}

impl Default for Info {
    fn default() -> Self {
        Self::new()
    }
}

impl Info {
    /// Create a new [Info] response.
    pub(crate) fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            git_sha: env!("VERGEN_GIT_SHA").to_string(),
            features: env!("VERGEN_CARGO_FEATURES").to_string(),
        }
    }
}

impl show::ConsoleTable for Info {
    fn table(&self) -> show::Output {
        Table::new(vec![&self]).default_with_title("info")
    }

    fn echo_table(&self) -> Result<(), std::io::Error> {
        self.table().echo()
    }
}
