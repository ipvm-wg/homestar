#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
// #![deny(unreachable_pub, private_in_public)]

#[macro_use]
extern crate diesel;
extern crate dotenvy;

mod cli;
mod db;
mod network;
mod wasm;
mod workflow;

use crate::{
    db::*,
    schema::{receipts, receipts::dsl::*},
    workflow::{
        closure::Closure,
        receipt::{NewReceipt, Receipt},
    },
};
use async_std::task::spawn;
use clap::Parser;
use cli::{Args, Argument};
use diesel::{prelude::*, SqliteConnection};
use dotenvy::dotenv;
use futures::{prelude::*, Stream, TryStreamExt};
use ipfs_api::{response::AddResponse, IpfsApi, IpfsClient};
use libipld::{
    cbor::DagCborCodec,
    cid::{multibase::Base, Version},
    prelude::Encode,
    Cid, Ipld, Link,
};
use libp2p::{core::PeerId, multiaddr::Protocol};
use multihash::{Code, MultihashDigest};
use std::{
    env,
    error::Error,
    io::{Cursor, Write},
    marker::Unpin,
    str::FromStr,
};
use url::Url;
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

    match opts.listen {
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
        Argument::Provide { wasm, fun, args } => {
            provide(wasm, fun, args, &mut network_client, &mut network_events).await
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
    wasm: String,
    fun: String,
    args: Vec<String>,
    network_client: &mut network::Client,
    network_events: &mut (impl Stream<Item = network::Event> + Unpin),
) {
    let ipfs = IpfsClient::default();

    // Pull Wasm *out* of IPFS
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

    let wasm_args: Vec<Value> = future::try_join_all(args.iter().map(|arg| async {
        // Pull arg *out* of IPFS
        let vec: Vec<u8> = ipfs
            .cat(arg.as_str())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .expect("to convert from utf8");

        i32::from_str(std::str::from_utf8(&vec).expect("to be valid WAsm value"))
            .map(|i| wasmer::Value::I32(i))
    }))
    .await
    .expect("args to resolve from IPFS");

    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes).expect("Wasm module to export");

    let imports = imports! {};
    let instance = Instance::new(&mut store, &module, &imports).expect("Wasm instance to be here");

    let _function = instance
        .exports
        .get::<Function>(&fun.as_str())
        .expect("a Wasm function");
    let _types: &[Type] = _function.ty(&store).params();

    // FIXME write Wasm::Value -> Ipld converter
    let boxed_results: Box<[Value]> = _function
        .call(&mut store, wasm_args.as_slice())
        .expect("tried to call function");

    let res: String = boxed_results
        .into_vec()
        .iter()
        .map(|v| v.to_string())
        .collect();

    let resource = Url::parse(format!("ipfs://{}", wasm).as_str()).expect("IPFS URL");

    let ipld_args: Ipld = Ipld::List(
        wasm_args
            .iter()
            .map(|a| match a {
                Value::I32(i) => Ipld::from(*i),
                _ => todo!(),
            })
            .collect(),
    );

    let closure = workflow::closure::Closure {
        resource,
        action: workflow::closure::Action::from("wasm/run"),
        inputs: workflow::closure::Input::IpldData(ipld_args),
    };

    let closure_ipld: Ipld = closure.clone().into();

    let mut closure_bytes = Vec::new();
    closure_ipld
        .encode(DagCborCodec, &mut closure_bytes)
        .expect("CBOR Serialization");

    let closure_cid2: String = <Closure as Into<Link<Closure>>>::into(closure.clone())
        .to_string_of_base(Base::Base32HexLower)
        .expect("string CID");

    let mut conn = establish_connection();

    let new_receipt = NewReceipt {
        val: res.parse::<i32>().expect("i32"), // FIXME!
        closure_cid: closure_cid2,             // WHYYYYYYY
    };

    diesel::insert_into(receipts::table)
        .values(&new_receipt)
        .execute(&mut conn)
        .expect("Error saving new post");

    todo!("advertise receipt");

    let res_copy = res.clone().into_bytes();

    println!("Wasm CID: {}", closure_cid2);
    println!("Result value: {}", res.to_string());

    let res_cursor = Cursor::new(res);
    let output = match ipfs.add(res_cursor).await.expect("a CID") {
        AddResponse { hash, .. } => hash,
    };

    println!("Result CID: {}", output);

    network_client.start_providing(closure_cid2).await;

    loop {
        match network_events.next().await {
            Some(network::Event::InboundRequest { request, channel }) => {
                if request == closure_cid2 {
                    network_client.respond_file(res_copy.clone(), channel).await;
                }
            }
            e => todo!("{:?}", e),
        }
    }
}
