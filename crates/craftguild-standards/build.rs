//! Bootstrap is a separate, non-distributed executable, never an empty final SBOM.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if std::env::var_os("CARGO_FEATURE_DISTRIBUTION").is_some() {
        let package = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
        craftguild_cli_build::emit(
            package.parent().unwrap().parent().unwrap(),
            "craftguild-standards-engine",
        )?;
    }
    Ok(())
}
