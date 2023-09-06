# Websocket Relay

## Description

An example application that connects to a **single** `homestar-runtime` node
over a websocket connection in order to run static Wasm-based, image
processing workflows that chain inputs and outputs using
[inlined promises][pipelines]. This application demonstrates:

  * websocket notifications of [Ucan Invocation receipts][spec-receipts] sent
    between a web client and a `homestar` runner
  * instantaneous replay of previously run, cached executions
  * fetching content (the original static image) over [IPFS][ipfs]
    through a local blockstore
  * the [WIT][wit] + [Ipld][ipld] interpreter for
    [Wasm(time)][wasmtime] embedded execution within a `homestar` runner.

## Usage

1. Run `cargo run -- start -c config/settings.toml` to start the runtime and
   an IPFS daemon as a background process. This runtime includes
   ANSI-coded logging by default.

2. In a separate terminal window, run `npm install --prefix relay-app` to
   install dependencies and `npm run --prefix relay-app dev` to start the
   relay web application (UI) on `http://localhost:5173/` by default.

3. Press the *play* buttons on the UI to run workflows. Follow along with this
   video for more information.

   https://www.loom.com/share/b0f882adc2ea45709d1f3031b5e61e92?sid=29cb403e-c666-4753-82f5-e35bbb710151

Note that IPFS may attempt to upgrade to a new version and produce an error after the update. Delete the `tmp/.ipfs/` directory and restart to reset the IPFS repo state.

[ipfs]: https://ipfs.tech/
[ipld]: https://ipld.io/
[pipelines]: https://github.com/ucan-wg/invocation#9-pipelines
[spec-receipts]: https://github.com/ucan-wg/invocation#8-receipt
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
