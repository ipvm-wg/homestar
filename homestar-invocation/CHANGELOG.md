# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/ipvm-wg/homestar/compare/homestar-invocation-v0.3.0...homestar-invocation-v0.4.0) - 2024-06-26

### Other
- wasm/warg namespace cleanup ([#630](https://github.com/ipvm-wg/homestar/pull/630))

## [0.3.0](https://github.com/ipvm-wg/homestar/compare/homestar-invocation-v0.2.0...homestar-invocation-v0.3.0) - 2024-03-13

### Added
- finish interpreter ([#595](https://github.com/ipvm-wg/homestar/pull/595))

### Fixed
- Update nonce schema with IPLD bytes ([#593](https://github.com/ipvm-wg/homestar/pull/593))

### Other
- handle nonce as incoming string/arraybuf ([#611](https://github.com/ipvm-wg/homestar/pull/611))
- test json/ipld/nonce ([#610](https://github.com/ipvm-wg/homestar/pull/610))
- poll DHT in background when worker runs up a workflow + dual-stack webserver ([#590](https://github.com/ipvm-wg/homestar/pull/590))

## [0.2.0](https://github.com/ipvm-wg/homestar/compare/homestar-invocation-v0.1.1...homestar-invocation-v0.2.0) - 2024-02-20

### Added
- Add OpenRPC API docs and associated JSON Schemas ([#534](https://github.com/ipvm-wg/homestar/pull/534))

### Other
- deps + flake cleanup ([#581](https://github.com/ipvm-wg/homestar/pull/581))
- Update homestar-functions to use cargo component ([#576](https://github.com/ipvm-wg/homestar/pull/576))
- move away from deadlines dealing w/ the runner and wasi-preview 2 wasmtime ([#560](https://github.com/ipvm-wg/homestar/pull/560))
- remove unnecessary deps and add tooling for those checks ([#541](https://github.com/ipvm-wg/homestar/pull/541))
- release docs and cp readmes ([#530](https://github.com/ipvm-wg/homestar/pull/530))

## [0.1.1](https://github.com/ipvm-wg/homestar/compare/homestar-invocation-v0.1.0...homestar-invocation-v0.1.1) - 2024-01-20

### Fixed
- docs for release ([#519](https://github.com/ipvm-wg/homestar/pull/519))
