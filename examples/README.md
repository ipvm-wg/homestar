# Examples

This folder contains examples and demos showcasing `homestar` packages
and the `homestar runtime`. Each example is set up as its own crate,
demonstrating the necessary dependencies and setup(s).

* [websocket relay](./websocket-relay) - An example (browser-based) application
  that connects to the `homestar-runtime` over a WebSocket connection in order
  to run a couple static Wasm-based, image processing workflows that chain
  inputs and outputs using [inlined promises][pipelines].

[pipelines]: https://github.com/ucan-wg/invocation#9-pipelines
