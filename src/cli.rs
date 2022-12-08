use clap::Parser;
use libp2p::core::Multiaddr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub peer: Option<Multiaddr>,

    #[arg(short, long)]
    pub listen_address: Option<Multiaddr>,

    #[clap(subcommand)]
    pub argument: Argument,
}

#[derive(Debug, Parser)]
pub enum Argument {
    Provide {
        #[clap(long)]
        name: String,

        #[arg(short, long)]
        wasm: String,

        #[arg(short, long)]
        fun: String,

        #[arg(short, long)]
        args: String,
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
