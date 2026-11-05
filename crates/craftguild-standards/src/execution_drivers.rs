//! Reusable Rust/npm build behavior, independent of scheduling and publication.

use crate::{
    build_config::Profile,
    config::{Artifact, Kind},
    execution_context::Context,
    execution_process::Runner,
    execution_record::output_hashes,
    execution_transaction::Snapshot,
    rust_graph,
};
use craftguild_core::{seal::Prepared, Result};
use std::path::{Path, PathBuf};

pub fn prerequisites(
    root: &Path,
    artifact: &Artifact,
    context: &Context,
    runner: &Runner,
) -> Result<Option<String>> {
    let host = if artifact.kind == Kind::Rust {
        let compiler =
            String::from_utf8(runner.capture(root, &["rustc".into(), "--version".into()])?)?;
        if compiler.trim() != context.compiler_version {
            return Err("Compiler changed after context resolution".into());
        }
        runner.capture(root, &["cargo".into(), "--version".into()])?;
        context.host.clone()
    } else {
        None
    };
    if artifact.kind == Kind::Frontend || artifact.bundle.is_some() {
        let version = runner.capture(root, &["node".into(), "--version".into()])?;
        if artifact.kind == Kind::Frontend
            && format!("node {}", String::from_utf8(version)?.trim()) != context.compiler_version
        {
            return Err("Node changed after context resolution".into());
        }
        runner.capture(root, &["npm".into(), "--version".into()])?;
    }
    Ok(host)
}

pub fn assets(
    root: &Path,
    artifact: &Artifact,
    runner: &Runner,
    snapshot: &mut Snapshot,
    fresh: bool,
) -> Result<()> {
    if let Some(bundle) = &artifact.bundle {
        for path in [&bundle.directory, &bundle.metafile] {
            if fresh {
                snapshot.clear(&root.join(path))?;
            } else {
                snapshot.capture(&root.join(path))?;
            }
        }
        runner.run(root, &bundle.command)?;
    }
    Ok(())
}

pub fn compile(root: &Path, artifact: &Artifact, context: &Context, runner: &Runner) -> Result<()> {
    if artifact.kind == Kind::Rust {
        let mut argv = context.compiler.clone();
        argv.push("build".into());
        if context.profile == Profile::Release {
            argv.push("--release".into());
        }
        argv.extend(rust_graph::flags(artifact, &context.target));
        if artifact.build.as_ref().is_some_and(|b| b.library.is_some()) {
            argv.push("--lib".into());
        } else {
            argv.extend(["--bin".into(), artifact.command.clone()]);
        }
        runner.run(root, &argv)?;
    }
    Ok(())
}

pub fn candidates(artifact: &Artifact, context: &Context, cache: &Path) -> Result<Vec<PathBuf>> {
    context
        .outputs
        .iter()
        .map(|output| match artifact.kind {
            Kind::Rust => Ok(cache
                .join(&context.target)
                .join(context.profile.name())
                .join(output.file_name().ok_or("Output has no filename")?)),
            Kind::Frontend => Ok(output.clone()),
        })
        .collect()
}

pub fn verify(
    root: &Path,
    artifact: &Artifact,
    prepared: &Prepared,
    context: &Context,
    runner: &Runner,
    candidates: &[PathBuf],
    host: Option<&str>,
) -> Result<()> {
    let bom = prepared.verify(root, &prepared.build, &prepared.features)?;
    let library = artifact.build.as_ref().is_some_and(|b| b.library.is_some());
    for candidate in candidates {
        output_hashes(candidate)?;
        if artifact.kind == Kind::Rust {
            if !library && host == Some(&context.target) {
                let actual = runner.capture(
                    root,
                    &[
                        candidate.to_str().ok_or("Non-UTF8 candidate")?.into(),
                        "sbom".into(),
                    ],
                )?;
                if actual != bom {
                    return Err("Compiled executable SBOM differs from the sealed bytes".into());
                }
            } else if !std::fs::read(candidate)?
                .windows(bom.len())
                .any(|bytes| bytes == bom)
            {
                return Err("Library/cross-target output does not contain the sealed SBOM".into());
            }
        }
    }
    Ok(())
}
