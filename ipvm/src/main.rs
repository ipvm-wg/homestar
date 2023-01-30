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
    wasm::wasmtime::shim,
    workflow::{
        closure::{Action, Closure, Input},
        receipt::Receipt,
    },
};
use itertools::Itertools;
use libipld::{
    cbor::DagCborCodec,
    cid::{multibase::Base, Cid},
    prelude::Codec,
    Ipld, Link,
};
use libp2p::{
    core::PeerId,
    futures::{future, FutureExt, TryStreamExt},
    identity::Keypair,
    multiaddr::Protocol,
};
use std::{
    io::{self, Cursor, Write},
    str::{self, FromStr},
};
use url::Url;
use wasmtime::{component::Linker, Config, Engine, Store};

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

    // TODO: abstraction for this and redo inner parts, around ownership, etc.
    match opts.argument {
        Argument::Get { name } => {
            let cid_name = Cid::from_str(&name)?;
            let cid_string = cid_name.to_string_of_base(Base::Base32Lower)?;
            let providers = client.get_providers(cid_string.clone()).await?;

            if providers.is_empty() {
                Err(anyhow!("could not find provider for file {name}"))?;
            }

            let requests = providers.into_iter().map(|p| {
                let mut client = client.clone();
                let name = cid_string.clone();
                async move { client.request_file(p, name).await }.boxed()
            });

            let file_content = future::select_ok(requests)
                .await
                .map_err(|_| anyhow!("none of the providers returned file"))?
                .0;

            io::stdout().write_all(&file_content)?
        }

        Argument::Provide { wasm, fun, args } => {
            let ipfs = IpfsClient::default();

            // Pull Wasm (module) *out* of IPFS
            let wasm_bytes = ipfs
                .cat(wasm.as_str())
                .map_ok(|chunk| chunk.to_vec())
                .try_concat()
                .await?;

            let wasm_args =
                // Pull arg *out* of IPFS
                future::try_join_all(args.iter().map(|arg| async {

                  ipfs
                    .cat(arg.as_str())
                    .map_ok(|chunk| {
                    chunk.to_vec()
                    })
                    .try_concat()
                    .await

                })).await?;

            // TODO: Don't read randomly from file.
            // The interior of this is test specific code,
            // unil we use a format for params, like Json.
            let ipld_args = wasm_args
                .iter()
                .map(|a| {
                    if let Ok(arg) = str::from_utf8(a) {
                        match i32::from_str(arg) {
                            Ok(num) => Ok::<Ipld, anyhow::Error>(Ipld::from(num)),
                            Err(_e) => Ok::<Ipld, anyhow::Error>(Ipld::from(arg)),
                        }
                    } else {
                        Err(anyhow!("Unreadable input bytes: {a:?}"))
                    }
                })
                .fold_ok(vec![], |mut acc, elem| {
                    acc.push(elem);
                    acc
                })?;

            let mut config = Config::new();
            config.wasm_component_model(true);
            config.async_support(true);
            let engine = Engine::new(&config)?;

            let linker = Linker::new(&engine);
            let mut store = Store::new(&engine, ());

            let component = shim::component_from_bytes(&wasm_bytes, engine)?;
            let (bindings, _instance) =
                shim::Wasmtime::instantiate(&mut store, &component, &linker, fun).await?;
            let res = bindings.execute(store, ipld_args.clone()).await?;

            let resource = Url::parse(format!("ipfs://{wasm}").as_str()).expect("IPFS URL");

            let closure = Closure {
                resource,
                action: Action::try_from("wasm/run")?,
                inputs: Input::IpldData(Ipld::List(ipld_args)),
            };

            let link: Link<Closure> = Closure::try_into(closure)?;
            let mut conn = db::establish_connection();

            let receipt = Receipt::new(link, res);

            // TODO: insert (or upsert via event handling when subscribed)
            diesel::insert_into(schema::receipts::table)
                .values(&receipt)
                .execute(&mut conn)
                .expect("Error saving new receipt");

            println!("{receipt}");

            let receipt_val = receipt.value();
            let encoded = DagCborCodec.encode(&receipt_val)?;
            let AddResponse { hash: _, .. } = ipfs.add(Cursor::new(encoded)).await.expect("a CID");

            let closure_cid = receipt.closure_cid();
            let async_client = client.clone();
            // We delay messages to make sure peers are within the mesh.
            tokio::spawn(async move {
                // TODO: make this configurable, but currently matching heartbeat.

                let _ = async_client
                    .publish_message(RECEIPTS_TOPIC, TopicMessage::Receipt(receipt.clone()))
                    .await;
            });

            let _ = client.start_providing(closure_cid.clone()).await;

            loop {
                match events.recv().await {
                    Some(Event::InboundRequest { request, channel }) => {
                        if request.eq(&closure_cid) {
                            let output = format!("{:?}", receipt_val);
                            client.respond_file(output.into_bytes(), channel).await?;
                        }
                    }
                    e => todo!("{:?}", e),
                }
            }
        }
    }

    Ok(())
}
