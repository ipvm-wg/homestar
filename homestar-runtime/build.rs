fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    vergen::EmitBuilder::builder()
        .git_sha(true)
        .git_commit_timestamp()
        .cargo_features()
        .emit()?;

    #[cfg(feature = "llm")]
    println!("cargo::rustc-env=LLAMA_METAL=1");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
