# Developing homestar

## Outline

- [Building and Running the Project](#building-and-running-the-project)
- [Testing the Project](#testing-the-project)
- [Running the Runtime on Docker](#running-the-runtime-on-docker)
- [Nix](#nix)
- [Formatting](#formatting)
- [Pre-commit Hook](#pre-commit-hook)
- [Recommended Development Flow](#recommended-development-flow)
- [Conventional Commits](#conventional-commits)

## Building and Running the Project

- Building `homestar`:

  For the fastest compile-times and prettiest logs while developing `homestar`,
  build with:

  ``` console
  cargo build --no-default-features --features dev
  ```

  This removes underlying `wasmtime` `zstd` compression while also turning on
  ANSI color-coded logs. If you build with default features, `zstd` compression
  and other `wasmtime` defaults will be included in the build.

- Running the `homestar` server/runtime:

  ``` console
  cargo run --no-default-features --features dev -- start
  ```

- Running alongside [`tokio-console`][tokio-console] for diagnosis and debugging:

  ``` console
  cargo run --no-default-features --features dev,console -- start
  ```

  Then, in another window:

  ```console
  tokio-console --retain-for 60sec
  ```

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
  docker buildx build --file docker/Dockerfile --platform=linux/amd64,linux/arm64 -t homestar --progress=plain .
  ```

- Run a Docker image (depending on your platform):

  ```console
  docker run --platform=linux/arm64 -t homestar
  ```

## Nix
This repository contains a [Nix flake][nix-flake] that initiates both the Rust
toolchain set in [rust-toolchain.toml](./rust-toolchain.toml) and a
[pre-commit hook](#pre-commit-hook). It also installs
[external dependencies](#external-dependencies), as well as helpful cargo
binaries for development. Please install [nix][nix] and [direnv][direnv] to get
started.

Run `nix develop` or `direnv allow` to load the `devShell` flake output,
according to your preference.

## Formatting

For formatting Rust in particular, we automatically format on `nightly`, as it
uses specific nightly features we recommend by default.

## Pre-commit Hook

This project recommends using [pre-commit][pre-commit] for running pre-commit
hooks. Please run this before every commit and/or push.

- If you are doing interim commits locally, and you want to skip the pre-commit checks
  you can run
  `git commit -a -m "Your message here" --no-verify`.

## Recommended Development Flow

- We recommend leveraging [cargo-watch][cargo-watch],
  [cargo-expand][cargo-expand] and [irust][irust] for Rust development.
- We also recommend using [cargo-udeps][cargo-udeps] for removing unused
  dependencies before commits and pull-requests.
- If using our [Nix flake](./flake.nix), there are a number of handy
  command shortcuts available for working with `cargo-watch`, `diesel`, and
  other items, including:
  * **`ci`**, which runs a sequence of commands to check formatting, lints,
    release builds, and tests
  * **`db`** and **`db-reset`** for running `diesel` setup and migrations
  * **`doc`** for generating cargo docs with private-items documented
  * **`docker-<amd64,arm64>`** for running docker builds
  * **`nx-test`**, which translates to
    `cargo nextest run --workspace && cargo test --workspace --doc`
  * **`x-test`** for testing continuously as files change, translating to
    `cargo watch -c -s "cargo nextest run --workspace --no-capture && cargo test --doc"`
  * **`x-<build,check,run,clippy>`** for running a variety of `cargo watch`
    execution stages
  * **`nx-test-<all,0>`**, which is just like `nx-test`, but adds `all` or `0`
    for running tests either with the `all-features` flag or
    `no-default-features` flag, respectively
  * **`x-<build,check,run,clippy,test>-<core,wasm,runtime>`** for
    package-specific builds, tests, etc.

## Conventional Commits

This project *lightly* follows the [Conventional Commits convention][commit-spec-site]
to help explain commit history and tie in with our release process. The full specification
can be found [here][commit-spec]. We recommend prefixing your commits with
a type of `fix`, `feat`, `docs`, `ci`, `refactor`, etc..., structured like so:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

[buildx]: https://docs.docker.com/engine/reference/commandline/buildx/
[cargo-expand]: https://github.com/dtolnay/cargo-expand
[cargo-nextest]: https://nexte.st/index.html
[cargo-nextest-install]: https://nexte.st/book/installation.html
[cargo-udeps]: https://github.com/est31/cargo-udeps
[cargo-watch]: https://github.com/watchexec/cargo-watch
[commit-spec]: https://www.conventionalcommits.org/en/v1.0.0/#specification
[commit-spec-site]: https://www.conventionalcommits.org/
[docker-engine]: https://docs.docker.com/engine/
[irust]: https://github.com/sigmaSd/IRust
[direnv]:https://direnv.net/
[nix]:https://nixos.org/download.html
[nix-flake]: https://nixos.wiki/wiki/Flakes
[pre-commit]: https://pre-commit.com/
[tokio-console]: https://github.com/tokio-rs/console
[wit-bindgen]: https://github.com/bytecodealliance/wit-bindgen
