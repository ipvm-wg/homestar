//! [Wasmtime] shim for parsing Wasm components, instantiating
//! the runtime, and executing Wasm functions on the runtime dynamically.
//!
//! [Wasmtime]: https://docs.rs/wasmtime/latest/wasmtime/

use crate::wasm::wasmtime::ipld::{InterfaceType, RuntimeVal};
use anyhow::{anyhow, Result};
use heck::{ToKebabCase, ToSnakeCase};
use itertools::Itertools;
use libipld::Ipld;
use std::iter;
use wasmtime::{
    component::{self, Component, Func, Instance, Linker},
    Engine,
};
use wit_component::ComponentEncoder;

/// Turn bytes into a Wasm [Component] module.
pub fn component_from_bytes(bytes: &[u8], engine: Engine) -> Result<Component> {
    fn is_component(chunk: wasmparser::Chunk<'_>) -> bool {
        matches!(
            chunk,
            wasmparser::Chunk::Parsed {
                payload: wasmparser::Payload::Version {
                    encoding: wasmparser::Encoding::Component,
                    ..
                },
                ..
            }
        )
    }
    match wasmparser::Parser::new(0).parse(bytes, true) {
        Ok(chunk) => {
            if is_component(chunk) {
                Component::from_binary(&engine, bytes)
            } else {
                let component = ComponentEncoder::default()
                    .module(bytes)?
                    .validate(true)
                    .encode()?;
                Component::from_binary(&engine, &component)
            }
        }
        Err(_) => {
            let wasm_bytes = wat::parse_bytes(bytes)?;
            if is_component(wasmparser::Parser::new(0).parse(&wasm_bytes, true)?) {
                Component::from_binary(&engine, &wasm_bytes)
            } else {
                Err(anyhow!("WAT must reference a Wasm component."))
            }
        }
    }
}

/// Shim for Wasmtime [Function] execution.
///
/// [Function]: Func
#[derive(Debug)]
pub struct Wasmtime(Func);

impl Wasmtime {
    /// Instantiates the provided `module` using the specified
    /// parameters, wrapping up the result in a structure that
    /// translates between wasm and the host.
    pub async fn instantiate<T>(
        mut store: impl wasmtime::AsContextMut<Data = T>,
        component: &Component,
        linker: &Linker<T>,
        fun_name: String,
    ) -> Result<(Self, Instance)>
    where
        T: Send,
    {
        let instance = linker.instantiate_async(&mut store, component).await?;
        Ok((Self::new(store, &instance, fun_name)?, instance))
    }

    /// Low-level creation wrapper for wrapping up the exports
    /// of the `instance` provided in this structure of wasm
    /// exports.
    ///
    /// This function will extract exports from the `instance`
    /// defined within `store` and wrap them all up in the
    /// returned structure which can be used to interact with
    /// the wasm module.
    fn new(
        mut store: impl wasmtime::AsContextMut,
        instance: &Instance,
        fun_name: String,
    ) -> Result<Self> {
        let mut store = store.as_context_mut();
        let mut exports = instance.exports(&mut store);
        let mut __exports = exports.root();
        let func = __exports
            .func(&fun_name)
            .or_else(|| __exports.func(&fun_name.to_kebab_case()))
            .or_else(|| __exports.func(&fun_name.to_snake_case()))
            .ok_or_else(|| anyhow!("function not found"))?;

        Ok(Wasmtime(func))
    }

    /// Execute Wasm function dynamically given [Ipld] arguments
    /// and returning [Ipld] results. Types must conform to [Wit]
    /// IDL types when Wasm was compiled/generated.
    ///
    /// [Wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
    pub async fn execute<T>(
        &self,
        mut store: impl wasmtime::AsContextMut<Data = T>,
        args: Vec<Ipld>,
    ) -> Result<Ipld>
    where
        T: Send,
    {
        let param_typs = self.func().params(&store);
        let result_typs = self.func().results(&store);

        let params: Vec<component::Val> = iter::zip(param_typs.iter(), args.into_iter())
            .into_iter()
            .map(|(typ, arg)| RuntimeVal::try_from(arg, InterfaceType::from(typ)))
            .fold_ok(vec![], |mut acc, v| {
                acc.push(v.into_inner());
                acc
            })?;

        let mut results_alloc: Vec<component::Val> = result_typs
            .iter()
            .map(|_res| component::Val::Bool(false))
            .collect();

        self.func()
            .call_async(store.as_context_mut(), &params, &mut results_alloc)
            .await?;
        self.func()
            .post_return_async(store.as_context_mut())
            .await?;

        let results: Vec<Ipld> = results_alloc
            .into_iter()
            .map(|v| Ipld::try_from(RuntimeVal(v)))
            .fold_ok(vec![], |mut acc, v| {
                acc.push(v);
                acc
            })?;

        Ok(Ipld::from(results))
    }

    fn func(&self) -> Func {
        self.0
    }
}
