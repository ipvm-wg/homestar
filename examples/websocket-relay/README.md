# Websocket Relay

## Description

An example application that connects to a `homestar-runtime` node
over a websocket connection in order to run static Wasm-based, image
processing workflows that chain inputs and outputs using
[inlined promises][pipelines].

This application demonstrates:

  * websocket notifications of [Ucan Invocation receipts][spec-receipts] sent
    between a web client and a `homestar` runner
  * instantaneous replay of previously run, cached executions
  * fetching content (the original static image) over [IPFS][ipfs]
    through a local blockstore
  * the [WIT][wit] + [Ipld][ipld] interpreter for
    [Wasm(time)][wasmtime] embedded execution within a `homestar` runner.

## Install

To get started, please install:

* [rust][install-rust], unless you're running `homestar` [as a binary][rust-binary]
* [node & npm][install-npm]
* [kubo/ipfs installed][install-ipfs]

If you're using our [nix file](../../flake.nix)], you get these installs for
free.

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

Once you're up and running on localhost, you'll see two workflows with several tasks. You can click on the stack icon on the top right hand corner to inspect the source of the Workflows.

Running the first workflow completes a number of tasks and then submits the output to the next task.

The second workflow uses some of the same tasks, and demonstrates how tasks can be skipped if they've been previously run.

## Tips & Common Issues

On MacOS, verything you need can be installed with brew: `brew install rust npm ipfs`.

If you've got an older install of rust, update it with `rustup update`.

You do not have to start-up `kubo`/`ipfs` on your own. The example will do this for you, and use `examples/websocket-relay/tmp/.ipfs` for a local blockstore. If you're already running an IPFS instance, e.g. IPFS Desktop, please quit it while running this example.

[install-ipfs]: https://docs.ipfs.tech/install/
[install-npm]: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
[install-rust]: https://www.rust-lang.org/tools/install
[ipfs]: https://ipfs.tech/
[ipld]: https://ipld.io/
[pipelines]: https://github.com/ucan-wg/invocation#9-pipelines
[rust-binary]: https://doc.rust-lang.org/book/ch01-03-hello-cargo.html#building-for-release
[spec-receipts]: https://github.com/ucan-wg/invocation#8-receipt
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
