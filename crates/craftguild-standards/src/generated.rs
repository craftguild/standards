//! Verify ownership, exact inventories, hashes and isolated generator reproduction.

use crate::{command, config::Generated};
use craftguild_core::{paths, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    owner: String,
    generator_revision: String,
    inputs: BTreeMap<String, String>,
    files: BTreeMap<String, String>,
}

/// Return only verified Rust paths eligible for the size exception.
pub fn verify(root: &Path, groups: &[Generated]) -> Result<BTreeSet<String>> {
    let mut owned = BTreeSet::new();
    let mut exempt = BTreeSet::new();
    for group in groups {
        let manifest: Manifest =
            serde_json::from_slice(&std::fs::read(paths::owned(root, &group.manifest)?)?)?;
        if manifest.schema_version != 1
            || manifest.owner != group.owner
            || manifest.inputs.is_empty()
            || manifest.files.is_empty()
            || !matches!(manifest.generator_revision.len(), 40 | 64)
            || !manifest
                .generator_revision
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err("Incomplete generated ownership/provenance".into());
        }
        let expected: BTreeSet<_> = manifest.files.keys().cloned().collect();
        if paths::inventory(root, &group.output_root)? != expected {
            return Err("Missing or extra files in generated output ownership".into());
        }
        for (name, hash) in manifest.inputs.iter().chain(&manifest.files) {
            if paths::hash_file(root, name)? != *hash {
                return Err(format!("Generated input/output hash mismatch: {name}").into());
            }
        }
        let temporary = tempfile::tempdir()?;
        let argv: Vec<_> = group
            .reproduce
            .iter()
            .map(|arg| {
                if arg == "{output}" {
                    temporary.path().to_str().unwrap().to_owned()
                } else {
                    arg.clone()
                }
            })
            .collect();
        command::run(root, &argv)?;
        let relative: BTreeMap<_, _> = manifest
            .files
            .iter()
            .map(|(name, hash)| {
                let path = name
                    .strip_prefix(&(group.output_root.clone() + "/"))
                    .ok_or("Generated file escapes owned output")?;
                Ok((path.to_owned(), hash))
            })
            .collect::<Result<_>>()?;
        // Inventory an already existing temporary directory through its parent so
        // the same safe-path and symlink checks apply to generated output.
        let temp_parent = temporary
            .path()
            .parent()
            .ok_or("Missing temporary parent")?;
        let temp_name = temporary
            .path()
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid temporary name")?;
        let actual = paths::inventory(temp_parent, temp_name)?;
        let actual: BTreeSet<_> = actual
            .into_iter()
            .map(|s| s[temp_name.len() + 1..].to_owned())
            .collect();
        if actual != relative.keys().cloned().collect() {
            return Err("Generator reproduction ownership differs".into());
        }
        for (path, hash) in relative {
            if paths::hash_file(temporary.path(), &path)? != *hash {
                return Err(format!("Nonreproducible generated output: {path}").into());
            }
        }
        for name in expected {
            if !owned.insert(name.clone()) {
                return Err("Overlapping generated ownership".into());
            }
            if group.rust_size_exception && name.ends_with(".rs") {
                exempt.insert(name);
            }
        }
    }
    Ok(exempt)
}
