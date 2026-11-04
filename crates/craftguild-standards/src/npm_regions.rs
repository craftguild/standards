//! TypeScript AST inspection prevents falsely counting mixed npm test source as implementation.

use craftguild_core::Result;
use std::path::Path;

pub fn verify(root: &Path, source: &Path) -> Result<()> {
    verify_with_resolution(
        root,
        source,
        source.parent().ok_or("Missing npm source parent")?,
    )
}

pub fn verify_with_resolution(root: &Path, source: &Path, resolution: &Path) -> Result<()> {
    let temp = tempfile::tempdir()?;
    let script = temp.path().join("npm-tests.cjs");
    std::fs::write(&script, include_bytes!("../../../adapters/npm-tests.cjs"))?;
    let data = crate::command::capture(
        root,
        &[
            "node".into(),
            script.to_str().ok_or("Non-UTF8 adapter path")?.into(),
            source.to_str().ok_or("Non-UTF8 source path")?.into(),
            resolution
                .to_str()
                .ok_or("Non-UTF8 parser resolution root")?
                .into(),
        ],
    )?;
    let result: serde_json::Value = serde_json::from_slice(&data)?;
    if result["mixed"] != false {
        return Err(
            "Mixed npm test/implementation regions cannot be reliably separated; unmeasured".into(),
        );
    }
    Ok(())
}
