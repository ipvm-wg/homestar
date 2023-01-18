use anyhow::{anyhow, bail, Result};
use clap::Parser;
use diesel::RunQueryDsl;
use ipfs_api::{response::AddResponse, IpfsApi, IpfsClient};
use ipvm::{
    cli::{Args, Argument},
    db::{self, schema},
    network::{
        client::Client,
        eventloop::{Event, RECEIPTS_TOPIC},
        swarm::{self, TopicMessage},
    },
    wasm::operator,
    workflow::{
        closure::{Action, Closure, Input},
        receipt::Receipt,
    },
};
use libipld::{cbor::DagCborCodec, cid::multibase::Base, prelude::Encode, Ipld, Link};
use libp2p::{
    core::PeerId,
    futures::{future, FutureExt, TryStreamExt},
    identity::Keypair,
    multiaddr::Protocol,
};
use std::{
    io::{self, Cursor, Write},
    str::{self, FromStr},
    sync::Arc,
};
use url::Url;
use uuid::Uuid;
use wasmer::{imports, CompilerConfig, EngineBuilder, Function, Instance, Module, Store, Value};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_middlewares::Metering;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opts = Args::parse();
    let keypair = Keypair::generate_ed25519();

    let mut swarm = swarm::new(keypair).await?;

    // subscribe to `receipts` topic
    swarm.behaviour_mut().gossip_subscribe(RECEIPTS_TOPIC)?;

    let (mut client, mut events, event_loop) = Client::new(swarm).await?;

    tokio::spawn(event_loop.run());

    if let Some(addr) = opts.peer {
        let peer_id = match addr.iter().last() {
            Some(Protocol::P2p(hash)) => PeerId::from_multihash(hash).expect("Valid hash."),
            _ => bail!("Expect peer multiaddr to contain peer ID."),
        };
        client.dial(peer_id, addr).await.expect("Dial to succeed.");
    }

    match opts.listen {
        Some(addr) => client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),

        None => client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    // TODO: abstraction for this.
    match opts.argument {
        Argument::Get { name } => {
            let providers = client.get_providers(name.clone()).await?;

            if providers.is_empty() {
                Err(anyhow!("Could not find provider for file {name}."))?;
            }

            let requests = providers.into_iter().map(|p| {
                let name = name.clone();
                let mut client = client.clone();
                async move { client.request_file(p, name).await }.boxed()
            });

            let file_content = future::select_ok(requests)
                .await
                .map_err(|_| anyhow!("None of the providers returned file."))?
                .0;

            io::stdout().write_all(&file_content)?
        }

        Argument::Provide { wasm, fun, args } => {
            let ipfs = IpfsClient::default();

            // Pull Wasm *out* of IPFS
            let wasm_bytes = ipfs
                .cat(wasm.as_str())
                .map_ok(|chunk| chunk.to_vec())
                .try_concat()
                .await
                .map(String::from_utf8)??;

            let wasm_args = future::try_join_all(args.iter().map(|arg| async {
                // Pull arg *out* of IPFS
                let vec = ipfs
                    .cat(arg.as_str())
                    .map_ok(|chunk| chunk.to_vec())
                    .try_concat()
                    .await
                    .expect("To grab arguments.");

                i32::from_str(str::from_utf8(&vec).expect("Valid Wasm value."))
                    .map(wasmer::Value::I32)
            }))
            .await?;

            let metering_middleware = Arc::new(Metering::new(10, operator::to_cost));

            let mut basic_compiler = Singlepass::new();
            let compiler_config = basic_compiler.canonicalize_nans(true);
            compiler_config.push_middleware(metering_middleware);

            let mut store = Store::new(EngineBuilder::new(compiler_config.to_owned()));

            let module = Module::new(&store, wasm_bytes).expect("Wasm module to export");

            let imports = imports! {};
            let instance =
                Instance::new(&mut store, &module, &imports).expect("Wasm instance to be here");

            let function = instance
                .exports
                .get::<Function>(fun.as_str())
                .expect("a Wasm function");

            // FIXME write Wasmer::Value -> Ipld converter
            let boxed_results: Box<[Value]> = function
                .call(&mut store, wasm_args.as_slice())
                .expect("tried to call function");

            let res: String = boxed_results
                .into_vec()
                .iter()
                .map(ToString::to_string)
                .collect();

            let resource = Url::parse(format!("ipfs://{wasm}").as_str()).expect("IPFS URL");

            let ipld_args: Ipld = Ipld::List(
                wasm_args
                    .iter()
                    .map(|a| match a {
                        Value::I32(i) => Ipld::from(*i),
                        _ => todo!(),
                    })
                    .collect(),
            );

            let closure = Closure {
                resource,
                action: Action::try_from("wasm/run")?,
                inputs: Input::IpldData(ipld_args),
            };

            let closure_ipld: Ipld = closure.clone().into();

            let mut closure_bytes = Vec::new();
            closure_ipld
                .encode(DagCborCodec, &mut closure_bytes)
                .expect("CBOR Serialization");

            let link: Link<Closure> = Closure::try_into(closure)?;
            let closure_cid = link
                .to_string_of_base(Base::Base32HexLower)
                .expect("string CID");

            let mut conn = db::establish_connection();

            let new_receipt = Receipt {
                id: Uuid::new_v4().to_string(),
                val: res.parse::<i32>().expect("i32"), // FIXME!
                closure_cid: closure_cid.clone(),
            };

            // TODO: insert (or upsert via event handling when subscribed)
            diesel::insert_into(schema::receipts::table)
                .values(&new_receipt)
                .execute(&mut conn)
                .expect("Error saving new post");

            println!("{new_receipt:?}");

            // TODO: Cleanup around pubsub execution.
            let async_client = client.clone();
            // We delay messages to make sure peers are within the mesh.
            tokio::spawn(async move {
                // TODO: make this configurable
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                let _ = async_client
                    .publish_message(RECEIPTS_TOPIC, TopicMessage::Receipt(new_receipt.clone()))
                    .await;
            });

            let res_copy = res.clone().into_bytes();

            let res_cursor = Cursor::new(res);
            let AddResponse { hash, .. } = ipfs.add(res_cursor).await.expect("a CID");

            println!("Result CID: {hash}");

            let _ = client.start_providing(closure_cid.clone()).await;

            loop {
                match events.recv().await {
                    Some(Event::InboundRequest { request, channel }) => {
                        if request.eq(&closure_cid) {
                            client.respond_file(res_copy.clone(), channel).await?;
                        }
                    }
                    e => todo!("{:?}", e),
                }
            }
        }
    }

    Ok(())
}
