//! Canonical artifact preparation; compiler invocation follows verified publication.

use crate::{
    command,
    config::{self, Artifact, Kind},
    npm_bundle, rust_graph,
};
use craftguild_cli_build::observe;
use craftguild_core::{paths, sbom, seal::Prepared, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn host(root: &Path) -> Result<String> {
    let bytes = command::capture(
        root,
        &[
            std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()),
            "-vV".into(),
        ],
    )?;
    String::from_utf8(bytes)?
        .lines()
        .find_map(|s| s.strip_prefix("host: ").map(str::to_owned))
        .ok_or_else(|| "Compiler did not identify its host target".into())
}

/// Prepare one artifact and optionally compile its locked release binary.
pub fn build(root: &Path, artifact: &Artifact, target: &str, compile: bool) -> Result<Prepared> {
    let root = root.canonicalize()?;
    let directory = format!(".craftguild/{}", artifact.command);
    owned_directory(&root, &directory)?;
    // Invalidate the old publication before commands run, so failure cannot reuse it.
    let destination = root.join(&directory).join("prepared.json");
    if destination.exists() {
        std::fs::remove_file(&destination)?;
    }
    let mut files: BTreeSet<_> = artifact.inputs.iter().cloned().collect();
    files.extend([
        "craftguild.toml".into(),
        artifact.manifest.clone(),
        artifact.lockfile.clone(),
    ]);
    let mut dirs: BTreeSet<_> = artifact.input_dirs.iter().cloned().collect();
    let (version, features, mut components, mut dependencies, root_ref) = match artifact.kind {
        Kind::Rust => {
            let graph = rust_graph::resolve(&root, artifact, target)?;
            files.extend(graph.local_inputs);
            dirs.extend(graph.local_dirs);
            let id = format!(
                "urn:craftguild:local:{}:{}",
                artifact.package, graph.version
            );
            (
                graph.version,
                graph.features,
                graph.components,
                graph.dependencies,
                id,
            )
        }
        Kind::Frontend => {
            let manifest: Value =
                serde_json::from_slice(&std::fs::read(paths::owned(&root, &artifact.manifest)?)?)?;
            let version = manifest["version"]
                .as_str()
                .ok_or("Frontend package has no version")?
                .to_owned();
            let id = format!("urn:craftguild:frontend:{}:{version}", artifact.package);
            (
                version,
                vec![],
                vec![],
                vec![json!({"ref":id,"dependsOn":[]})],
                id,
            )
        }
    };
    let mut extra = vec![];
    if let Some(bundle) = &artifact.bundle {
        let bundled = npm_bundle::collect(&root, bundle)?;
        files.extend(bundled.files);
        dirs.insert(bundle.directory.clone());
        extra.extend(bundled.components);
    }
    if let Some(supplemental) = &artifact.supplemental {
        files.insert(supplemental.clone());
        let records: Vec<Supplement> =
            serde_json::from_slice(&std::fs::read(paths::owned(&root, supplemental)?)?)?;
        for record in records {
            if record.name.is_empty()
                || record.version.is_empty()
                || record.files.is_empty()
                || record.evidence.trim().is_empty()
            {
                return Err(
                    "Supplemental components require identity, actual files and evidence".into(),
                );
            }
            let hashes: Vec<_> = record
                .files
                .iter()
                .map(|name| {
                    Ok(json!({"name":name,
                "sha256":paths::hash_file(&root,name)?}))
                })
                .collect::<Result<_>>()?;
            files.extend(record.files);
            extra.push(json!({"type":record.kind,"bom-ref":record.reference,
                "name":record.name,"version":record.version,
                "properties":[{"name":"craftguild:supplemental-files","value":serde_json::to_string(&hashes)?},
                    {"name":"craftguild:evidence","value":record.evidence}]}));
        }
    }
    let root_edges = dependencies
        .iter_mut()
        .find(|d| d["ref"] == root_ref)
        .ok_or("Missing artifact graph root")?;
    for component in &extra {
        let id = component["bom-ref"]
            .as_str()
            .ok_or("Supplement has no reference")?;
        root_edges["dependsOn"]
            .as_array_mut()
            .ok_or("Invalid artifact edges")?
            .push(json!(id));
    }
    dependencies.extend(
        extra
            .iter()
            .map(|c| json!({"ref":c["bom-ref"],"dependsOn":[]})),
    );
    components.extend(extra);
    let mut inventories = BTreeMap::new();
    for dir in dirs {
        let inventory = paths::inventory(&root, &dir)?;
        files.extend(inventory.iter().cloned());
        inventories.insert(dir, inventory);
    }
    let inputs = files
        .into_iter()
        .map(|name| Ok((name.clone(), paths::hash_file(&root, &name)?)))
        .collect::<Result<_>>()?;
    let info = observe::capture(&root, &artifact.command, &version, target, "release");
    info.validate()?;
    let mut prepared = Prepared {
        schema_version: 1,
        build: info,
        features,
        inputs,
        inventories,
        sbom: format!("{directory}/bom.json"),
        sbom_sha256: String::new(),
    };
    let bom = json!({"$schema":"http://cyclonedx.org/schema/bom-1.5.schema.json",
        "bomFormat":"CycloneDX","specVersion":"1.5","version":1,
        "metadata":{"component":{"type":"application","bom-ref":root_ref,
            "name":artifact.command,"version":version},
            "properties":prepared.properties()?},
        "components":components,"dependencies":dependencies});
    let mut bytes = serde_json::to_vec_pretty(&bom)?;
    bytes.push(b'\n');
    sbom::validate(&bytes)?;
    prepared.sbom_sha256 = paths::digest(&bytes);
    atomic_write(&root.join(&prepared.sbom), &bytes)?;
    prepared.verify(&root, &prepared.build, &prepared.features)?;
    atomic_write(&destination, &serde_json::to_vec_pretty(&prepared)?)?;
    if compile && artifact.kind == Kind::Rust {
        let mut argv = vec!["cargo".into(), "build".into(), "--release".into()];
        argv.extend(rust_graph::flags(artifact, target));
        argv.extend(["--bin".into(), artifact.command.clone()]);
        command::run(&root, &argv)?;
    }
    Ok(prepared)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Supplement {
    name: String,
    version: String,
    #[serde(rename = "type")]
    kind: String,
    reference: String,
    files: Vec<String>,
    evidence: String,
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    use std::io::Write;
    if path.is_symlink() {
        return Err("Prepared output cannot replace a symlink".into());
    }
    let mut staging =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing output parent")?)?;
    staging.write_all(data)?;
    staging.persist(path)?;
    Ok(())
}

fn owned_directory(root: &Path, name: &str) -> Result<()> {
    let mut current = root.to_owned();
    for part in paths::relative(name)?.components() {
        current.push(part);
        if current.exists() || current.is_symlink() {
            let kind = std::fs::symlink_metadata(&current)?.file_type();
            if kind.is_symlink() || !kind.is_dir() {
                return Err("Preparation directory is not an owned directory".into());
            }
        } else {
            std::fs::create_dir(&current)?;
        }
    }
    Ok(())
}

pub fn run(args: &[String], compile: bool) -> Result<()> {
    if matches!(args,[flag] if flag=="-h" || flag=="--help") {
        println!("Usage: build|prepare [--artifact ID] [--target TRIPLE]");
        return Ok(());
    }
    let root = std::env::current_dir()?.canonicalize()?;
    let config = config::load(&root)?;
    let mut id = None;
    let mut target = None;
    for pair in args.chunks(2) {
        match pair {
            [flag, value] if flag == "--artifact" && id.is_none() => id = Some(value.as_str()),
            [flag, value] if flag == "--target" && target.is_none() => target = Some(value.clone()),
            _ => return Err("Usage: build|prepare [--artifact ID] [--target TRIPLE]".into()),
        }
    }
    let target = target.map_or_else(|| host(&root), Ok)?;
    let selected: Vec<_> = if let Some(id) = id {
        vec![config.artifact(id)?]
    } else {
        config.artifacts.iter().collect()
    };
    for a in selected {
        let prepared = build(&root, a, &target, compile)?;
        eprintln!("Prepared {} for {}: {}", a.id, target, prepared.sbom);
    }
    Ok(())
}
