# Websocket Relay

## Description

An example application that connects to the `homestar-runtime` over a websocket
connection in order to run a couple static Wasm-based, image processing
workflows that chain inputs and outputs via [inlined promises][pipelines]. This
application demonstrates:

  * websocket notifications of [Ucan Invocation receipts][spec-receipts] sent
    between a UI-client and a `homestar` runner
  * instantaneous replay of previously run, cached executions
  * fetching of content (the original static image) over [IPFS][ipfs]
    through a local blockstore
  * the [WIT][wit] + [Ipld][ipld] interpreter in-action for
    [Wasm(time)][wasmtime] embedded execution within a `homestar` runner.

## Usage

1. Run `cargo run -- start -c config/settings.toml` to start the runtime and
   (for example purposes), an IPFS daemon as a background process. This will
   feature ANSI-coded logging by default for the *relay web application*.

2. In a separate terminal window, run `npm install --prefix relay-app` to
   install dependencies, and then `npm run --prefix relay-app dev` to start the
   relay web application (UI) on `http://localhost:5173/` by default.

3. Press the *play* buttons on the UI to run workflows. Follow along with the
   video for more information.

TODO: Embed Video

[ipfs]: https://ipfs.tech/
[ipld]: https://ipld.io/
[pipelines]: https://github.com/ucan-wg/invocation#9-pipelines
[spec-receipts]: https://github.com/ucan-wg/invocation#8-receipt
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
