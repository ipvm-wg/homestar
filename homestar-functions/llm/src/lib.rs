#![cfg(target_arch = "wasm32")]

#[allow(clippy::all, dead_code)]
#[rustfmt::skip]
mod bindings;

use bindings::{
    homestar::host::chain::{prompt_chain, prompt_seq, prompt_with},
    wasi::logging::logging::{log, Level},
    Guest,
};
use std::path::PathBuf;

struct Component;

impl Guest for Component {
    fn gen(topic: String) -> String {
        log(Level::Info, "topic", &topic);
        let input = format!(
            "{} is an awesome topic. Let's write about it in a few sentences.",
            topic
        );
        prompt_with(
            &input,
            Some(
                PathBuf::from("./models/Meta-Llama-3-8B-Instruct.Q4_0.gguf")
                    .display()
                    .to_string()
                    .as_str(),
            ),
        )
    }

    fn gen_by_section(input: String, topic: String) -> String {
        log(Level::Info, "input", &input);
        log(Level::Info, "topic", &topic);

        let system = "You are a journalist writing about cities.";

        let step = format!(
            "Given an article already containing {}, copy {} again, and then write more about the topic {} in a new paragraph: \n{{text}}",
            input,
            input,
            topic
        );
        prompt_seq(system, &input, &step, None)
    }

    fn gen_map_reduce(input: String) -> String {
        log(Level::Info, "input", &input);
        let system = "You are a journalist writing about cities.";

        let map_step = "Given an article already on a topic, add a new fact about the city, tell us what that fact is, and write a sentence about it:\n{{text}}";
        let reduce_step = "Summarize facts of the city into a well-written paragraph made up of at least 4 sentences:\n{{text}}";
        prompt_chain(&system, &input, map_step, reduce_step, None)
    }
}

bindings::export!(Component with_types_in bindings);
