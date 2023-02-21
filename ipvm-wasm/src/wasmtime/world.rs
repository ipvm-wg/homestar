//! [Wasmtime] shim for parsing Wasm components, instantiating
//! a module, and executing a Wasm function dynamically.
//!
//! [Wasmtime]: https://docs.rs/wasmtime/latest/wasmtime/

use crate::wasmtime::ipld::{InterfaceType, RuntimeVal};
use anyhow::{anyhow, Result};
use heck::{ToKebabCase, ToSnakeCase};
use itertools::Itertools;
use libipld::Ipld;
use std::iter;
use wasmtime::{
    component::{self, Component, Func, Instance, Linker},
    Config, Engine, Store,
};
use wit_component::ComponentEncoder;

// One unit of fuel represents around 100k instructions.
const UNIT_OF_COMPUTE_INSTRUCTIONS: u64 = 100_000;

// TODO: Implement errors over thiserror and bubble up traps from here to
// our error set.

/// Incoming `state` from host runtime.
#[derive(Debug)]
pub struct State {
    fuel: u64,
}

impl Default for State {
    fn default() -> Self {
        Self { fuel: u64::MAX }
    }
}

impl State {
    /// Create a new [State] object.
    pub fn new(fuel: u64) -> Self {
        Self { fuel }
    }

    /// Set fuel add.
    pub fn add_fuel(&mut self, fuel: u64) {
        self.fuel = fuel
    }
}

/// Runtime struct wrapping wasm/host bindings, the
/// wasmtime [Instance], [Engine], [Linker], and [Store].
#[allow(missing_debug_implementations)]
pub struct Env<T> {
    bindings: World,
    engine: Engine,
    instance: Instance,
    linker: Linker<T>,
    store: Store<T>,
}

impl<T> Env<T> {
    fn new(
        bindings: World,
        engine: Engine,
        instance: Instance,
        linker: Linker<T>,
        store: Store<T>,
    ) -> Env<T> {
        Env {
            bindings,
            engine,
            instance,
            linker,
            store,
        }
    }

    fn set_bindings(&mut self, bindings: World) {
        self.bindings = bindings;
    }

    fn set_instance(&mut self, instance: Instance) {
        self.instance = instance;
    }

    /// Execute Wasm function dynamically given [Ipld] arguments
    /// and returning [Ipld] results. Types must conform to [Wit]
    /// IDL types when Wasm was compiled/generated.
    ///
    /// [Wit]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
    pub async fn execute(&mut self, args: Ipld) -> Result<Ipld>
    where
        T: Send,
    {
        use libipld::serde::from_ipld;
        let param_typs = self.bindings.func().params(&self.store);
        let result_typs = self.bindings.func().results(&self.store);
        let args = from_ipld::<Vec<Ipld>>(args)?;

        let params: Vec<component::Val> = iter::zip(param_typs.iter(), args.into_iter())
            .map(|(typ, arg)| RuntimeVal::try_from(arg, &InterfaceType::from(typ)))
            .fold_ok(vec![], |mut acc, v| {
                acc.push(v.into_inner());
                acc
            })?;

        let mut results_alloc: Vec<component::Val> = result_typs
            .iter()
            .map(|_res| component::Val::Bool(false))
            .collect();

        self.bindings
            .func()
            .call_async(&mut self.store, &params, &mut results_alloc)
            .await?;
        self.bindings
            .func()
            .post_return_async(&mut self.store)
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

    /// Return `wasmtime` bindings.
    pub fn bindings(&self) -> &World {
        &self.bindings
    }

    /// Return the initialized [wasmtime::component::Instance].
    pub fn instance(&self) -> Instance {
        self.instance
    }

    /// Return the initialized [wasmtime::component::Linker].
    pub fn linker(&self) -> &Linker<T> {
        &self.linker
    }

    /// Return a reference to the  initialized [wasmtime::Store].
    pub fn store(&self) -> &Store<T> {
        &self.store
    }
}

/// Shim for Wasmtime [Function] execution.
///
/// [Function]: Func
#[derive(Debug)]
pub struct World(Func);

impl World {
    /// Instantiates the provided `module` using the specified
    /// parameters, wrapping up the result in a [Env] structure
    /// that translates between wasm and the host, and gives access
    /// for future invocations to use the already-initialized linker, store.
    ///
    /// Used when first initiating a module of a workflow.
    pub async fn instantiate<'a>(
        bytes: Vec<u8>,
        fun_name: String,
        data: State,
    ) -> Result<Env<State>> {
        let config = Self::configure();
        let engine = Engine::new(&config)?;
        let linker = Self::define_linker(&engine);

        let mut store = Store::new(&engine, data);
        store.add_fuel(store.data().fuel)?;

        // Configures a `Store` to yield execution of async WebAssembly code
        // periodically and not cause extended polling.
        store.out_of_fuel_async_yield(u64::MAX, UNIT_OF_COMPUTE_INSTRUCTIONS);

        // engine clones are shallow (not deep).
        let component = component_from_bytes(&bytes, engine.clone())?;

        let instance = linker.instantiate_async(&mut store, &component).await?;
        let bindings = Self::new(&mut store, &instance, fun_name)?;
        let env = Env::new(bindings, engine, instance, linker, store);
        Ok(env)
    }

    /// Instantiates the provided `module` using the current
    /// [environment]'s engine, linker, and store, producing
    /// a new set of bindings for execution, and overriding
    /// the instance for the Wasm component.
    ///
    /// [environment]: Env
    pub async fn instantiate_with_current_env<T>(
        bytes: Vec<u8>,
        fun_name: String,
        env: &mut Env<T>,
    ) -> Result<&mut Env<T>>
    where
        T: Send,
    {
        // engine clones are shallow (not deep).
        let component = component_from_bytes(&bytes, env.engine.clone())?;

        let instance = env
            .linker
            .instantiate_async(&mut env.store, &component)
            .await?;
        let bindings = Self::new(&mut env.store, &instance, fun_name)?;
        env.set_instance(instance);
        env.set_bindings(bindings);
        Ok(env)
    }

    fn func(&self) -> Func {
        self.0
    }

    fn configure() -> Config {
        let mut config = Config::new();
        config.strategy(wasmtime::Strategy::Cranelift);
        config.wasm_component_model(true);
        config.async_support(true);
        config.cranelift_nan_canonicalization(true);

        // Most Wasm instructions consume 1 unit of fuel.
        // Some instructions, such as nop, drop, block, and loop, consume 0
        // units, as any execution cost associated with them involves other
        // instructions which do consume fuel. We use *these* defaults for now
        // for Ops, instead of parsing each Op.
        config.consume_fuel(true);

        config
    }

    fn define_linker<U>(engine: &Engine) -> Linker<U> {
        Linker::<U>::new(engine)
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
        let mut store_ctx = store.as_context_mut();
        let mut exports = instance.exports(&mut store_ctx);
        let mut __exports = exports.root();
        let func = __exports
            .func(&fun_name)
            .or_else(|| __exports.func(&fun_name.to_kebab_case()))
            .or_else(|| __exports.func(&fun_name.to_snake_case()))
            .ok_or_else(|| anyhow!("function not found"))?;

        Ok(World(func))
    }
}

/// Turn bytes into a Wasm [Component] module.
fn component_from_bytes(bytes: &[u8], engine: Engine) -> Result<Component> {
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
