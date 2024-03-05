<div align="center">
  <a href="https://github.com/ipvm-wg/homestar" target="_blank">
    <img src="https://raw.githubusercontent.com/ipvm-wg/homestar/main/assets/mascot_full_transparent.png" alt="Homestar logo" width="400"></img>
  </a>

  <h1 align="center">Homestar Functions</h1>

  <p>
    <a href="https://github.com/ipvm-wg/homestar/blob/main/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://discord.gg/fissioncodes">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

##

This is a template-like, example-driven set of non-published crates used for
building Wasm components in order to run and test them on the latest
[wasmtime][wasmtime] runtime, with the `component-model` feature turned on.

We use the components compiled from this crate as fixtures for our
execution-and-[IPLD][ipld]-focused [homestar-wasm crate](../homestar-wasm). We
currently rely on the [WIT format][wit-mvp] IDL to describe exports, for
example:

```wit
default world homestar {
  export add-one: func(a: s32) -> s32
  export append-string: func(a: string) -> string
  export transpose: func(matrix: list<list<u8>>) -> list<list<u8>>
}
```

We then implement these functions in [lib.rs](./src/lib.rs) using
[wit-bindgen][wit-bindgen]/[wit-bindgen-rt][wit-bindgen-rt], a guest language
bindings generator for [WIT][wit-mvp] and the
[Component Model][component-model].

## Build

Once functions are implemented, we can use [cargo-component][cargo-component] to
generate the necessary bindings and build the component in release-mode,
targeting [`wasm32-unknown-unknown`][wasm32-unknown]:

```console
# from this directory:
cd test && cargo component build --target wasm32-unknown-unknown --profile release-wasm-fn

# or from the top-level workspace:
cargo component build -p homestar-functions-test --target wasm32-unknown-unknown --profile release-wasm-fn
```

We can also use the [cargo-component][cargo-component] default [`wasm32-wasi`][wasm32-wasi] target:

``` console
cargo component build -p homestar-functions-test --profile release-wasm-fn
```

Guest Wasm modules will be generated in the top-level `homestar` directory:
`./target/wasm32-unknown-unknown/release-wasm-fn/homestar_functions_test.wasm`
or `./target/wasm32-wasi/release-wasm-fn/homestar_functions_test.wasm`.

### Other Helpful Repos

* [keyvalue-component-model-demo][kv-demo]
* [SpiderLightning][spiderlightning] - defines a set of `*.wit` files that
  abstract distributed application capabilities, such as key-value, messaging,
  http-server/client and more.

## License

This project is licensed under the [Apache License 2.0](./LICENSE), or
[http://www.apache.org/licenses/LICENSE-2.0][apache].

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.


[apache]: https://www.apache.org/licenses/LICENSE-2.0
[cargo-component]: https://github.com/bytecodealliance/cargo-component
[component-model]: https://github.com/WebAssembly/component-model
[ipld]: https://ipld.io/
[kv-demo]: https://github.com/Mossaka/keyvalue-component-model-demo
[spiderlightning]: https://github.com/deislabs/spiderlightning
[wasi]: https://github.com/WebAssembly/WASI
[wasm32-unknown]: https://rustwasm.github.io/docs/wasm-pack/prerequisites/non-rustup-setups.html#manually-add-wasm32-unknown-unknown
[wasm32-wasi]: https://wasmbyexample.dev/examples/wasi-hello-world/wasi-hello-world.rust.en-us
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[wasm-tools]: https://github.com/bytecodealliance/wasm-tools
[wit-bindgen]: https://github.com/bytecodealliance/wit-bindgen
[wit-bindgen-rt]: https://github.com/bytecodealliance/wit-bindgen-rt
[wit-component]: https://crates.io/crates/wit-component
[wit-mvp]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
