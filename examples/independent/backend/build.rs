//! Capture the explicit application root and verify its prepared distribution.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let package = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = package.parent().unwrap().to_path_buf();
    craftguild_cli_build::emit(&root, "backend-demo")?;
    Ok(())
}
