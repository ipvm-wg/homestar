# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/ipvm-wg/homestar/compare/homestar-runtime-v0.1.1...homestar-runtime-v0.2.0) - 2024-02-20

### Added
- Add OpenRPC API docs and associated JSON Schemas ([#534](https://github.com/ipvm-wg/homestar/pull/534))
- redial `node_addresses` at an interval on connection close ([#529](https://github.com/ipvm-wg/homestar/pull/529))

### Fixed
- add handling of dns multiaddrs + bootstrapping + CLI / Conn changes ([#547](https://github.com/ipvm-wg/homestar/pull/547))

### Other
- deps + flake cleanup ([#581](https://github.com/ipvm-wg/homestar/pull/581))
- Allow dead code default timeout ([#577](https://github.com/ipvm-wg/homestar/pull/577))
- Update homestar-functions to use cargo component ([#576](https://github.com/ipvm-wg/homestar/pull/576))
- fix transport order for wss possibility ([#563](https://github.com/ipvm-wg/homestar/pull/563))
- small comment, sorry ([#561](https://github.com/ipvm-wg/homestar/pull/561))
- move away from deadlines dealing w/ the runner and wasi-preview 2 wasmtime ([#560](https://github.com/ipvm-wg/homestar/pull/560))
- docker updates with info command and rpc host update ([#558](https://github.com/ipvm-wg/homestar/pull/558))
- just test conn ([#544](https://github.com/ipvm-wg/homestar/pull/544))
- handle this evil workflow_info test ([#543](https://github.com/ipvm-wg/homestar/pull/543))
- remove unnecessary deps and add tooling for those checks ([#541](https://github.com/ipvm-wg/homestar/pull/541))
- [chore(cargo)](deps): bump puffin from 0.18.1 to 0.19.0 ([#537](https://github.com/ipvm-wg/homestar/pull/537))
- updates/flaky kills on ci ([#540](https://github.com/ipvm-wg/homestar/pull/540))
- release docs and cp readmes ([#530](https://github.com/ipvm-wg/homestar/pull/530))
- port selection and test config generation macro ([#528](https://github.com/ipvm-wg/homestar/pull/528))
- [chore(cargo)](deps): bump serde_with from 3.4.0 to 3.5.0 ([#524](https://github.com/ipvm-wg/homestar/pull/524))
- [chore(cargo)](deps): bump moka from 0.12.3 to 0.12.4 ([#525](https://github.com/ipvm-wg/homestar/pull/525))

## [0.1.1](https://github.com/ipvm-wg/homestar/compare/homestar-runtime-v0.1.0...homestar-runtime-v0.1.1) - 2024-01-20

### Fixed
- docs for release ([#519](https://github.com/ipvm-wg/homestar/pull/519))

### Other
- deps-clean ([#522](https://github.com/ipvm-wg/homestar/pull/522))
