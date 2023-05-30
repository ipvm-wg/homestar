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
    <a href="https://codecov.io/gh/ipvm-wg/homestar">
      <img src="https://codecov.io/gh/ipvm-wg/homestar/branch/main/graph/badge.svg?token=SOMETOKEN" alt="Code Coverage"/>
    </a>
    <a href="https://github.com/ipvm-wg/homestar/actions?query=">
      <img src="https://github.com/ipvm-wg/homestar/actions/workflows/tests_and_checks.yml/badge.svg" alt="Build Status">
    </a>
    <a href="https://github.com/ipvm-wg/homestar/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/homestar-core">
      <img src="https://img.shields.io/static/v1?label=Docs&message=core.docs.rs&color=blue" alt="Docs">
    </a>
    <a href="https://docs.rs/homestar-wasm">
      <img src="https://img.shields.io/static/v1?label=Docs&message=wasm.docs.rs&color=blue" alt="Docs">
    </a>
    <a href="https://discord.gg/fissioncodes">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

<div align="center"><sub>:warning: Work in progress :warning:</sub></div>

##

## Outline

- [Testing the Project](#testing-the-project)
- [Contributing](#contributing)
- [Getting Help](#getting-help)
- [External Resources](#external-resources)
- [License](#license)


## Testing the Project

- Run tests

  ```console
  cargo test --all-features
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
- We recommend using [cargo-udeps][cargo-udeps] for removing unused dependencies
  before commits and pull-requests.

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
[cargo-udeps]: https://github.com/est31/cargo-udeps
[cargo-watch]: https://github.com/watchexec/cargo-watch
[cod-ipvm]: https://www.youtube.com/watch?v=3y1RB8wt_YY
[commit-spec]: https://www.conventionalcommits.org/en/v1.0.0/#specification
[commit-spec-site]: https://www.conventionalcommits.org/
[demo-1]: https://www.loom.com/share/3204037368fe426ba3b4c952b0691c5c
[direnv]:https://direnv.net/
[ipfs-thing-ipvm]: https://www.youtube.com/watch?v=rzJWk1nlYvs
[ipvm-spec]: https://github.com/ipvm-wg/spec
[ipvm-wg]: https://github.com/ipvm-wg
[irust]: https://github.com/sigmaSd/IRust
[mit]: http://opensource.org/licenses/MIT
[nix]:https://nixos.org/download.html
[nix-flake]: https://nixos.wiki/wiki/Flakes
[pre-commit]: https://pre-commit.com/
[sqlite]: https://sqlite.org/index.html
[ucan-invocation]: https://github.com/ucan-wg/invocation
