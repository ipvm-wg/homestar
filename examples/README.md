# Examples

This folder contains numerous examples and demos showcasing `homestar` packages
and the `homestar runtime`. Each example is setup as its own crate,
demonstrating the necessary dependencies and setup(s).

* [websocket relay](./websocket-relay) - An example application that connects to
  the `homestar-runtime` over a websocket connection in order to run a
  couple static Wasm-based, image processing workflows that chain inputs and
  outputs via [inlined promises][pipelines].

[pipelines]: https://github.com/ucan-wg/invocation#9-pipelines
