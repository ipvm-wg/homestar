//! Fetch module for gathering data over the network related to [Task]
//! resources.
//!
//! [Task]: homestar_core::workflow::Task

#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::workflow::{self, Resource};
use anyhow::Result;
use fnv::FnvHashSet;
use indexmap::IndexMap;
use std::sync::Arc;

pub(crate) struct Fetch;

#[cfg(any(test, feature = "test-utils"))]
const WASM_CID: &str = "bafybeiczefaiu7464ehupezpzulnti5jvcwnvdalqrdliugnnwcdz6ljia";
#[cfg(any(test, feature = "test-utils"))]
const CAT_CID: &str = "bafybeiejevluvtoevgk66plh5t6xiy3ikyuuxg3vgofuvpeckb6eadresm";

impl Fetch {
    /// Gather resources from IPFS or elsewhere, leveraging an exponential backoff.
    #[cfg(all(feature = "ipfs", not(test), not(feature = "test-utils")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    pub(crate) async fn get_resources(
        resources: FnvHashSet<Resource>,
        settings: Arc<workflow::Settings>,
        ipfs: IpfsCli,
    ) -> Result<IndexMap<Resource, Vec<u8>>> {
        use futures::{stream::FuturesUnordered, TryStreamExt};
        let settings = settings.as_ref();
        let tasks = FuturesUnordered::new();
        for rsc in resources.iter() {
            tracing::info!(rsc = rsc.to_string(), "Fetching resource");
            let task = tryhard::retry_fn(|| async { Self::fetch(rsc.clone(), ipfs.clone()).await })
                .with_config(
                    tryhard::RetryFutureConfig::new(settings.retries)
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
    #[cfg(all(not(feature = "ipfs"), not(test)))]
    #[allow(dead_code)]
    pub(crate) async fn get_resources<T>(
        _resources: FnvHashSet<Resource>,
        _settings: Arc<workflow::Settings>,
    ) -> Result<IndexMap<Resource, T>> {
        Ok(IndexMap::default())
    }

    #[cfg(all(not(feature = "ipfs"), any(test, feature = "test-utils")))]
    #[doc(hidden)]
    #[allow(dead_code)]
    pub(crate) async fn get_resources(
        _resources: FnvHashSet<Resource>,
        _settings: Arc<workflow::Settings>,
    ) -> Result<IndexMap<Resource, Vec<u8>>> {
        println!("Running in test mode");
        use crate::tasks::FileLoad;
        let wasm_path = std::path::PathBuf::from(format!(
            "{}/../homestar-wasm/fixtures/example_test.wasm",
            env!("CARGO_MANIFEST_DIR")
        ));
        let img_path = std::path::PathBuf::from(format!(
            "{}/../examples/websocket-relay/synthcat.png",
            env!("CARGO_MANIFEST_DIR")
        ));

        let bytes = crate::tasks::WasmContext::load(wasm_path).await.unwrap();
        let buf = crate::tasks::WasmContext::load(img_path).await.unwrap();
        let mut map = IndexMap::default();
        map.insert(
            Resource::Url(url::Url::parse(format!("ipfs://{WASM_CID}").as_str()).unwrap()),
            bytes,
        );
        map.insert(Resource::Cid(libipld::Cid::try_from(CAT_CID).unwrap()), buf);
        Ok(map)
    }

    #[cfg(all(feature = "ipfs", any(test, feature = "test-utils")))]
    #[doc(hidden)]
    #[allow(dead_code)]
    pub(crate) async fn get_resources(
        _resources: FnvHashSet<Resource>,
        _settings: Arc<workflow::Settings>,
        _ipfs: IpfsCli,
    ) -> Result<IndexMap<Resource, Vec<u8>>> {
        println!("Running in test mode");
        use crate::tasks::FileLoad;
        let wasm_path = std::path::PathBuf::from(format!(
            "{}/../homestar-wasm/fixtures/example_test.wasm",
            env!("CARGO_MANIFEST_DIR")
        ));
        let img_path = std::path::PathBuf::from(format!(
            "{}/../examples/websocket-relay/synthcat.png",
            env!("CARGO_MANIFEST_DIR")
        ));

        println!("Running in test mode: {}", img_path.display());

        let bytes = crate::tasks::WasmContext::load(wasm_path).await.unwrap();
        let buf = crate::tasks::WasmContext::load(img_path).await.unwrap();
        let mut map = IndexMap::default();
        map.insert(
            Resource::Url(url::Url::parse(format!("ipfs://{WASM_CID}").as_str()).unwrap()),
            bytes,
        );
        map.insert(Resource::Cid(libipld::Cid::try_from(CAT_CID).unwrap()), buf);
        Ok(map)
    }

    #[cfg(all(feature = "ipfs", not(test), not(feature = "test-utils")))]
    async fn fetch(rsc: Resource, client: IpfsCli) -> Result<(Resource, Result<Vec<u8>>)> {
        match rsc {
            Resource::Url(url) => {
                let bytes = match (url.scheme(), url.domain(), url.path()) {
                    ("ipfs", Some(cid), _) => {
                        let parsed_cid = libipld::Cid::try_from(cid)?;
                        client.get_cid(parsed_cid).await
                    }
                    (_, Some("ipfs.io"), _) => client.get_resource(&url).await,
                    (_, _, path) if path.contains("/ipfs/") || path.contains("/ipns/") => {
                        client.get_resource(&url).await
                    }
                    (_, Some(domain), _) => {
                        let split: Vec<&str> = domain.splitn(3, '.').collect();
                        // subdomain-gateway case:
                        // <https://bafybeiemxf5abjwjbikoz4mc3a3dla6ual3jsgpdr4cjr3oz3evfyavhwq.ipfs.dweb.link/wiki/>
                        if let (Ok(_cid), "ipfs") = (libipld::Cid::try_from(split[0]), split[1]) {
                            client.get_resource(&url).await
                        } else {
                            // TODO: reqwest call or error
                            todo!()
                        }
                    }
                    // TODO: reqwest call or error
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
}
