# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/ipvm-wg/homestar/compare/homestar-runtime-v0.3.0...homestar-runtime-v0.4.0) - 2024-06-26

### Added
- Add AutoNAT behavior ([#632](https://github.com/ipvm-wg/homestar/pull/632))

### Other
- *(schemas)* update OpenRPC API doc and JSON schemas
- wasm/warg namespace cleanup ([#630](https://github.com/ipvm-wg/homestar/pull/630))

## [0.3.0](https://github.com/ipvm-wg/homestar/compare/homestar-runtime-v0.2.0...homestar-runtime-v0.3.0) - 2024-03-13

### Added
- log the creation of the key file in `init`
- default `key-file` path to output directory
- support generating PEM encoded ed25519 keys from `init`
- translate `InquireError` to `miette::Result` in `init`
- recursively create config directory on `init`
- output a cleaner error when an invalid seed is passed to `init`
- support configuring key using `init` command
- run `init` command non-interactively if a TTY isn't detected
- add `--no-input` to `init` command
- add `--force` to `init` command for forcing destructive operations
- add `--quiet` flag to `init` command
- support `--dry-run` for `init` command
- add `init` command for initializing a settings file
- load settings from a well-known config file
- finish interpreter ([#595](https://github.com/ipvm-wg/homestar/pull/595))

### Fixed
- cleanup empty key file when bailing out of generating secp256k1
- parse ed25519 keys using the old strategy as a fallback
- use `ed25519-dalek` for parsing PEM-encoded PKCS[#8](https://github.com/ipvm-wg/homestar/pull/8) ed25519 keys
- only constrain `inquire` and `derive_builder` by minor version
- hide `IpfsSettings` behind "ipfs" feature
- set `truncate(true)` when forcefully overwriting the config
- Update nonce schema with IPLD bytes ([#593](https://github.com/ipvm-wg/homestar/pull/593))

### Other
- Add workflow spans and every cli logging ([#603](https://github.com/ipvm-wg/homestar/pull/603))
- *(schemas)* update OpenRPC API doc and JSON schemas
- handle nonce as incoming string/arraybuf ([#611](https://github.com/ipvm-wg/homestar/pull/611))
- [chore(cargo)](deps): Bump toml from 0.8.10 to 0.8.11 ([#612](https://github.com/ipvm-wg/homestar/pull/612))
- document that a random seed will be chosen if `key-seed` is unset
- document that if unset, a default path is used with `key-file`
- update help text for `key-file` to say it'll generate a key
- prompt for the key file as a `String` instead of `PathBuf`
- add a test for writing the generated config file + key
- split `force` field out of `OutputMode::File`
- remove `KeyTypeArg` in favor of using `KeyType`
- remove unneeded `defaults.toml`
- add simple tests for `init` command
- remove out of date TODO in `init.rs`
- remove unneeded `#[allow(dead_code)]` in `settings.rs`
- wrap all `init` args in `InitArgs` and consolidate handling
- sort imports in `cli/init.rs`
- remove docs link to private `homestar_runtime::db::pool`
- improve error for passing `--no-input` to `init` with no key
- remove extraneous `...` destructuring of `Command::Init`
- change `--config` flag to `--output` for `init` command
- move handling of `init` command to `init.rs`
- fix comments listing supported public key types
- *(schemas)* update OpenRPC API doc and JSON schemas
- *(schemas)* update OpenRPC API doc and JSON schemas
- poll DHT in background when worker runs up a workflow + dual-stack webserver ([#590](https://github.com/ipvm-wg/homestar/pull/590))
- [chore(cargo)](deps): Bump config from 0.13.4 to 0.14.0 ([#588](https://github.com/ipvm-wg/homestar/pull/588))
- [chore(cargo)](deps): Bump nix from 0.27.1 to 0.28.0 ([#587](https://github.com/ipvm-wg/homestar/pull/587))

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
