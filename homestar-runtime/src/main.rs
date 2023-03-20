use anyhow::{anyhow, bail, Result};
use clap::Parser;
use diesel::RunQueryDsl;
use homestar_core::workflow::{
    config::Resources, input::Parse, prf::UcanPrf, receipt::Receipt as LocalReceipt, Ability,
    Input, Invocation, InvocationResult, Task,
};
use homestar_runtime::{
    cli::{Args, Argument},
    db::{self, schema},
    network::{
        client::Client,
        eventloop::{Event, RECEIPTS_TOPIC},
        swarm::{self, Topic, TopicMessage},
    },
    Receipt,
};
use homestar_wasm::wasmtime;
use ipfs_api::{
    request::{DagCodec, DagPut},
    response::DagPutResponse,
    IpfsApi, IpfsClient,
};
use itertools::Itertools;
use libipld::{
    cid::{multibase::Base, Cid},
    Ipld,
};
use libp2p::{
    futures::{future, TryStreamExt},
    identity::Keypair,
    multiaddr::Protocol,
};
use libp2p_identity::PeerId;
use std::{
    collections::BTreeMap,
    io::{stdout, Cursor, Write},
    str::{self, FromStr},
};
use url::Url;

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
    // TODO: cleanup-up use, clones, etc.
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
                #[allow(unknown_lints, clippy::redundant_async_block)]
                Box::pin(async move { client.request_file(p, name).await })
            });

            let file_content = future::select_ok(requests)
                .await
                .map_err(|_| anyhow!("none of the providers returned file"))?
                .0;

            stdout().write_all(&file_content)?
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
                future::try_join_all(args.iter().map(|arg|
                  ipfs
                    .cat(arg.as_str())
                    .map_ok(|chunk| {
                    chunk.to_vec()
                    })
                    .try_concat()
                )).await?;

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

            // TODO: Only works off happy path, but need to work with traps to
            // capture error.
            // TODO: State will derive from resources, other configuration.
            let resource = Url::parse(format!("ipfs://{wasm}").as_str()).expect("IPFS URL");

            let task = Task::new(
                resource,
                Ability::from("wasm/run"),
                Input::Ipld(Ipld::Map(BTreeMap::from([(
                    "args".into(),
                    Ipld::List(ipld_args),
                )]))),
                None,
            );
            let config = Resources::default();
            let invocation = Invocation::new(
                task.clone().into(),
                config.clone().into(),
                UcanPrf::default(),
            )?;

            let mut env =
                wasmtime::World::instantiate(wasm_bytes, fun, wasmtime::State::default()).await?;
            let res = env.execute(task.input().parse()?.try_into()?).await?;

            let local_receipt = LocalReceipt::new(
                invocation.try_into()?,
                InvocationResult::Ok(res.try_into()?),
                Ipld::Null,
                None,
                UcanPrf::default(),
            );
            let receipt = Receipt::try_from(&local_receipt)?;

            let receipt_bytes: Vec<u8> = local_receipt.try_into()?;
            let dag_builder = DagPut::builder()
                .input_codec(DagCodec::Cbor)
                .hash("sha3-256") // sadly no support for blake3-256
                .build();
            let DagPutResponse { cid } = ipfs
                .dag_put_with_options(Cursor::new(receipt_bytes.clone()), dag_builder)
                .await
                .expect("a CID");

            // //Test for now
            assert_eq!(cid.cid_string, receipt.cid());

            let mut conn = db::establish_connection();
            // TODO: insert (or upsert via event handling when subscribed)
            diesel::insert_into(schema::receipts::table)
                .values(&receipt)
                .on_conflict(schema::receipts::cid)
                .do_nothing()
                .execute(&mut conn)
                .expect("Error saving new receipt");
            println!("stored: {receipt}");

            let invoked_cid = receipt.ran();
            let output = receipt.output().clone();
            let async_client = client.clone();
            // We delay messages to make sure peers are within the mesh.
            tokio::spawn(async move {
                // TODO: make this configurable, but currently matching heartbeat.
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let _ = async_client
                    .publish_message(
                        Topic::new(RECEIPTS_TOPIC.to_string()),
                        TopicMessage::Receipt(receipt),
                    )
                    .await;
            });

            let _ = client.start_providing(invoked_cid.clone()).await;

            loop {
                match events.recv().await {
                    Some(Event::InboundRequest { request, channel }) => {
                        if request.eq(&invoked_cid) {
                            let output = format!("{output:?}");
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
