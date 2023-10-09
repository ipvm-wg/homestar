# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/ipvm-wg/homestar/releases/tag/homestar-runtime-v0.1.0) - 2023-10-09

### Added
- builds, packages, pushing contains to ghcr ([#360](https://github.com/ipvm-wg/homestar/pull/360))
- Add out-of-band node metrics ([#299](https://github.com/ipvm-wg/homestar/pull/299))
- Rendezvous for peer discovery ([#236](https://github.com/ipvm-wg/homestar/pull/236))

### Other
- flaky test updates ([#359](https://github.com/ipvm-wg/homestar/pull/359))
- README updates, example updates ([#352](https://github.com/ipvm-wg/homestar/pull/352))
- deps-run! ([#343](https://github.com/ipvm-wg/homestar/pull/343))
- updates around bin warning and windows-latest+nightly ([#329](https://github.com/ipvm-wg/homestar/pull/329))
- [chore(cargo)](deps): Bump clap from 4.4.4 to 4.4.5 ([#325](https://github.com/ipvm-wg/homestar/pull/325))
- deps 9-18 & filter CI ([#319](https://github.com/ipvm-wg/homestar/pull/319))
- default-features, dev-mode, feature-fun ([#308](https://github.com/ipvm-wg/homestar/pull/308))
- Example application with image processing workflow ([#263](https://github.com/ipvm-wg/homestar/pull/263))
- deps b/c clippy ([#292](https://github.com/ipvm-wg/homestar/pull/292))
- help dependabot ([#280](https://github.com/ipvm-wg/homestar/pull/280))
- [chore(cargo)](deps): bump serde_with from 3.2.0 to 3.3.0 ([#265](https://github.com/ipvm-wg/homestar/pull/265))
- Swarm Bootstrap Nodes ([#201](https://github.com/ipvm-wg/homestar/pull/201))
- Prep for first example application with various fixes and changes. ([#235](https://github.com/ipvm-wg/homestar/pull/235))
- Add capture for windows signals ([#234](https://github.com/ipvm-wg/homestar/pull/234))
- for mdns, do not add explicit peer, just add to kademlia ([#232](https://github.com/ipvm-wg/homestar/pull/232))
- [chore(cargo)](deps): Bump tabled from 0.13.0 to 0.14.0 ([#230](https://github.com/ipvm-wg/homestar/pull/230))
- [chore(cargo)](deps): Bump serde_with from 3.1.0 to 3.2.0 ([#231](https://github.com/ipvm-wg/homestar/pull/231))
- e2e workflow(s) run ([#223](https://github.com/ipvm-wg/homestar/pull/223))
- [chore(cargo)](deps): Bump tokio-tungstenite from 0.19.0 to 0.20.0 ([#210](https://github.com/ipvm-wg/homestar/pull/210))
- Runner + RPC ([#203](https://github.com/ipvm-wg/homestar/pull/203))
- [chore(cargo)](deps): Bump dashmap from 5.4.0 to 5.5.0 ([#197](https://github.com/ipvm-wg/homestar/pull/197))
- [chore(cargo)](deps): Bump serde_with from 3.0.0 to 3.1.0 ([#195](https://github.com/ipvm-wg/homestar/pull/195))
- move away from inner-join, count up unique receipts for workflow info ([#188](https://github.com/ipvm-wg/homestar/pull/188))
- libp2p get providers and handle peer requests for records ([#185](https://github.com/ipvm-wg/homestar/pull/185))
- Update wasmtime to 10.0.*, along w/ wasmparser & other rela… ([#181](https://github.com/ipvm-wg/homestar/pull/181))
- first-pass at runtime interface, signals, shutdown, and more ([#180](https://github.com/ipvm-wg/homestar/pull/180))
- Add pubkey importing / seed generation ([#172](https://github.com/ipvm-wg/homestar/pull/172))
- ** for all prs ([#182](https://github.com/ipvm-wg/homestar/pull/182))
- [chore(cargo)](deps): Bump indexmap from 1.9.3 to 2.0.0 ([#168](https://github.com/ipvm-wg/homestar/pull/168))
- workspace inherit fun again ([#155](https://github.com/ipvm-wg/homestar/pull/155))
- workspace inherit fun ([#153](https://github.com/ipvm-wg/homestar/pull/153))
- add memory limits, update scheduler, progress and info ([#144](https://github.com/ipvm-wg/homestar/pull/144))
- to/from json, traits for cid ([#142](https://github.com/ipvm-wg/homestar/pull/142))
- error handling with thiserror, anyhow for runtime only ([#139](https://github.com/ipvm-wg/homestar/pull/139))
- worker tests, cargo nextest, and better handle on workflow progress ([#137](https://github.com/ipvm-wg/homestar/pull/137))
- first run-up of docker for runtime ([#135](https://github.com/ipvm-wg/homestar/pull/135))
- massive jump toward a legit homestar runtime ([#133](https://github.com/ipvm-wg/homestar/pull/133))
- Add image examples ([#124](https://github.com/ipvm-wg/homestar/pull/124))
- clippy allow due to subtle behavior ([#72](https://github.com/ipvm-wg/homestar/pull/72))
- Breakout crates: core, runtime, wasm and prep for generic/diff inputs, parse step ([#69](https://github.com/ipvm-wg/homestar/pull/69))
