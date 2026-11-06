//! Build-script entry points. This crate never invokes Cargo recursively.

#![forbid(unsafe_code)]

pub mod observe;
mod process;

use craftguild_core::{seal::Prepared, BuildInfo, Dirty, Result};
use std::path::Path;

/// Capture this application's actual Cargo build, verify preparation, and embed it.
///
/// `root` is the application's explicit repository/workspace root. Include the
/// generated `craftguild.rs` from OUT_DIR in the application binary. The default
/// record is `.craftguild/<command>/prepared.json`; CRAFTGUILD_PREPARED may select
/// another record below the same source root.
pub fn emit(root: &Path, command: &str) -> Result<BuildInfo> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("Missing OUT_DIR")?);
    // Cargo does not watch newly created untracked files or every Git transition.
    // A deliberately absent input re-observes identity at each normal invocation.
    println!(
        "cargo:rerun-if-changed={}",
        out.join("recapture-each-build").display()
    );
    for name in ["CRAFTGUILD_PREPARED", "CRAFTGUILD_COMMAND", "RUSTC"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let info = observe::capture(
        root,
        command,
        &std::env::var("CARGO_PKG_VERSION")?,
        &std::env::var("TARGET")?,
        &std::env::var("PROFILE")?,
    );
    let selected = std::env::var("CRAFTGUILD_COMMAND").ok();
    let record = if selected
        .as_deref()
        .is_none_or(|selected| selected == command)
    {
        std::env::var("CRAFTGUILD_PREPARED").ok()
    } else {
        None
    }
    .unwrap_or_else(|| format!(".craftguild/{command}/prepared.json"));
    let result = (|| -> Result<Vec<u8>> {
        let prepared: Prepared = serde_json::from_slice(&std::fs::read(
            craftguild_core::paths::owned(root, &record)?,
        )?)?;
        let mut features: Vec<_> = prepared
            .features
            .iter()
            .map(|f| normalize_feature(f))
            .collect();
        features.sort();
        let mut actual: Vec<_> = std::env::vars()
            .filter_map(|(name, _)| name.strip_prefix("CARGO_FEATURE_").map(str::to_owned))
            .collect();
        actual.sort();
        if features != actual {
            return Err("Prepared features differ from Cargo's effective root features".into());
        }
        prepared.verify(root, &info, &prepared.features)
    })();
    let bytes = result.map_err(|e| {
        format!("Craftguild preparation failed: {e}. Run `make build` (or craftguild-standards-engine build) first.")
    })?;
    write_changed(&out.join("craftguild-sbom.json"), &bytes)?;
    let dirty = match info.dirty {
        Dirty::Clean => "Some(false)",
        Dirty::Dirty => "Some(true)",
        Dirty::Unknown => "None",
    };
    let source = format!(
        "pub const BUILD_INFO: craftguild_cli::BuildInfo<'static> = craftguild_cli::BuildInfo {{\n\
         command: {:?}, version: {:?}, commit: {:?}, dirty: {dirty},\n\
         compiler: {:?}, target: {:?}, profile: {:?},\n}};\n\
         pub const BUILD_INFO_JSON: &str = {:?};\n\
         pub const SBOM: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/craftguild-sbom.json\"));\n",
        info.command, info.version, info.commit, info.compiler, info.target, info.profile,
        serde_json::to_string(&info)?
    );
    write_changed(&out.join("craftguild.rs"), source.as_bytes())?;
    Ok(info)
}

fn normalize_feature(feature: &str) -> String {
    feature.replace('-', "_").to_ascii_uppercase()
}

fn write_changed(path: &Path, bytes: &[u8]) -> Result<()> {
    if std::fs::read(path).ok().as_deref() != Some(bytes) {
        std::fs::write(path, bytes)?;
    }
    Ok(())
}
