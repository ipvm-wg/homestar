//! Helper functions that can be used in guest Wasm components.

#[cfg(feature = "llm")]
use crate::wasmtime::world::homestar::host::chain;
use crate::wasmtime::{
    world::{homestar::host::helpers, wasi},
    State,
};
use async_trait::async_trait;
#[cfg(feature = "llm")]
use llm_chain::{
    chains::map_reduce::Chain, executor, options, parameters, prompt, step::Step, Parameters,
};
#[cfg(feature = "llm")]
use std::path::PathBuf;
use std::time::Instant;
use tracing::instrument;

#[async_trait]
impl helpers::Host for State {
    /// Get the current time.
    async fn get_current_time(&mut self) -> wasmtime::Result<helpers::Time> {
        let now = Instant::now();
        let duration = now.duration_since(self.start_time());
        Ok(helpers::Time {
            seconds: duration.as_secs(),
            milliseconds: duration.subsec_millis(),
            nanoseconds: duration.subsec_nanos(),
        })
    }

    /// Print a message.
    async fn print(&mut self, from_wasm: String) -> wasmtime::Result<()> {
        println!("{from_wasm}");
        Ok(())
    }
}

#[cfg(feature = "llm")]
#[async_trait]
impl chain::Host for State {
    async fn prompt_with(
        &mut self,
        input: String,
        model: Option<String>,
    ) -> wasmtime::Result<String> {
        let opts = options!(
            Model: options::ModelRef::from_path(model.unwrap_or(PathBuf::from("./models/Meta-Llama-3-8B-Instruct.Q4_0.gguf").display().to_string())),
            ModelType: "llama",
            MaxContextSize: 4096_usize,
            NThreads: 4_usize,
            MaxTokens: 2048_usize,
            MaxBatchSize: 4096_usize,
            TopK: 40_i32,
            TopP: 0.95,
            TfsZ: 1.0,
            TypicalP: 1.0,
            Temperature: 0.8,
            RepeatPenalty: 1.1,
            RepeatPenaltyLastN: 64_usize,
            FrequencyPenalty: 0.0,
            PresencePenalty: 0.0,
            Mirostat: 0_i32,
            MirostatTau: 5.0,
            MirostatEta: 0.1,
            PenalizeNl: true,
            StopSequence: vec!["\n\n".to_string()]
        );

        let exec = executor!(llama, opts.clone())?;
        let res = prompt!(input).run(&parameters!(), &exec).await?;
        match res.to_immediate().await {
            Ok(res) => Ok(res.to_string()),
            Err(e) => Err(e.into()),
        }
    }

    async fn prompt_seq(
        &mut self,
        system: String,
        input: String,
        next: String,
        model: Option<String>,
    ) -> wasmtime::Result<String> {
        let opts = options!(
            Model: options::ModelRef::from_path(model.unwrap_or(PathBuf::from("./models/Meta-Llama-3-8B-Instruct.Q4_0.gguf").display().to_string())),
            ModelType: "llama",
            MaxContextSize: 4096_usize,
            NThreads: 8_usize,
            MaxTokens: 2048_usize,
            MaxBatchSize: 4096_usize,
            TopK: 40_i32,
            TopP: 0.95,
            TfsZ: 1.0,
            TypicalP: 1.0,
            Temperature: 0.8,
            RepeatPenalty: 1.1,
            RepeatPenaltyLastN: 64_usize,
            FrequencyPenalty: 0.0,
            PresencePenalty: 0.0,
            Mirostat: 0_i32,
            MirostatTau: 5.0,
            MirostatEta: 0.1,
            PenalizeNl: true,
            StopSequence: vec!["\n\n".to_string()]
        );

        let exec = executor!(llama, opts.clone())?;
        let chain = Step::for_prompt_template(prompt!(&system, &next)).to_chain();
        let parameters = parameters!("text" => input);
        let res = chain.run(parameters, &exec).await?;
        match res.to_immediate().await {
            Ok(res) => Ok(res.get_content().to_string()),
            Err(e) => Err(e.into()),
        }
    }

    async fn prompt_chain(
        &mut self,
        system: String,
        input: String,
        map: String,
        reduce: String,
        model: Option<String>,
    ) -> wasmtime::Result<String> {
        let opts = options!(
            Model: options::ModelRef::from_path(model.unwrap_or(PathBuf::from("./models/Meta-Llama-3-8B-Instruct.Q4_0.gguf").display().to_string())),
            ModelType: "llama",
            MaxContextSize: 4096_usize,
            NThreads: 4_usize,
            MaxTokens: 2048_usize,
            MaxBatchSize: 4096_usize,
            TopK: 40_i32,
            TopP: 0.95,
            TfsZ: 1.0,
            TypicalP: 1.0,
            Temperature: 0.8,
            RepeatPenalty: 1.1,
            RepeatPenaltyLastN: 64_usize,
            FrequencyPenalty: 0.0,
            PresencePenalty: 0.0,
            Mirostat: 0_i32,
            MirostatTau: 5.0,
            MirostatEta: 0.1,
            PenalizeNl: true,
            StopSequence: vec!["\n\n".to_string()]
        );

        let exec = executor!(llama, opts.clone())?;
        let map_prompt = Step::for_prompt_template(prompt!(&system, &map));
        let reduce_prompt = Step::for_prompt_template(prompt!(&system, &reduce));
        let chain = Chain::new(map_prompt, reduce_prompt);
        let docs = vec![Parameters::new_with_text(input)];
        let res = chain.run(docs, Parameters::new(), &exec).await?;
        match res.to_immediate().await {
            Ok(res) => Ok(res.get_content().to_string()),
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait]
impl wasi::logging::logging::Host for State {
    /// Log a message, formatted by the runtime subscriber.
    #[instrument(name = "wasi_log", skip_all)]
    async fn log(
        &mut self,
        level: wasi::logging::logging::Level,
        context: String,
        message: String,
    ) -> wasmtime::Result<()> {
        match level {
            wasi::logging::logging::Level::Trace => {
                tracing::trace!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Debug => {
                tracing::debug!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Info => {
                tracing::info!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Warn => {
                tracing::warn!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Error => {
                tracing::error!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
            wasi::logging::logging::Level::Critical => {
                tracing::error!(
                    subject = "wasm_execution",
                    category = context.as_str(),
                    "{message}"
                )
            }
        }
        Ok(())
    }
}
