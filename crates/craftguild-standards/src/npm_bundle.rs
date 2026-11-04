//! Map actual esbuild output inputs to installed npm packages and locked identities.

use crate::config::Bundle;
use craftguild_core::{paths, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct Bundled {
    pub components: Vec<Value>,
    pub files: BTreeSet<String>,
    pub assets: BTreeSet<String>,
}

pub fn collect(root: &Path, b: &Bundle, supplemental: &BTreeSet<String>) -> Result<Bundled> {
    let lock: Value = serde_json::from_slice(&std::fs::read(paths::owned(root, &b.lockfile)?)?)?;
    if lock["lockfileVersion"] != 3 {
        return Err("npm bundle adapter requires package-lock v3".into());
    }
    let manifest_path = under(&b.root, "package.json");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(paths::owned(root, &manifest_path)?)?)?;
    let locked_root = lock["packages"][""]
        .as_object()
        .ok_or("Missing npm root package record")?;
    for key in [
        "name",
        "version",
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        if manifest.get(key) != locked_root.get(key) {
            return Err(format!("npm manifest differs from locked root: {key}").into());
        }
    }
    let meta: Value = serde_json::from_slice(&std::fs::read(paths::owned(root, &b.metafile)?)?)?;
    let inputs = meta["inputs"].as_object().ok_or("Missing esbuild inputs")?;
    let outputs = meta["outputs"]
        .as_object()
        .ok_or("Missing esbuild outputs")?;
    if outputs.is_empty() {
        return Err("Empty bundle output inventory".into());
    }
    let actual = paths::inventory(root, &b.directory)?;
    let mut assets = BTreeSet::new();
    let mut files = BTreeSet::from([b.lockfile.clone(), b.metafile.clone(), manifest_path]);
    let tsconfig = under(&b.root, "tsconfig.json");
    if root.join(&tsconfig).exists() {
        files.insert(tsconfig);
    }
    let mut packages = BTreeMap::new();
    for (output, record) in outputs {
        let path = under(&b.root, output);
        let bytes = std::fs::read(paths::owned(root, &path)?)?;
        if record["bytes"].as_u64() != Some(bytes.len() as u64) || !actual.contains(&path) {
            return Err(format!("Bundle output bytes/ownership mismatch: {path}").into());
        }
        assets.insert(path);
        for (input, contribution) in record["inputs"]
            .as_object()
            .ok_or("Missing output input evidence")?
        {
            if contribution["bytesInOutput"]
                .as_u64()
                .ok_or("Missing bundle byte contribution")?
                == 0
            {
                continue;
            }
            let full = under(&b.root, input);
            let data = std::fs::read(paths::owned(root, &full)?)?;
            if inputs[input]["bytes"].as_u64() != Some(data.len() as u64) {
                return Err(format!("Bundle input differs from esbuild evidence: {full}").into());
            }
            files.insert(full);
            if let Some(key) = package_key(input)? {
                let entry = &lock["packages"][&key];
                let version = entry["version"]
                    .as_str()
                    .ok_or("Bundled npm package absent from lockfile")?;
                if entry["link"] == true {
                    return Err("Linked npm packages need explicit supplemental records".into());
                }
                let manifest_path = under(&b.root, &format!("{key}/package.json"));
                let manifest: Value =
                    serde_json::from_slice(&std::fs::read(paths::owned(root, &manifest_path)?)?)?;
                let name = manifest["name"]
                    .as_str()
                    .ok_or("Installed npm package has no name")?;
                let locked_name = entry["name"]
                    .as_str()
                    .unwrap_or_else(|| key.rsplit("node_modules/").next().unwrap());
                if manifest["version"] != version || name != locked_name {
                    return Err("Installed npm package differs from lockfile".into());
                }
                files.insert(manifest_path);
                let id = format!("pkg:npm/{}@{version}", name.replace('@', "%40"));
                packages.insert(
                    id.clone(),
                    json!({"type":"library", "bom-ref":id,
                    "name":name,"version":version,"purl":id,
                    "properties":[{"name":"craftguild:npm-lock-path","value":key}]}),
                );
            }
        }
    }
    assets.extend(actual.intersection(supplemental).cloned());
    if assets != actual {
        return Err("Missing or extra files in bundled asset directory".into());
    }
    files.extend(assets.iter().cloned());
    Ok(Bundled {
        components: packages.into_values().collect(),
        files,
        assets,
    })
}

fn package_key(input: &str) -> Result<Option<String>> {
    paths::relative(input)?;
    let parts: Vec<_> = input.split('/').collect();
    let Some(index) = parts.iter().rposition(|p| *p == "node_modules") else {
        return Ok(None);
    };
    let mut end = index + 2;
    let name = parts.get(index + 1).ok_or("Invalid npm bundle input")?;
    if name.starts_with('@') {
        end += 1;
    }
    if end > parts.len() {
        return Err("Invalid scoped npm bundle input".into());
    }
    Ok(Some(parts[..end].join("/")))
}

fn under(root: &str, name: &str) -> String {
    if root == "." {
        name.into()
    } else {
        format!("{root}/{name}")
    }
}
