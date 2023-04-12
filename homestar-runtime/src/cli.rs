//! CLI commands/arguments.

use clap::Parser;

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Ipvm-specific [Argument].
    #[clap(subcommand)]
    pub argument: Argument,
}

/// CLI Argument types.
#[derive(Debug, Parser)]
pub enum Argument {
    /// TODO: Run [Workflow] given a file.
    ///
    /// [Workflow]: crate::Workflow
    Run {
        /// Configuration file for *homestar* node settings.
        #[arg(short, long)]
        runtime_config: Option<String>,
    },
}
