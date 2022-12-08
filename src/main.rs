#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
// #![deny(unreachable_pub, private_in_public)]

//! ipvm
mod cli;
mod ipvm;
mod network;

use async_std::task::spawn;
use clap::Parser;
use cli::{Args, CliArgument};
use futures::{prelude::*, TryStreamExt};
use ipfs_api::{response::AddResponse, IpfsApi, IpfsClient};
use libp2p::{core::PeerId, multiaddr::Protocol};
use std::{
    error::Error,
    io::{Cursor, Write},
    str::FromStr,
};
use wasmer::{imports, Function, Instance, Module, Store, Type, Value};

/// Main entry point.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let ipfs = IpfsClient::default();
    let vals = Args::parse();

    let wasm_bytes: String = match ipfs
        .cat(&vals.wasm.as_str())
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(s) => match String::from_utf8(s) {
            Ok(str) => str,
            Err(err) => panic!("Couldn't convert Wasm from UTF8. Error: {}", err),
        },
        Err(err) => panic!("Couldn't format Wasm. Error: {}", err),
    };

    let _args = match ipfs
        .cat(&vals.args.as_str())
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(yay) => match String::from_utf8(yay) {
            Ok(str) => str,
            Err(err) => panic!("Couldn't convert args from UTF8. Error: {}", err),
        },
        Err(err) => panic!("Couldn't format args. Error: {}", err),
    };

    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes).expect("Module to export");

    let imports = imports! {};
    let instance = Instance::new(&mut store, &module, &imports).expect("Instance to be here");

    let _function = instance
        .exports
        .get::<Function>(&vals.fun.as_str())
        .expect("Should be a Wasm function");
    let _types: &[Type] = _function.ty(&store).params();

    let wasm_args = [Value::I32(
        i32::from_str(_args.as_str()).expect("to be an i32"),
    )]; // [Value::I32(1)]; FIXME I guess it's all ints for now!
    let boxed_results: Box<[Value]> = _function
        .call(&mut store, &wasm_args)
        .expect("tried to call function");

    let res: String = boxed_results
        .into_vec()
        .iter()
        .map(|v| v.to_string())
        .collect();

    let res_copy = res.clone().into_bytes();

    println!("Result value: {}", res.to_string());

    let res_cursor = Cursor::new(res);
    let output = match ipfs.add(res_cursor).await.expect("a CID") {
        AddResponse { hash, .. } => hash,
    };

    println!("Result CID: {}", output);

    let (mut network_client, mut network_events, network_event_loop) = network::new(None).await?;

    // Spawn the network task for it to run in the background.
    spawn(network_event_loop.run());

    // In case a listen address was provided use it, otherwise listen on any
    // address.
    match vals.listen_address {
        Some(addr) => network_client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),
        None => network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    // FIXME shove the stuff to provide in here

    /////////////////////////////////////////

    // Now for a DAG and some light type checking ;)

    //    const Sha3_256: u64 = 0x16;
    //    let digest_bytes = [
    //        0x16, 0x20, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8, 0x9e,
    //        0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c, 0x3b, 0xfb,
    //        0xf2, 0x4e, 0x39, 0x38
    //    ];

    //    let multihash = Multihash::from_bytes(&digest_bytes).unwrap();

    //    Job {
    //        tasks: BTreeMap::from([
    //            (TaskLabel("left"), PureTask(Pure{
    //                wasm: Cid.new_v0(...),
    //                inputs: [
    //                    WasmParam(Value::I32(1)),
    //                    WasmParam(Value::I32(2))
    //                ]
    //            })),
    //            (TaskLabel("right"), PureTask(Pure{
    //                wasm: Cid.new_v0(...),
    //                inputs: [
    //                    Absolute(Cid.new_v0(multihash))
    //                ]
    //            })),
    //            (TaskLabel("end"), PureTask(Pure{
    //                wasm: Cid.new_v0(...),
    //                inputs: [
    //                    Relative(TaskLabel("left")),
    //                    WasmParam(Value::I32(42)),
    //                    Relative(TaskLabel("right"))
    //                ]
    //            }))
    //        ])
    //    }

    // In case the user provided an address of a peer on the CLI, dial it.
    if let Some(addr) = vals.peer {
        let peer_id = match addr.iter().last() {
            Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash).expect("Valid hash."),
            _ => return Err("Expect peer multiaddr to contain peer ID.".into()),
        };
        network_client
            .dial(peer_id, addr)
            .await
            .expect("Dial to succeed");
    }

    match vals.argument {
        // Providing a file.
        CliArgument::Provide { name } => {
            // Advertise oneself as a provider of the file on the DHT.
            network_client.start_providing(name.clone()).await;

            loop {
                match network_events.next().await {
                    // Reply with the content of the file on incoming requests.
                    Some(network::Event::InboundRequest { request, channel }) => {
                        if request == name {
                            network_client.respond_file(res_copy.clone(), channel).await;
                        }
                    }
                    e => todo!("{:?}", e),
                }
            }
        }
        // Locating and getting a file.
        CliArgument::Get { name } => {
            // Locate all nodes providing the file.
            let providers = network_client.get_providers(name.clone()).await;
            if providers.is_empty() {
                return Err(format!("Could not find provider for file {name}.").into());
            }

            // Request the content of the file from each node.
            let requests = providers.into_iter().map(|p| {
                let mut network_client = network_client.clone();
                let name = name.clone();
                async move { network_client.request_file(p, name).await }.boxed()
            });

            // Await the requests, ignore the remaining once a single one succeeds.
            let file_content = futures::future::select_ok(requests)
                .await
                .map_err(|_| "None of the providers returned file.")?
                .0;

            std::io::stdout().write_all(&file_content)?;
        }
    }

    Ok(())
}
