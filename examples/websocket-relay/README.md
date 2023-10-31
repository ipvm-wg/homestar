# Websocket Relay

## Description

An example application that connects to a `homestar-runtime` node
over a websocket connection in order to run static Wasm-based, image
processing workflows that chain inputs and outputs using
[inlined promises][pipelines].

This application demonstrates:

  * websocket notifications of [UCAN Invocation Receipts][spec-receipts] sent
    between a web client and a `homestar` runner
  * instantaneous replay of previously run, cached executions
  * fetching content (the original static image) over [IPFS][ipfs]
    through a local blockstore
  * the [WIT][wit] + [IPLD][ipld] interpreter for
    [Wasm(time)][wasmtime] embedded execution within a `homestar` runner.

## Install

### Nix

If you're using our [Nix file](../../flake.nix), you get these installs for free.

### Manual

To get started, please install:

* [Rust][install-rust], unless you're running `homestar` [as a binary][rust-binary]
* [Node & NPM][install-npm]
* [Kubo IPFS][install-ipfs]

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

Following along with the video, once you're up and running on `localhost`,
you'll see two workflows with several tasks. You can click on the stack icon on
the top right hand corner to inspect the source of the workflows.

Running the first workflow completes a number of image-processing tasks, i.e.
`crop` -> `rotate90` -> `blur`, submitting the rendered output to each
subsequent task.

The second workflow executes `crop` -> `rotate90` as well, and then something
new: `grayscale`. As demonstrated, the first two task executions can be skipped
if they've been previously run.

## Tips & Common Issues

On macOS, for example, a simple homebrew install would install everything you
need: `brew install rust npm ipfs`

We have packaged homestar binaries via brew, so
`brew install fission-codes/fission/homestar` will install everything you need,
including `ipfs`. You will still need npm to run this example. From this folder,
you can then run the example like this:

```
homestar start --config ./config/settings.toml --db homestar.db`
```

Running `homestar` via `cargo run` requires a minimum Rust version of
`1.70.0`. If you've got an older install of rust, update it with
`rustup update`.

You do not have to start-up Kubo (IPFS) on your own. The example will do this
for you, and use `examples/websocket-relay/tmp/.ipfs` as a local blockstore.
Feel free to discard it when you don't need it.

If you're already running an IPFS instance however, e.g. [IPFS Desktop][ipfs-desktop],
the example will check for an already running instance and not start a new,
local one.

[install-ipfs]: https://docs.ipfs.tech/install/
[install-npm]: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
[install-rust]: https://www.rust-lang.org/tools/install
[ipfs]: https://ipfs.tech/
[ipfs-desktop]: https://docs.ipfs.tech/install/ipfs-desktop/
[ipld]: https://ipld.io/
[pipelines]: https://github.com/ucan-wg/invocation#9-pipelines
[rust-binary]: https://doc.rust-lang.org/book/ch01-03-hello-cargo.html#building-for-release
[spec-receipts]: https://github.com/ucan-wg/invocation#8-receipt
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
