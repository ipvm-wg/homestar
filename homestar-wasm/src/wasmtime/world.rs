#![allow(missing_docs)]

//! [Wasmtime] shim for parsing Wasm components, instantiating
//! a module, and executing a Wasm function dynamically.
//!
//! [Wasmtime]: <https://docs.rs/wasmtime/latest/wasmtime/>

use crate::{
    io::{Arg, Output},
    wasmtime::{
        ipld::{InterfaceType, RuntimeVal},
        limits::StoreLimitsAsync,
        Error,
    },
};
use heck::{ToKebabCase, ToSnakeCase};
use homestar_core::{
    bail,
    workflow::{error::ResolveError, input::Args, Input},
};
use std::{iter, time::Instant};
use wasmtime::{
    component::{self, Component, Func, Instance, Linker},
    Config, Engine, Store,
};
use wit_component::ComponentEncoder;

wasmtime::component::bindgen!({
    world: "imports",
    tracing: true,
    async: true
});

// One unit of fuel represents around 100k instructions.
const UNIT_OF_COMPUTE_INSTRUCTIONS: u64 = 100_000;

/// Incoming `state` from host runtime.
#[allow(missing_debug_implementations)]
pub struct State {
    /// Start time of the Wasm module.
    start_time: Instant,
    /// Fuel is a measure of how much computation a Wasm module is allowed to
    /// perform.
    fuel: u64,
    /// Limits are a set of limits that can be applied to a store, i.e. memory,
    /// table elements.
    limits: StoreLimitsAsync,
    /// Context for WASI modules.
    wasi_ctx: wasmtime_wasi::preview2::WasiCtx,
    /// WASI table.
    table: wasmtime_wasi::preview2::Table,
}

impl Default for State {
    fn default() -> Self {
        let table = wasmtime_wasi::preview2::Table::new();
        let wasi_ctx = wasmtime_wasi::preview2::WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();
        Self {
            start_time: Instant::now(),
            fuel: u64::MAX,
            limits: StoreLimitsAsync::default(),
            wasi_ctx,
            table,
        }
    }
}

/// [WasiView] implementation for [State] in order to support WASI component
/// modules.
///
/// [WasiView]: wasmtime_wasi::preview2::WasiView
impl wasmtime_wasi::preview2::WasiView for State {
    fn table(&self) -> &wasmtime_wasi::preview2::Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut wasmtime_wasi::preview2::Table {
        &mut self.table
    }

    fn ctx(&self) -> &wasmtime_wasi::preview2::WasiCtx {
        &self.wasi_ctx
    }

    fn ctx_mut(&mut self) -> &mut wasmtime_wasi::preview2::WasiCtx {
        &mut self.wasi_ctx
    }
}

impl State {
    /// Create a new [State] object.
    pub fn new(fuel: u64, limits: StoreLimitsAsync) -> Self {
        let table = wasmtime_wasi::preview2::Table::new();
        let wasi_ctx = wasmtime_wasi::preview2::WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
        Self {
            start_time: Instant::now(),
            fuel,
            limits,
            wasi_ctx,
            table,
        }
    }

    /// Set fuel.
    pub fn set_fuel(&mut self, fuel: u64) {
        self.fuel = fuel
    }

    /// Initial time from instantiation.
    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}

/// Runtime struct wrapping wasm/host bindings, the
/// wasmtime [Instance], [Engine], [Linker], and [Store].
#[allow(missing_debug_implementations)]
pub struct Env<T> {
    bindings: Option<World>,
    engine: Engine,
    instance: Option<Instance>,
    linker: Linker<T>,
    store: Store<T>,
}

impl<T> Env<T> {
    fn new(engine: Engine, linker: Linker<T>, store: Store<T>) -> Env<T> {
        Self {
            bindings: None,
            engine,
            instance: None,
            linker,
            store,
        }
    }

    fn set_bindings(&mut self, bindings: World) {
        self.bindings = Some(bindings);
    }

    fn set_instance(&mut self, instance: Instance) {
        self.instance = Some(instance);
    }

    /// Execute Wasm function dynamically given a list ([Args]) of [Ipld] or
    /// [wasmtime::component::Val] arguments and returning [Output] results.
    /// Types must conform to [Wit] IDL types when Wasm was compiled/generated.
    ///
    /// [Wit]: <https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md>
    /// [Ipld]: libipld::Ipld
    pub async fn execute(&mut self, args: Args<Arg>) -> Result<Output, Error>
    where
        T: Send,
    {
        let param_types = self
            .bindings
            .as_mut()
            .ok_or(Error::WasmInstantiation)?
            .func()
            .params(&self.store);
        let result_types = self
            .bindings
            .as_mut()
            .ok_or(Error::WasmInstantiation)?
            .func()
            .results(&self.store);

        let params: Vec<component::Val> = iter::zip(param_types.iter(), args.into_inner())
            .try_fold(vec![], |mut acc, (typ, arg)| {
                let v = match arg {
                    Input::Ipld(ipld) => {
                        RuntimeVal::try_from(ipld, &InterfaceType::from(typ))?.value()
                    }
                    Input::Arg(val) => match val.into_inner() {
                        Arg::Ipld(ipld) => {
                            RuntimeVal::try_from(ipld, &InterfaceType::from(typ))?.value()
                        }
                        Arg::Value(v) => v,
                    },
                    Input::Deferred(await_promise) => {
                        bail!(Error::ResolvePromise(ResolveError::UnresolvedCid(format!(
                            "deferred task/instruction not yet resolved or exists for promise: {}: {}",
                            await_promise.result(),
                            await_promise.instruction_cid()
                        ))))
                    }
                };
                acc.push(v);
                Ok::<_, Error>(acc)
            })?;

        let mut results_alloc: Vec<component::Val> = result_types
            .iter()
            .map(|_res| component::Val::Bool(false))
            .collect();

        self.bindings
            .as_mut()
            .ok_or(Error::WasmInstantiation)?
            .func()
            .call_async(&mut self.store, &params, &mut results_alloc)
            .await?;

        self.bindings
            .as_mut()
            .ok_or(Error::WasmInstantiation)?
            .func()
            .post_return_async(&mut self.store)
            .await?;

        let results = match &results_alloc[..] {
            [v] => Output::Value(v.to_owned()),
            [_v, ..] => Output::Values(results_alloc),
            [] => Output::Void,
        };

        Ok(results)
    }

