//! Read-only shared verification utilities used by bounded product gates.

use crate::{
    config::{self, Kind},
    execution_context::{Context, Options},
    execution_process::Runner,
    measure, quality,
    release::versions,
    rust_graph,
};
use craftguild_cli_build::observe;
use craftguild_core::{paths, sbom, seal::Prepared, BuildInfo, Result};
use std::{io::Write, path::Path};

pub fn run(name: &str, args: &[String], output: &mut impl Write) -> Result<()> {
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        writeln!(output, "Usage: {name} {}", match name {
            "check-release-config" => "",
            "validate-sbom" => "--path FILE",
            "assess" => "[--root PATH] [--changed | --base REV] [--json]",
            "measure" => "--path FILE",
            "verify-receipt" => "--artifact ID",
            _ => "--artifact ID [--target TRIPLE] [--profile debug|release] [--toolchain NAME] [--features LIST] [--record FILE]",
        })?;
        return Ok(());
    }
    let root = std::env::current_dir()?.canonicalize()?;
    match (name, args) {
        ("verify-receipt", [flag, id]) if flag == "--artifact" => {
            let config = config::load(&root)?;
            let artifact = config.artifact(id)?;
            let path = format!(".craftguild/{}/receipt.json", artifact.command);
            let receipt: crate::execution_record::Receipt =
                serde_json::from_slice(&std::fs::read(paths::owned(&root, &path)?)?)?;
            if receipt.context.artifact != *id || receipt.context.command != artifact.command {
                return Err("Receipt belongs to another artifact".into());
            }
            receipt.verify(&root)
        }
        ("check-release-config", []) => release_config(&root, output),
        ("validate-sbom", [flag, file]) if flag == "--path" => {
            sbom::validate(&std::fs::read(paths::owned(&root, file)?)?)?;
            Ok(())
        }
        ("measure", [flag, file]) if flag == "--path" => {
            writeln!(output, "{}", measure::file(&root, file)?)?;
            Ok(())
        }
        ("measure-conditional", [file]) => {
            writeln!(output, "{}", measure::file(&root, file)?)?;
            Ok(())
        }
        ("assess", args) => assess(&root, args, output),
        ("verify" | "verify-frontend", args) => verify(&root, args, name == "verify-frontend"),
        _ => Err(format!("Invalid {name} arguments; use --help").into()),
    }
}

fn release_config(root: &Path, output: &mut impl Write) -> Result<()> {
    let config = config::load(root)?;
    if config.releases.is_empty() {
        return Err("No release groups configured".into());
    }
    for group in &config.releases {
        let current = versions::current(root, group)?;
        let mut candidate = versions::stable(&current)?;
        candidate.patch = candidate.patch.checked_add(1).ok_or("Version overflow")?;
        let changes = versions::changes(root, group, &current, &candidate.to_string())?;
        if changes.is_empty() {
            return Err("Candidate version produced no owned edits".into());
        }
        for change in changes {
            if change.before.as_deref() != Some(&paths::hash_file(root, &change.path)?) {
                return Err("Version ownership differs from current source".into());
            }
        }
        if !versions::changes(root, group, &current, &current)?.is_empty() {
            return Err("Unchanged version produced duplicate edits".into());
        }
        writeln!(
            output,
            "Verified {} version ownership at {current}; no files written",
            group.id
        )?;
    }
    Ok(())
}

fn assess(default_root: &Path, args: &[String], output: &mut impl Write) -> Result<()> {
    let mut root = default_root.to_owned();
    let mut changed = None;
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--root" => {
                root = Path::new(args.next().ok_or("Missing assessment root")?).canonicalize()?
            }
            "--changed" if changed.is_none() => changed = Some("HEAD".to_owned()),
            "--base" if changed.is_none() => {
                changed = Some(args.next().ok_or("Missing base revision")?.clone())
            }
            "--json" => {}
            _ => {
                return Err("Usage: assess [--root PATH] [--changed | --base REV] [--json]".into())
            }
        }
    }
    let config = config::load(&root)?;
    let report = quality::assess(&root, &config, changed.as_deref())?;
    serde_json::to_writer_pretty(&mut *output, &report)?;
    writeln!(output)?;
    if report.findings.is_empty() {
        Ok(())
    } else {
        Err("Quality assessment failed; see rule findings".into())
    }
}

fn verify(root: &Path, args: &[String], frontend_only: bool) -> Result<()> {
    let mut selectors = vec![];
    let mut record = None;
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        if flag == "--record" {
            if record.is_some() {
                return Err("Duplicate prepared record selector".into());
            }
            record = Some(args.next().ok_or("Missing prepared record")?.clone());
        } else {
            selectors.push(flag.clone());
        }
    }
    let options = Options::parse(&selectors)?;
    let config = config::load(root)?;
    let id = options
        .get("artifact")
        .ok_or("Select --artifact ID to verify")?;
    let artifact = config.artifact(&id)?;
    if frontend_only && artifact.kind != Kind::Frontend {
        return Err("Expected a frontend artifact".into());
    }
    let path = record.unwrap_or_else(|| format!(".craftguild/{}/prepared.json", artifact.command));
    let prepared: Prepared = serde_json::from_slice(&std::fs::read(paths::owned(root, &path)?)?)?;
    let mut options = options;
    options.standard_layout = config.rust_output_layout.is_some();
    options.fallback_target = Some(prepared.build.target.clone());
    let context = Context::resolve(root, artifact, &options)?;
    let selected = context.artifact(artifact);
    let runner = Runner {
        toolchain: context.toolchain.clone(),
        ..Runner::default()
    };
    let (version, features, compiler) = match artifact.kind {
        Kind::Rust => {
            let graph = rust_graph::resolve_with(root, &selected, &context.target, &runner)?;
            (
                graph.version,
                graph.features,
                String::from_utf8(runner.capture(root, &["rustc".into(), "--version".into()])?)?
                    .trim()
                    .to_owned(),
            )
        }
        Kind::Frontend => {
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(paths::owned(root, &artifact.manifest)?)?)?;
            (
                manifest["version"]
                    .as_str()
                    .ok_or("Missing frontend version")?
                    .into(),
                vec![],
                format!(
                    "node {}",
                    String::from_utf8(runner.capture(root, &["node".into(), "--version".into()])?)?
                        .trim()
                ),
            )
        }
    };
    let (commit, dirty) = observe::repository(root);
    let expected = BuildInfo {
        command: artifact.command.clone(),
        version,
        commit,
        dirty,
        compiler,
        target: context.target,
        profile: context.profile.name().into(),
    };
    let bytes = prepared.verify(root, &expected, &features)?;
    if artifact.build.as_ref().is_some_and(|b| b.library.is_some())
        && sbom::validate(&bytes)?["metadata"]["component"]["type"] != "library"
    {
        return Err("Expected a library SBOM".into());
    }
    Ok(())
}
