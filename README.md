<div align="center">
  <a href="https://github.com/ipvm-wg/homestar" target="_blank">
    <img src="https://raw.githubusercontent.com/ipvm-wg/homestar/main/assets/a_logo.png" alt="homestar Logo" width="100"></img>
  </a>

  <h1 align="center">homestar</h1>

  <p>
    <a href="https://crates.io/crates/homestar-core">
      <img src="https://img.shields.io/crates/v/homestar-core?label=crates" alt="Crate">
    </a>
    <a href="https://crates.io/crates/homestar-wasm">
      <img src="https://img.shields.io/crates/v/homestar-wasm?label=crates" alt="Crate">
    </a>
    <a href="https://crates.io/crates/homestar-runtime">
      <img src="https://img.shields.io/crates/v/homestar-runtime?label=crates" alt="Crate">
    </a>
    <a href="https://codecov.io/gh/ipvm-wg/homestar">
      <img src="https://codecov.io/gh/ipvm-wg/homestar/branch/main/graph/badge.svg?token=SOMETOKEN" alt="Code Coverage"/>
    </a>
    <a href="https://github.com/ipvm-wg/homestar/actions/workflows/tests_and_checks.yml">
      <img src="https://github.com/ipvm-wg/homestar/actions/workflows/tests_and_checks.yml/badge.svg" alt="Tests and Checks Status">
    </a>
    <a href="https://github.com/ipvm-wg/homestar/actions/workflows/docker.yml">
      <img src="https://github.com/ipvm-wg/homestar/actions/workflows/docker.yml/badge.svg" alt="Build Docker Status">
    </a>
    <a href="https://github.com/ipvm-wg/homestar/actions/workflows/audit.yml">
      <img src="https://github.com/ipvm-wg/homestar/actions/workflows/audit.yml/badge.svg" alt="Cargo Audit Status">
    </a>
    <a href="https://github.com/ipvm-wg/homestar/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/homestar-core">
      <img src="https://img.shields.io/static/v1?label=Docs&message=core.docs.rs&color=pink" alt="Docs">
    </a>
    <a href="https://docs.rs/homestar-wasm">
      <img src="https://img.shields.io/static/v1?label=Docs&message=wasm.docs.rs&color=pink" alt="Docs">
    </a>
    <a href="https://docs.rs/homestar-runtime">
      <img src="https://img.shields.io/static/v1?label=Docs&message=runtime.docs.rs&color=pink" alt="Docs">
    </a>
    <a href="https://fission.codes/discord">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

##

## Outline

