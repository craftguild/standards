//! Copy-only installation from verified receipts; no compiler or bootstrap paths.

use crate::{
    config::{self, Kind},
    execution_context::{absolute, Context, Options},
    execution_record::Receipt,
};
use craftguild_core::{paths, Result};
use std::path::Path;

pub fn run(args: &[String]) -> Result<()> {
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        println!("Usage: install [--artifact ID] [--target TRIPLE] [--profile debug|release] [--target-dir PATH] [--prefix PATH] [--bindir PATH] [--destdir PATH]");
        return Ok(());
    }
    let root = std::env::current_dir()?.canonicalize()?;
    let options = Options::parse(args)?;
    let config = config::load(&root)?;
    let selected: Vec<_> = match options.get("artifact") {
        Some(id) => vec![config.artifact(&id)?],
        None => config
            .artifacts
            .iter()
            .filter(|a| {
                a.kind == Kind::Rust && a.build.as_ref().is_none_or(|b| b.library.is_none())
            })
            .collect(),
    };
    if selected.is_empty() {
        return Err("No executable artifact selected for install".into());
    }
    for artifact in selected {
        if artifact.kind != Kind::Rust
            || artifact.build.as_ref().is_some_and(|b| b.library.is_some())
        {
            return Err("Install requires an executable Rust artifact".into());
        }
        let path = format!(".craftguild/{}/receipt.json", artifact.command);
        let receipt: Receipt =
            serde_json::from_slice(&std::fs::read(paths::owned(&root, &path).map_err(
                |_| "No verified build receipt. Run setup and make build without privileges first.",
            )?)?)?;
        let mut selectors = options.clone();
        selectors.standard_layout = config.rust_output_layout.is_some();
        selectors.offline = true;
        selectors.known_host = receipt.context.host.clone();
        selectors.known_compiler = Some(receipt.context.compiler_version.clone());
        if selectors.get("target").is_none() && artifact.target.is_none() {
            selectors.fallback_target = Some(receipt.context.target.clone());
            selectors.fallback_explicit_target = receipt.context.explicit_target;
        }
        if selectors.get("toolchain").is_none()
            && artifact
                .build
                .as_ref()
                .and_then(|b| b.toolchain.as_ref())
                .is_none()
        {
            if let Some(toolchain) = &receipt.context.toolchain {
                selectors
                    .values
                    .insert("toolchain".into(), toolchain.clone());
            }
        }
        let context = Context::resolve(&root, artifact, &selectors)?;
        if context != receipt.context {
            return Err("Install selectors differ from the verified build. Run make build with matching selectors first.".into());
        }
        receipt.verify(&root)?;
        let bytes = std::fs::read(&context.output)?;
        if receipt.outputs.get(&context.file) != Some(&paths::digest(&bytes)) {
            return Err("Built output changed during install verification".into());
        }
        let prefix = options.get("prefix").unwrap_or_else(|| "/usr/local".into());
        let bindir = options
            .get("bindir")
            .unwrap_or_else(|| format!("{prefix}/bin"));
        if !Path::new(&bindir).is_absolute() {
            return Err("BINDIR must be absolute; use DESTDIR for staging".into());
        }
        let directory = match options.get("destdir").filter(|s| !s.is_empty()) {
            Some(destdir) => absolute(&root, &destdir)?.join(Path::new(&bindir).strip_prefix("/")?),
            None => absolute(&root, &bindir)?,
        };
        let destination = directory.join(
            context
                .output
                .file_name()
                .ok_or("Built output has no name")?,
        );
        absolute(
            &root,
            destination.to_str().ok_or("Non-UTF8 install destination")?,
        )?;
        std::fs::create_dir_all(&directory)?;
        crate::distribution::atomic_write(&destination, &bytes)?;
        std::fs::set_permissions(
            &destination,
            std::fs::metadata(&context.output)?.permissions(),
        )?;
        println!("{}", destination.display());
    }
    Ok(())
}
