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
pub struct Cli {
    /// Optional runtime configuration file, otherwise use defaults.
    #[arg(
        short = 'c',
        long = "config",
        value_name = "CONFIG",
        help = "runtime configuration file"
    )]
    pub runtime_config: Option<String>,

    /// Homestar [Command].
    #[clap(subcommand)]
    pub command: Command,
}

/// CLI Argument types.
#[derive(Debug, Parser)]
pub enum Command {
    /// Start the Runtime with the Homestar runner.
    Start,
}
