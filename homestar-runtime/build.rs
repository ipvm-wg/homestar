fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    vergen::EmitBuilder::builder()
        .fail_on_error()
        .git_sha(true)
        // TODO: Look into why this returns old date/times under nix.
        //.git_commit_timestamp()
        .cargo_features()
        .emit()?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
