//! Capture the explicit application root and verify its prepared distribution.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let package = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = package;
    craftguild_cli_build::emit(&root, "sample")?;
    Ok(())
}
