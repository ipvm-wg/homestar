//! CLI commands/arguments.

use clap::Parser;

const HELP_TEMPLATE: &str = "{about} {version}

USAGE:
    {usage}

{all-args}
";

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, help_template = HELP_TEMPLATE)]
pub struct Args {
    /// Ipvm-specific [Argument].
    #[clap(subcommand)]
    pub argument: Argument,
}

/// CLI Argument types.
#[derive(Debug, Parser)]
pub enum Argument {
    /// Run a workflow given a file.
    Run {
        /// Configuration file for *homestar* node settings.
        #[arg(
            short = 'c',
            long = "config",
            value_name = "CONFIG",
            help = "runtime configuration file"
        )]
        runtime_config: Option<String>,
    },
}
