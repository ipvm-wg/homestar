use clap::Parser;
use libp2p::core::Multiaddr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub wasm: String,

    #[arg(short, long)]
    pub fun: String,

    #[arg(short, long)]
    pub args: String,

    #[arg(long)]
    pub peer: Option<Multiaddr>,

    #[arg(short, long)]
    pub listen_address: Option<Multiaddr>,

    #[clap(subcommand)]
    pub argument: CliArgument,
}

#[derive(Debug, Parser)]
pub enum CliArgument {
    Provide {
        #[clap(long)]
        name: String,
    },
    Get {
        #[clap(long)]
        name: String,
    },
}

// use cid::multihash::{Code, MultihashDigest};
// use cid::Cid;
// use multihash;
// use std::collections::BTreeMap;

// #[derive(Debug, Clone, Sized)]
// struct TaskLabel(String);
//
// #[derive(Debug, Clone)]
// enum Input {
//     WasmParam(Value),
//     Absolute(Cid),
//     Relative(TaskLabel)
// }
//
// #[derive(Debug, Sized)]
// struct Pure {
//     wasm: Cid,
//     inputs: [Input]
// }
//
// #[derive(Debug)]
// struct Effect {
//     resource: String, // Uri,
//     action: String
// }
//
// #[derive(Debug)]
// enum Task {
//     PureTask(Pure),
//     EffectTask(Effect)
// }
//
// #[derive(Debug)]
// struct Job {
//     // version: semver::Version,
//     // nonce: String,
//     tasks: BTreeMap<TaskLabel, Task>
//     // signature:
// }