    /// Return `wasmtime` bindings.
    pub fn bindings(&self) -> &Option<World> {
        &self.bindings
    }

    /// Return the initialized [wasmtime::component::Instance].
    pub fn instance(&self) -> Option<Instance> {
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
    /// Instantiate a default [environment] given a configuration
    /// for a [World], given [State].
    ///
    /// [environment]: Env
    pub fn default(data: State) -> Result<Env<State>, Error> {
        let config = Self::configure();
        let engine = Engine::new(&config)?;
        let mut linker = Self::define_linker(&engine);

        // Add WASI to the linker in order to support WASI modules.
        // This is a temporary measure until WASI is supported by default and is
        // unused otherwise.
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker)?;
        Imports::add_to_linker(&mut linker, |state: &mut State| state)?;

        let mut store = Store::new(&engine, data);
        store.set_fuel(store.data().fuel)?;

        // Configures a `Store` to yield execution of async WebAssembly code
        // periodically and not cause extended polling.
        store.fuel_async_yield_interval(Some(UNIT_OF_COMPUTE_INSTRUCTIONS))?;

        let env = Env::new(engine, linker, store);
        Ok(env)
    }

    /// Instantiates the provided `module` using the specified
    /// parameters, wrapping up the result in a [Env] structure
    /// that translates between wasm and the host, and gives access
    /// for future invocations to use the already-initialized linker, store.
    ///
    /// Used when first initiating a module of a workflow.
    pub async fn instantiate(
        bytes: Vec<u8>,
        fun_name: &str,
        data: State,
    ) -> Result<Env<State>, Error> {
        let config = Self::configure();
        let engine = Engine::new(&config)?;
        let mut linker = Self::define_linker(&engine);

        // Add WASI to the linker in order to support WASI modules.
        // This is a temporary measure until WASI is supported by default and is
        // unused otherwise.
        wasmtime_wasi::preview2::command::add_to_linker(&mut linker)?;
        Imports::add_to_linker(&mut linker, |state: &mut State| state)?;

        let mut store = Store::new(&engine, data);
        store.limiter_async(|s| &mut s.limits);
        store.set_fuel(store.data().fuel)?;

        // Configures a `Store` to yield execution of async WebAssembly code
        // periodically and not cause extended polling.
        store.fuel_async_yield_interval(Some(UNIT_OF_COMPUTE_INSTRUCTIONS))?;

        // engine clones are shallow (not deep).
        let component = component_from_bytes(&bytes, engine.clone())?;

        let (_bindings, instance) =
            Imports::instantiate_async(&mut store, &component, &linker).await?;

        let bindings = Self::new(&mut store, &instance, fun_name)?;

        //let bindings = Self::new(&mut store, &instance, fun_name)?;
        let mut env = Env::new(engine, linker, store);
        env.set_instance(instance);
        env.set_bindings(bindings);
        Ok(env)
    }

    /// Instantiates the provided `module` using the current
    /// [environment]'s engine, linker, and store, producing
    /// a new set of bindings for execution, and overriding
    /// the instance for the Wasm component.
    ///
    /// [environment]: Env
    pub async fn instantiate_with_current_env<'a, T>(
        bytes: Vec<u8>,
        fun_name: &'a str,
        env: &'a mut Env<T>,
    ) -> Result<&'a mut Env<T>, Error>
    where
        T: Send,
    {
        // engine clones are shallow (not deep).
        let component = component_from_bytes(&bytes, env.engine.clone())?;

        let (_bindings, instance) =
            Imports::instantiate_async(&mut env.store, &component, &env.linker).await?;

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
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);

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
        fun_name: &str,
    ) -> Result<Self, Error> {
        let mut store_ctx = store.as_context_mut();
        let mut exports = instance.exports(&mut store_ctx);
        let mut __exports = exports.root();
        let func = __exports
            .func(fun_name)
            .or_else(|| __exports.func(&fun_name.to_kebab_case()))
            .or_else(|| __exports.func(&fun_name.to_snake_case()))
            .ok_or_else(|| Error::WasmFunctionNotFound(fun_name.to_string()))?;

        Ok(World(func))
    }
}

/// Turn bytes into a Wasm [Component] module.
fn component_from_bytes(bytes: &[u8], engine: Engine) -> Result<Component, Error> {
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
                Component::from_binary(&engine, bytes).map_err(Error::IntoWasmComponent)
            } else {
                let component = ComponentEncoder::default()
                    .module(bytes)?
                    .validate(true)
                    .encode()?;
                Component::from_binary(&engine, &component).map_err(Error::IntoWasmComponent)
            }
        }
        Err(_) => {
            let wasm_bytes = wat::parse_bytes(bytes)?;
            if is_component(wasmparser::Parser::new(0).parse(&wasm_bytes, true)?) {
                Component::from_binary(&engine, &wasm_bytes).map_err(Error::IntoWasmComponent)
            } else {
                Err(Error::WatComponent(
                    "WAT must reference a Wasm component.".to_string(),
                ))
            }
        }
    }
}
