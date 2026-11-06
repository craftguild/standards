//! Canonical artifact preparation; compiler invocation follows verified publication.

use crate::{
    command,
    config::{Artifact, Kind},
    execution_process::Runner,
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

/// Compatibility entry: actual builds use the same executor as CLI and Make.
pub fn build(root: &Path, artifact: &Artifact, target: &str, compile: bool) -> Result<Prepared> {
    if compile {
        use crate::execution_context::{Context, Options};
        let root = root.canonicalize()?;
        let mut options = Options::parse(&["--target".into(), target.into()])?;
        options.standard_layout = crate::config::load(&root)?.rust_output_layout.is_some();
        let context = Context::resolve(&root, artifact, &options)?;
        let _lock = crate::execution_transaction::Lock::acquire(&root)?;
        return crate::execution::execute(
            &root,
            artifact,
            context,
            &options,
            true,
            artifact.build.is_some(),
        );
    }
    let runner = Runner::default();
    let prepared = prepare_with(
        root,
        artifact,
        target,
        "release",
        &format!(".craftguild/{}", artifact.command),
        &runner,
        true,
    )?;
    Ok(prepared)
}

/// Seal one resolved build context after the engine has prepared required assets.
pub fn prepare_with(
    root: &Path,
    artifact: &Artifact,
    target: &str,
    profile: &str,
    directory: &str,
    runner: &Runner,
    build_bundle: bool,
) -> Result<Prepared> {
    if target.is_empty()
        || !target
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
    {
        return Err(
            "Use a named target; custom target specification files are not supported".into(),
        );
    }
    let root = root.canonicalize()?;
    owned_directory(&root, directory)?;
    // Invalidate the old publication before commands run, so failure cannot reuse it.
    let destination = root.join(directory).join("prepared.json");
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
    let mut root_license = None;
    let library = artifact
        .build
        .as_ref()
        .and_then(|build| build.library.as_ref());
    let (version, features, mut components, mut dependencies, root_ref) = match artifact.kind {
        Kind::Rust => {
            let graph = rust_graph::resolve_with(&root, artifact, target, runner)?;
            root_license = graph.license;
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
    if build_bundle {
        if let Some(bundle) = &artifact.bundle {
            runner.run(&root, &bundle.command)?;
        }
    }
    let records: Vec<Supplement> = if let Some(path) = &artifact.supplemental {
        files.insert(path.clone());
        let records: Vec<Supplement> =
            serde_json::from_slice(&std::fs::read(paths::owned(&root, path)?)?)?;
        if records.is_empty() {
            return Err("Configured supplemental components cannot be empty".into());
        }
        records
    } else {
        vec![]
    };
    let supplemental_files = records
        .iter()
        .flat_map(|r| r.files.iter().cloned())
        .collect();
    if let Some(bundle) = &artifact.bundle {
        let bundled = npm_bundle::collect(&root, bundle, &supplemental_files)?;
        files.extend(bundled.files);
        dirs.insert(bundle.directory.clone());
        extra.extend(bundled.components);
    }
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
        let mut component = json!({"type":record.kind,"bom-ref":record.reference,
                "name":record.name,"version":record.version,
                "properties":[{"name":"craftguild:supplemental-files","value":serde_json::to_string(&hashes)?},
                    {"name":"craftguild:evidence","value":record.evidence}]});
        if let Some(license) = record.license {
            if license.trim().is_empty() {
                return Err("Supplemental license expression cannot be empty".into());
            }
            component["licenses"] = json!([{"expression":license}]);
        }
        extra.push(component);
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
    let info = if artifact.kind == Kind::Rust {
        let mut info = observe::capture(&root, &artifact.command, &version, target, profile);
        info.compiler =
            String::from_utf8(runner.capture(&root, &["rustc".into(), "--version".into()])?)?
                .trim()
                .to_owned();
        info
    } else {
        let (commit, dirty) = observe::repository(&root);
        let node = String::from_utf8(runner.capture(&root, &["node".into(), "--version".into()])?)?;
        craftguild_core::BuildInfo {
            command: artifact.command.clone(),
            version: version.clone(),
            commit,
            dirty,
            compiler: format!("node {}", node.trim()),
            target: target.into(),
            profile: profile.into(),
        }
    };
    info.validate()?;
    if runner
        .compiler_version
        .as_ref()
        .is_some_and(|expected| expected != &info.compiler)
    {
        return Err("Compiler changed after context resolution".into());
    }
    let mut prepared = Prepared {
        schema_version: 2,
        build: info,
        features,
        inputs,
        inventories,
        sbom: format!("{directory}/bom.json"),
        sbom_sha256: String::new(),
    };
    let mut component = json!({"type":if library.is_some() { "library" } else { "application" },
        "bom-ref":root_ref,"name":artifact.command,"version":version});
    if let Some(license) = root_license {
        component["licenses"] = json!([{"expression":license}]);
    }
    if let Some(library) = library {
        let mut properties = library.properties.clone();
        properties.insert("craftguild:library-target".into(), library.name.clone());
        component["properties"] = json!(properties
            .into_iter()
            .map(|(name, value)| json!({"name":name,"value":value}))
            .collect::<Vec<_>>());
    }
    let bom = json!({"$schema":"http://cyclonedx.org/schema/bom-1.5.schema.json",
        "bomFormat":"CycloneDX","specVersion":"1.5","version":1,
        "metadata":{"component":component,"properties":prepared.properties()?},
        "components":components,"dependencies":dependencies});
    let mut bytes = serde_json::to_vec_pretty(&bom)?;
    bytes.push(b'\n');
    sbom::validate(&bytes)?;
    prepared.sbom_sha256 = paths::digest(&bytes);
    atomic_write(&root.join(&prepared.sbom), &bytes)?;
    prepared.verify(&root, &prepared.build, &prepared.features)?;
    atomic_write(&destination, &serde_json::to_vec_pretty(&prepared)?)?;
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
    license: Option<String>,
}

pub(crate) fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
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

pub(crate) fn owned_directory(root: &Path, name: &str) -> Result<()> {
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
    crate::execution::run(args, compile)
}
