fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Rebuild if i18n files change
    println!("cargo:rerun-if-changed=i18n");

    // Emit version information (if not cached by just vendor)
    vergen::EmitBuilder::builder()
        .git_sha(true)
        .git_commit_date()
        .emit()?;

    println!("cargo:rerun-if-env-changed=VERGEN_GIT_COMMIT_DATE");
    println!("cargo:rerun-if-env-changed=VERGEN_GIT_SHA");

    Ok(())
}
