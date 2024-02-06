fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    vergen::EmitBuilder::builder()
        .fail_on_error()
        .use_local_build()
        .git_sha(true)
        .cargo_features()
        .emit()?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
