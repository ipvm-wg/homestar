//! CLI commands/arguments.

use clap::Parser;
use libp2p::core::Multiaddr;

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Peer address.
    #[arg(long)]
    pub peer: Option<Multiaddr>,

    /// Listen address.
    #[arg(long)]
    pub listen: Option<Multiaddr>,

    /// Ipvm-specific [Argument].
    #[clap(subcommand)]
    pub argument: Argument,
}

/// An Ipvm-specific CLI argument.
#[derive(Debug, Parser)]
pub enum Argument {
    /// Provider arguments.
    Provide {
        /// Wasm or WAT [Cid].
        ///
        /// [Cid]: libipld::cid::Cid
        #[arg(short, long)]
        wasm: String,

        /// Function name within Wasm module.
        #[arg(short, long)]
        fun: String,

        /// Parameters / arguments to Wasm function.
        #[arg(short, long, num_args(0..))]
        args: Vec<String>,
    },
    /// GET/read arguments.
    Get {
        #[clap(long)]
        /// [Cid] name/pointer to content.
        ///
        /// [Cid]: libipld::cid::Cid
        name: String,
    },
}
