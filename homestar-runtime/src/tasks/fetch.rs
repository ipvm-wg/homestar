//! Fetch module for gathering data over the network related to [Task]
//! resources.
//!
//! [Task]: homestar_core::workflow::Task

#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
#[cfg(any(test, feature = "test-utils"))]
use crate::tasks::WasmContext;
use crate::workflow::{self, Resource};
use anyhow::Result;
#[cfg(all(feature = "ipfs", not(test), not(feature = "test-utils")))]
use futures::{stream::FuturesUnordered, TryStreamExt};
use indexmap::IndexMap;
#[cfg(all(feature = "ipfs", not(test), not(feature = "test-utils")))]
use libipld::Cid;
use std::sync::Arc;
#[cfg(all(feature = "ipfs", not(test), not(feature = "test-utils")))]
use tryhard::RetryFutureConfig;

/// Gather resources from IPFS or elsewhere, leveraging an exponential backoff.
#[cfg(all(feature = "ipfs", not(test), not(feature = "test-utils")))]
#[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
pub(crate) async fn get_resources(
    resources: Vec<Resource>,
    settings: Arc<workflow::Settings>,
    ipfs: IpfsCli,
) -> Result<IndexMap<Resource, Vec<u8>>> {
    let settings = settings.as_ref();
    let tasks = FuturesUnordered::new();
    for rsc in resources.iter() {
        let task = tryhard::retry_fn(|| async { fetch(rsc.clone(), ipfs.clone()).await })
            .with_config(
                RetryFutureConfig::new(settings.retries)
                    .exponential_backoff(settings.retry_initial_delay)
                    .max_delay(settings.retry_max_delay),
            );

        tasks.push(task);
    }

    tasks.try_collect::<Vec<_>>().await?.into_iter().try_fold(
        IndexMap::default(),
        |mut acc, res| {
            let answer = res.1?;
            acc.insert(res.0, answer);
            Ok::<_, anyhow::Error>(acc)
        },
    )
}

/// Gather resources via URLs, leveraging an exponential backoff.
/// TODO: Client calls (only) over http(s).
#[cfg(all(not(feature = "ipfs"), not(test), not(feature = "test-utils")))]
#[allow(dead_code)]
pub(crate) async fn get_resources<T>(
    _resources: Vec<Resource>,
    _settings: Arc<workflow::Settings>,
) -> Result<IndexMap<Resource, T>> {
    Ok(IndexMap::default())
}

#[cfg(all(not(feature = "ipfs"), any(test, feature = "test-utils")))]
#[doc(hidden)]
#[allow(dead_code)]
pub(crate) async fn get_resources(
    _resources: Vec<Resource>,
    _settings: Arc<workflow::Settings>,
) -> Result<IndexMap<Resource, Vec<u8>>> {
    println!("Running in test mode");
    use crate::tasks::FileLoad;
    let path = std::path::PathBuf::from(format!(
        "{}/../homestar-wasm/fixtures/example_test.wasm",
        env!("CARGO_MANIFEST_DIR")
    ));
    let bytes = WasmContext::load(path).await?;
    let mut map = IndexMap::default();
    let rsc = "ipfs://bafybeihzvrlcfqf6ffbp2juhuakspxj2bdsc54cabxnuxfvuqy5lvfxapy";
    map.insert(Resource::Url(url::Url::parse(rsc)?), bytes);
    Ok(map)
}

#[cfg(all(feature = "ipfs", any(test, feature = "test-utils")))]
#[doc(hidden)]
#[allow(dead_code)]
pub(crate) async fn get_resources(
    _resources: Vec<Resource>,
    _settings: Arc<workflow::Settings>,
    _ipfs: IpfsCli,
) -> Result<IndexMap<Resource, Vec<u8>>> {
    println!("Running in test mode");
    use crate::tasks::FileLoad;
    let path = std::path::PathBuf::from(format!(
        "{}/../homestar-wasm/fixtures/example_test.wasm",
        env!("CARGO_MANIFEST_DIR")
    ));
    let bytes = WasmContext::load(path).await?;
    let mut map = IndexMap::default();
    let rsc = "ipfs://bafybeihzvrlcfqf6ffbp2juhuakspxj2bdsc54cabxnuxfvuqy5lvfxapy";
    map.insert(Resource::Url(url::Url::parse(rsc)?), bytes);
    Ok(map)
}

#[cfg(all(feature = "ipfs", not(test), not(feature = "test-utils")))]
#[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
async fn fetch(rsc: Resource, client: IpfsCli) -> Result<(Resource, Result<Vec<u8>>)> {
    match rsc {
        Resource::Url(url) => {
            let bytes = match (url.scheme(), url.domain(), url.path()) {
                ("ipfs", Some(cid), _) => {
                    let cid = Cid::try_from(cid)?;
                    client.get_cid(cid).await
                }
                (_, Some("ipfs.io"), _) => client.get_resource(&url).await,
                (_, _, path) if path.contains("/ipfs/") || path.contains("/ipns/") => {
                    client.get_resource(&url).await
                }
                (_, Some(domain), _) => {
                    let split: Vec<&str> = domain.splitn(3, '.').collect();
                    // subdomain-gateway case:
                    // <https://bafybeiemxf5abjwjbikoz4mc3a3dla6ual3jsgpdr4cjr3oz3evfyavhwq.ipfs.dweb.link/wiki/>
                    if let (Ok(_cid), "ipfs") = (Cid::try_from(split[0]), split[1]) {
                        client.get_resource(&url).await
                    } else {
                        // TODO: reqwest call
                        todo!()
                    }
                }
                // TODO: reqwest call
                (_, _, _) => todo!(),
            };
            Ok((Resource::Url(url), bytes))
        }

        Resource::Cid(cid) => {
            // TODO: Check blockstore first.
            let bytes = client.get_cid(cid).await;
            Ok((Resource::Cid(cid), bytes))
        }
    }
}
