fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    vergen::EmitBuilder::builder()
        .git_sha(true)
        .git_commit_timestamp()
        .cargo_features()
        .emit()?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
