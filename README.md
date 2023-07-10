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
    <a href="https://discord.gg/fissioncodes">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

<div align="center"><sub>:warning: Work in progress :warning:</sub></div>

##

## Outline

- [Workspace](#workspace)
- [Testing the Project](#testing-the-project)
- [Running the Runtime on Docker](#running-the-runtime-on-docker)
- [Contributing](#contributing)
- [Getting Help](#getting-help)
- [External Resources](#external-resources)
- [License](#license)

## Workspace

This repository is comprised of a few library packages and a binary that
represents the `homestar` runtime.

### Core Crates

- [homestar-core](./homestar-core)

  The *core* library implements much of the [Ucan Invocation][ucan-invocation]
  and [Ipvm Workflow][ipvm-workflow-spec] specifications and is used as the
  foundation for other packages in this `workspace` and within the runtime engine.

- [homestar-wasm](./homestar-wasm)

  This *wasm* library manages the [wasmtime][wasmtime] runtime, provides the
  [Ipld][ipld] to/from [Wit][wit] interpreter/translation-layer, and implements
  the input interface for working with Ipvm's standard Wasm tasks.

### Runtime Crate

- [homestar-runtime](./homestar-runtime)

  The *runtime* is responsible for bootstrapping and running nodes, scheduling
  and executing workflows as well as tasks within workflows, handling retries
  and failure modes, etc.

### Non-published, Helper Crates

- [homestar-functions](./homestar-functions)

  Currently, this is a helper and example crate for writing and compiling
  [Wasm components][wasm-component] using [wit-bindgen][wit-bindgen].

  **It will be expanded upon as a default set of Wasm mods and functions.**

## Testing the Project

- Running the tests:

We recommend using [cargo nextest][cargo-nextest], which is installed by default
in our [Nix flake](#nix) or can be [installed separately][cargo-nextest-install].

  ```console
  cargo nextest run --all-features --no-capture
  ```

The above command translates to this using the default `cargo test`:

  ```console
  cargo test --all-features -- --nocapture
  ```

## Running the Runtime on Docker

We recommend setting your [Docker Engine][docker-engine] configuration
with `experimental` and `buildkit` set to `true`, for example:

``` json
{
  "builder": {
    "gc": {
      "defaultKeepStorage": "20GB",
      "enabled": true
    }
  },
  "experimental": true,
  "features": {
    "buildkit": true
  }
}
```

- Build a multi-plaform Docker image via [buildx][buildx]:

  ```console
  docker buildx build --file docker/Dockerfile --platform=linux/amd64,linux/arm64 -t homestar-runtime --progress=plain .
  ```

- Run a Docker image (depending on your platform):

  ```console
  docker run --platform=linux/arm64 -t homestar-runtime
  ```

## Contributing

:balloon: We're thankful for any feedback and help in improving our project!
We have a [contributing guide](./CONTRIBUTING.md) to help you get involved. We
also adhere to our [Code of Conduct](./CODE_OF_CONDUCT.md).

### Nix
This repository contains a [Nix flake][nix-flake] that initiates both the Rust
toolchain set in [rust-toolchain.toml](./rust-toolchain.toml) and a
[pre-commit hook](#pre-commit-hook). It also installs
[external dependencies](#external-dependencies), as well as helpful cargo
binaries for development. Please install [nix][nix] and [direnv][direnv] to get
started.

Run `nix develop` or `direnv allow` to load the `devShell` flake output,
according to your preference.

### Formatting

For formatting Rust in particular, we automatically format on `nightly`, as it
uses specific nightly features we recommend by default.

### Pre-commit Hook

This project recommends using [pre-commit][pre-commit] for running pre-commit
hooks. Please run this before every commit and/or push.

- If you are doing interim commits locally, and for some reason if you _don't_
  want pre-commit hooks to fire, you can run
  `git commit -a -m "Your message here" --no-verify`.

### Recommended Development Flow

- We recommend leveraging [cargo-watch][cargo-watch],
  [cargo-expand][cargo-expand] and [irust][irust] for Rust development.
- We also recommend using [cargo-udeps][cargo-udeps] for removing unused
  dependencies before commits and pull-requests.
- If using our [Nix flake](./flake.nix), there are a number of handy
  command shortcuts available for working with `cargo-watch`, `diesel`, and
  other items, including:
  * **`ci`**, which runs a sequence of commands to check formatting, lints, release
    builds, and tests
  * **`db`** and **`db-reset`** for running `diesel` setup and migrations
  * **`doc`** for generating cargo docs with private-items documented
  * **`compile-wasm`** for compiling [homestar-functions](./homestar-functions),
    a [wit-bindgen][wit-bindgen]-driven example, to the `wasm32-unknown-unknown` target
  * **`docker-<amd64,arm64>`** for running docker builds
  * **`nx-test`**, which translates to `cargo nextest run && cargo test --doc`
  * **`x-test`** for testing continuously as files change, translating to
    `cargo watch -c -s "cargo nextest run && cargo test --doc"`
  * **`x-<build,check,run,clippy>`** for running a variety of `cargo watch`
    execution stages
  * **`nx-test-<all,0>`**, which is just like `nx-test`, but adds `all` or `0`
    for running tests either with the `all-features` flag or
    `no-default-features` flag, respectively
  * **`x-<build,check,run,clippy,test>-<core,wasm,runtime>`** for package-specific
    builds, tests, etc.

### Conventional Commits

This project *lightly* follows the [Conventional Commits
convention][commit-spec-site] to help explain
commit history and tie in with our release process. The full specification
can be found [here][commit-spec]. We recommend prefixing your commits with
a type of `fix`, `feat`, `docs`, `ci`, `refactor`, etc..., structured like so:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Getting Help

For usage questions, usecases, or issues reach out to us in our [Discord channel](https://discord.gg/fissioncodes).

We would be happy to try to answer your question or try opening a new issue on Github.

## External Resources

- [What Is An IPVM][ipvm-wg]
- [IPVM: High-Level Spec][ipvm-spec]
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
[cargo-expand]: https://github.com/dtolnay/cargo-expand
[cargo-nextest]: https://nexte.st/index.html
[cargo-nextest-install]: https://nexte.st/book/installation.html
[cargo-udeps]: https://github.com/est31/cargo-udeps
[cargo-watch]: https://github.com/watchexec/cargo-watch
[cod-ipvm]: https://www.youtube.com/watch?v=3y1RB8wt_YY
[commit-spec]: https://www.conventionalcommits.org/en/v1.0.0/#specification
[commit-spec-site]: https://www.conventionalcommits.org/
[demo-1]: https://www.loom.com/share/3204037368fe426ba3b4c952b0691c5c
[direnv]:https://direnv.net/
[foundations-for-openworld-compute]: https://youtu.be/dRz5mau6fsY
[ipfs-thing-ipvm]: https://www.youtube.com/watch?v=rzJWk1nlYvs
[ipld]: https://ipld.io/
[ipvm-spec]: https://github.com/ipvm-wg/spec
[ipvm-wg]: https://github.com/ipvm-wg
[ipvm-workflow-spec]: https://github.com/ipvm-wg/workflow
[irust]: https://github.com/sigmaSd/IRust
[mit]: http://opensource.org/licenses/MIT
[nix]:https://nixos.org/download.html
[nix-flake]: https://nixos.wiki/wiki/Flakes
[pre-commit]: https://pre-commit.com/
[seamless-services]: https://youtu.be/Kr3B3sXh_VA
[ucan-invocation]: https://github.com/ucan-wg/invocation
[wasm-component]: https://github.com/WebAssembly/component-model
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
[wit-bindgen]: https://github.com/bytecodealliance/wit-bindgen