- [Quickstart](#quickstart)
- [Packages](#packages)
- [Running Examples](#running-examples)
- [Workspace](#workspace)
- [Contributing](#contributing)
- [Releases](#releases)
- [Getting Help](#getting-help)
- [External Resources](#external-resources)
- [License](#license)

## Quickstart

If you're looking to help develop `homestar`, please dive right into our
[development](./DEVELOPMENT.md) guide.

Otherwise, the easiest way to get started and see `homestar` in action is to
follow-along and run our image-processing
[websocket relay](./examples/websocket-relay) example, which integrates
`homestar` with a browser application to run a
statically-configured workflow. The associated `README.md` walks through
what to install (i.e. `rust`, `node/npm`, `ipfs`), what commands
to run, and embeds a video demonstrating its usage.

Throughout the `homestar` ecosystem and documentation, we'll draw a distinction
between the [host runtime][host-runtime] and the support for different
[guest languages and bindings][guest].

If you're mainly interested in learning how to write and build-out Wasm
components (currently focused on authoring in Rust), please jump into
our [`homestar-functions`](./homestar-functions) directory and check out
our examples there.

## Packages

Each `homestar` release will also build packages for distribution across
different platforms.

- [homebrew][homebrew]: `brew install fission-codes/fission/homestar`
  This includes `ipfs` in the install by default.

## Running Examples

All [examples](./examples) contain instructions for running
them, including what to install and how to run them. Please clone this repo,
and get started!

Each example showcases something specific and interesting about `homestar`
as a system.

Our current list includes:

- [websocket relay](./examples/websocket-relay/README.md) - An example
  (browser-based) application that connects to the `homestar-runtime` over a
  websocket connection in order to run a couple static Wasm-based, image
  processing workflows that chain inputs and outputs.

## Workspace

This repository is comprised of a few library packages and a library/binary that
represents the `homestar` runtime. We recommend diving into each package's own
`README.md` for more information when available.

### Core Crates

- [homestar-core](./homestar-core)

  The *core* library implements much of the [Ucan Invocation][ucan-invocation]
  and [Ipvm Workflow][ipvm-workflow-spec] specifications and is used as the
  foundation for other packages in this `workspace` and within the runtime
  engine.

- [homestar-wasm](./homestar-wasm)

  This *wasm* library manages the [wasmtime][wasmtime] runtime, provides the
  [Ipld][ipld] to/from [Wit][wit] interpreter/translation-layer, and implements
  the input interface for working with Ipvm's standard Wasm tasks.

### Runtime Crate

- [homestar-runtime](./homestar-runtime)

  The *runtime* is responsible for bootstrapping and running nodes, scheduling
  and executing workflows as well as tasks within workflows, handling retries
  and failure modes, etc.

### Non-published Crates

- [homestar-functions/*](./homestar-functions)

  `homestar-functions` is a directory of helper, test, and example crates for
  writing and compiling [Wasm component][wasm-component] modules using
  [wit-bindgen][wit-bindgen].

- [examples/*](./examples)

  `examples` contains examples and demos showcasing `homestar` packages
  and the `homestar runtime`. Each example is set up as its own crate,
  demonstrating the necessary dependencies and setup(s).

## Contributing

:balloon: We're thankful for any feedback and help in improving our project!
We have a focused [development](./DEVELOPMENT.md) guide, as well as a
more general [contributing](./CONTRIBUTING.md) guide to help you get involved.
We always adhere to our [Code of Conduct](./CODE_OF_CONDUCT.md).

## Releases

TBA

## Getting Help

For usage questions, usecases, or issues reach out to us in our [Discord channel](https://fission.codes/discord).

We would be happy to try to answer your question or try opening a new issue on Github.

## External Resources

- [What Is An IPVM][ipvm-wg]
- [IPVM: High-Level Spec][ipvm-spec]
- [Contributing Research][research]
- [Seamless Services for an Open World][seamless-services] by Brooklyn Zelenka
- [Foundations for Open-World Compute][foundations-for-openworld-compute] by Zeeshan Lakhani
- [IPVM: The Long-Fabled Execution Layer][cod-ipvm] by Brooklyn Zelenka
- [IPVM - IPFS and WASM][ipfs-thing-ipvm] by Brooklyn Zelenka
- [Breaking Down the Interplanetary Virtual Machine][blog-1]
- [Ucan Invocation Spec][ucan-invocation]
- [Wasm/Wit Demo - Februrary 2023][demo-1] by Zeeshan Lakhani

## License

This project is licensed under the [Apache License 2.0](./LICENSE), or
[http://www.apache.org/licenses/LICENSE-2.0][apache].

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[apache]: https://www.apache.org/licenses/LICENSE-2.0
[blog-1]: https://fission.codes/blog/ipfs-thing-breaking-down-ipvm/
[cod-ipvm]: https://www.youtube.com/watch?v=3y1RB8wt_YY
[demo-1]: https://www.loom.com/share/3204037368fe426ba3b4c952b0691c5c
[foundations-for-openworld-compute]: https://youtu.be/dRz5mau6fsY
[guest]: https://github.com/bytecodealliance/wit-bindgen#supported-guest-languages
[homebrew]: https://brew.sh/
[host-runtime]: https://github.com/bytecodealliance/wit-bindgen#host-runtimes-for-components
[ipfs-thing-ipvm]: https://www.youtube.com/watch?v=rzJWk1nlYvs
[ipld]: https://ipld.io/
[ipvm-spec]: https://github.com/ipvm-wg/spec
[ipvm-wg]: https://github.com/ipvm-wg
[ipvm-workflow-spec]: https://github.com/ipvm-wg/workflow
[mit]: http://opensource.org/licenses/MIT
[research]: https://github.com/ipvm-wg/research
[seamless-services]: https://youtu.be/Kr3B3sXh_VA
[ucan-invocation]: https://github.com/ucan-wg/invocation
[wasm-component]: https://github.com/WebAssembly/component-model
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
[wit-bindgen]: https://github.com/bytecodealliance/wit-bindgen
