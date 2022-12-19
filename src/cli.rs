use clap::Parser;
use libp2p::core::Multiaddr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub peer: Option<Multiaddr>,

    #[arg(long)]
    pub listen: Option<Multiaddr>,

    #[clap(subcommand)]
    pub argument: Argument,
}

#[derive(Debug, Parser)]
pub enum Argument {
    Provide {
        #[arg(short, long)]
        wasm: String,

        #[arg(short, long)]
        fun: String,

        #[arg(short, long, num_args(0..))]
        args: Vec<String>,
    },
    Get {
        #[clap(long)]
        name: String,
    },
}
