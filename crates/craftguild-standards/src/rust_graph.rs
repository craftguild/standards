//! Selected Cargo runtime graph: target-filtered, no development/build/proc-macro edges.

use crate::{command, config::Artifact};
use craftguild_core::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub packages: Vec<Package>,
    pub workspace_root: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub manifest_path: PathBuf,
    pub source: Option<String>,
    pub targets: Vec<Target>,
}

#[derive(Deserialize, Debug)]
pub struct Target {
    pub name: String,
    pub kind: Vec<String>,
}

pub struct Graph {
    pub version: String,
    pub features: Vec<String>,
    pub components: Vec<Value>,
    pub dependencies: Vec<Value>,
    pub local_inputs: BTreeSet<String>,
    pub local_dirs: BTreeSet<String>,
}

pub fn flags(a: &Artifact, target: &str) -> Vec<String> {
    let mut flags = vec![
        "--locked".into(),
        "--manifest-path".into(),
        a.manifest.clone(),
        "--target".into(),
        target.into(),
        "-p".into(),
        a.package.clone(),
    ];
    if a.no_default_features {
        flags.push("--no-default-features".into());
    }
    if !a.features.is_empty() {
        flags.extend(["--features".into(), a.features.join(",")]);
    }
    flags
}

pub fn resolve(root: &Path, a: &Artifact, target: &str) -> Result<Graph> {
    let metadata: Metadata = serde_json::from_slice(&command::capture(
        root,
        &[
            "cargo".into(),
            "metadata".into(),
            "--locked".into(),
            "--format-version".into(),
            "1".into(),
            "--manifest-path".into(),
            a.manifest.clone(),
            "--filter-platform".into(),
            target.into(),
        ],
    )?)?;
    let workspace = metadata.workspace_root.canonicalize()?;
    if !workspace.starts_with(root.canonicalize()?) {
        return Err("Artifact workspace must be inside the explicit source root".into());
    }
    if root.join(&a.lockfile).canonicalize()? != workspace.join("Cargo.lock").canonicalize()? {
        return Err("Artifact lockfile must be the selected Cargo workspace lockfile".into());
    }
    let mut argv = vec!["cargo".into(), "tree".into()];
    argv.extend(flags(a, target));
    argv.extend(
        [
            "--edges",
            "normal,no-proc-macro",
            "--no-dedupe",
            "--prefix",
            "depth",
            "--format",
            "{p}|{f}",
            "--color",
            "never",
        ]
        .map(str::to_owned),
    );
    let tree = String::from_utf8(command::capture(root, &argv)?)?;
    parse(root, a, &metadata, &tree)
}

fn parse(root: &Path, a: &Artifact, metadata: &Metadata, tree: &str) -> Result<Graph> {
    let root = root.canonicalize()?;
    let mut graph = Graph {
        version: String::new(),
        features: vec![],
        components: vec![],
        dependencies: vec![],
        local_inputs: BTreeSet::new(),
        local_dirs: BTreeSet::new(),
    };
    let mut components = BTreeMap::new();
    for path in [".cargo/config.toml", ".cargo/config"] {
        if root.join(path).exists() {
            graph.local_inputs.insert(path.into());
        }
    }
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut stack: Vec<String> = vec![];
    for line in tree.lines().filter(|s| !s.is_empty()) {
        let split = line
            .find(|c: char| !c.is_ascii_digit())
            .ok_or("Invalid Cargo tree depth")?;
        let depth: usize = line[..split].parse()?;
        let (display, features) = line[split..]
            .split_once('|')
            .ok_or("Invalid Cargo tree package")?;
        let mut words = display.split_whitespace();
        let name = words.next().ok_or("Missing Cargo package")?;
        let version = words
            .next()
            .and_then(|v| v.strip_prefix('v'))
            .ok_or("Missing Cargo version")?;
        let candidates: Vec<_> = metadata
            .packages
            .iter()
            .filter(|p| p.name == name && p.version == version)
            .collect();
        let package = if candidates.len() == 1 {
            candidates[0]
        } else {
            // Refuse ambiguity instead of inventing the identity of duplicate sources.
            let local = candidates.iter().find(|p| {
                display.contains(&format!(
                    "({})",
                    p.manifest_path.parent().unwrap().display()
                ))
            });
            *local.ok_or("Ambiguous Cargo package sources; use unique package identities")?
        };
        let id = package_ref(package);
        if depth > stack.len() {
            return Err("Invalid Cargo tree nesting".into());
        }
        if depth == 0 {
            if package.name != a.package
                || !package
                    .targets
                    .iter()
                    .any(|t| t.name == a.command && t.kind.iter().any(|k| k == "bin"))
            {
                return Err("Artifact command must select a real binary in its package".into());
            }
            graph.version = package.version.clone();
            graph.features = features
                .split(',')
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect();
            graph.features.sort();
            if !stack.is_empty() {
                return Err("Expected exactly one artifact root".into());
            }
        } else {
            edges
                .entry(stack[depth - 1].clone())
                .or_default()
                .insert(id.clone());
            let mut component = json!({"type":"library", "bom-ref": id,
                "name":package.name, "version":package.version});
            if package.source.is_some() {
                component["purl"] = json!(package_ref(package));
            }
            components.insert(id.clone(), component);
        }
        edges.entry(id.clone()).or_default();
        stack.truncate(depth);
        stack.push(id);
    }
    if graph.version.is_empty() {
        return Err("Empty Cargo runtime graph".into());
    }
    // Seal all local build inputs too; build-only packages stay out of components.
    for p in metadata.packages.iter().filter(|p| p.source.is_none()) {
        let path = p
            .manifest_path
            .strip_prefix(&root)
            .map_err(|_| "Local package outside source root; remove sibling path dependencies")?;
        graph
            .local_inputs
            .insert(path.to_str().ok_or("Non-UTF8 manifest")?.into());
        let dir = p.manifest_path.parent().unwrap();
        for name in ["build.rs", ".cargo/config.toml", ".cargo/config"] {
            if dir.join(name).is_file() {
                graph.local_inputs.insert(
                    dir.join(name)
                        .strip_prefix(&root)?
                        .to_str()
                        .ok_or("Non-UTF8 input")?
                        .into(),
                );
            }
        }
        if dir.join("src").is_dir() {
            graph.local_dirs.insert(
                dir.join("src")
                    .strip_prefix(&root)?
                    .to_str()
                    .ok_or("Non-UTF8 source")?
                    .into(),
            );
        }
    }
    graph.components = components.into_values().collect();
    graph.dependencies = edges
        .into_iter()
        .map(|(id, deps)| json!({"ref":id,"dependsOn":deps}))
        .collect();
    Ok(graph)
}

fn package_ref(p: &Package) -> String {
    let base = format!("pkg:cargo/{}@{}", p.name, p.version);
    match &p.source {
        Some(source) if source.starts_with("git+") => {
            format!("{base}?vcs_url={}", encode(&source[4..]))
        }
        Some(source) if !source.contains("crates.io") => {
            format!("{base}?repository_url={}", encode(source))
        }
        None => format!("urn:craftguild:local:{}:{}", p.name, p.version),
        _ => base,
    }
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
