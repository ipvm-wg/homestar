use super::FileLoad;
use async_trait::async_trait;
use homestar_core::workflow::input::Args;
use homestar_wasm::{
    io::{Arg, Output},
    wasmtime::{world::Env, Error as WasmRuntimeError, State, World},
};

#[allow(missing_debug_implementations)]
pub(crate) struct WasmContext {
    env: Env<State>,
}

impl WasmContext {
    pub(crate) fn new(data: State) -> Result<Self, WasmRuntimeError> {
        Ok(Self {
            env: World::default(data)?,
        })
    }

    /// Instantiate environment via [World] and execute on [Args].
    pub(crate) async fn run<'a>(
        &mut self,
        bytes: Vec<u8>,
        fun_name: &'a str,
        args: Args<Arg>,
    ) -> Result<Output, WasmRuntimeError> {
        let env = World::instantiate_with_current_env(bytes, fun_name, &mut self.env).await?;
        env.execute(args).await
    }
}

#[async_trait]
impl FileLoad for WasmContext {}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    fn fixtures(file: &str) -> PathBuf {
        PathBuf::from(format!(
            "{}/../homestar-wasm/fixtures/{file}",
            env!("CARGO_MANIFEST_DIR")
        ))
    }

    #[tokio::test]
    async fn load_wasm_file_as_bytes() {
        let wat = WasmContext::load(fixtures("add_one_component.wat"))
            .await
            .unwrap();

        assert!(!wat.is_empty());
    }
}
