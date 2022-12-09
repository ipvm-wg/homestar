#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
// #![deny(unreachable_pub, private_in_public)]

//! ipvm
mod cli;
mod ipvm;
mod network;

use async_std::task::spawn;
use clap::Parser;
use cli::{Args, Argument};
use futures::{prelude::*, Stream, TryStreamExt};
use ipfs_api::{response::AddResponse, IpfsApi, IpfsClient};
use libp2p::{core::PeerId, multiaddr::Protocol};
use std::{
    error::Error,
    io::{Cursor, Write},
    marker::Unpin,
    str::FromStr,
};
use wasmer::{imports, Function, Instance, Module, Store, Type, Value};

/// Main entry point.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (mut network_client, mut network_events, network_event_loop) = network::new(None).await?;

    spawn(network_event_loop.run());

    let opts = Args::parse();

    if let Some(addr) = opts.peer {
        let peer_id = match addr.iter().last() {
            Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash).expect("Valid hash."),
            _ => return Err("Expect peer multiaddr to contain peer ID.".into()),
        };
        network_client
            .dial(peer_id, addr)
            .await
            .expect("Dial to succeed");
    }

    match opts.listen_address {
        Some(addr) => network_client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),

        None => network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    match opts.argument {
        Argument::Get { name } => request(name, &mut network_client).await,
        Argument::Provide {
            name,
            wasm,
            fun,
            args,
        } => {
            provide(
                name,
                wasm,
                fun,
                args,
                &mut network_client,
                &mut network_events,
            )
            .await
        }
    }

    Ok(())
}

async fn request(name: String, network_client: &mut network::Client) -> () {
    let providers = network_client.get_providers(name.clone()).await;
    if providers.is_empty() {
        panic!("Could not find provider for file {}.", name);
    }

    let requests = providers.into_iter().map(|p| {
        let mut network_client = network_client.clone();
        let name = name.clone();
        async move { network_client.request_file(p, name).await }.boxed()
    });

    let file_map = futures::future::select_ok(requests)
        .await
        .map_err(|_| "None of the providers returned file.");

    let file_content = match file_map {
        Ok(x) => x.0,
        Err(y) => panic!("no file {}", y),
    };

    let _ = std::io::stdout().write_all(&file_content);
    return;
}

async fn provide(
    name: String,
    wasm: String,
    fun: String,
    args: String,
    network_client: &mut network::Client,
    network_events: &mut (impl Stream<Item = network::Event> + Unpin),
) {
    let ipfs = IpfsClient::default();
    let wasm_bytes: String = match ipfs
        .cat(&wasm.as_str())
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
        .cat(&args.as_str())
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
        .get::<Function>(&fun.as_str())
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

    network_client.start_providing(name.clone()).await;

    loop {
        match network_events.next().await {
            Some(network::Event::InboundRequest { request, channel }) => {
                if request == name {
                    network_client.respond_file(res_copy.clone(), channel).await;
                }
            }
            e => todo!("{:?}", e),
        }
    }
}
