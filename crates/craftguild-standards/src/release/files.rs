//! Guarded file replacement and immutable preparation receipts.

use super::{history, versions::Group, Plan};
use craftguild_core::{paths, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub path: String,
    pub before: Option<String>,
    pub after: String,
    #[serde(skip)]
    pub contents: String,
}
impl Change {
    pub fn existing(root: &Path, path: &str, contents: String) -> Result<Self> {
        Ok(Self {
            path: path.into(),
            before: Some(paths::hash_file(root, path)?),
            after: paths::digest(contents.as_bytes()),
            contents,
        })
    }
    pub fn new(path: String, contents: String) -> Self {
        Self {
            path,
            before: None,
            after: paths::digest(contents.as_bytes()),
            contents,
        }
    }
}

fn destination(root: &Path, name: &str) -> Result<PathBuf> {
    let mut target = root.canonicalize()?;
    for component in paths::relative(name)?.components() {
        target.push(component);
        match fs::symlink_metadata(&target) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("Symlink in release output path".into())
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(target)
}

pub fn pending(root: &Path, group: &Group) -> Result<Option<Plan>> {
    let directory = destination(root, &group.notes)?;
    if !directory.exists() {
        return Ok(None);
    }
    let mut found = None;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let plan: Plan = serde_json::from_slice(&fs::read(&path)?)?;
        if plan.group != group.id {
            continue;
        }
        if !history::git(root, &["tag", "--list", &plan.tag])?.is_empty() {
            continue;
        }
        if found.is_some() {
            return Err("Multiple pending release preparations".into());
        }
        if plan.schema_version != 1
            || plan.config_hash != paths::digest(&serde_json::to_vec(group)?)
            || !history::ancestor(root, &plan.head, "HEAD")?
        {
            return Err("Pending release configuration/ancestry changed".into());
        }
        let receipt = plan
            .receipt_path
            .as_deref()
            .ok_or("Missing release receipt path")?;
        if destination(root, receipt)? != path {
            return Err("Release receipt path mismatch".into());
        }
        let allowed: BTreeSet<_> = plan
            .changes
            .iter()
            .map(|c| c.path.clone())
            .chain([receipt.into()])
            .collect();
        let changed = history::git(
            root,
            &["diff", "--name-only", "-z", &plan.head, "HEAD", "--"],
        )?;
        if changed
            .split('\0')
            .filter(|s| !s.is_empty())
            .chain(history::dirty(root)?.iter().map(String::as_str))
            .any(|p| !allowed.contains(p))
        {
            return Err("Source/unrelated work changed after preparation; finish or revert the pending release".into());
        }
        for change in &plan.changes {
            if paths::hash_file(root, &change.path)? != change.after {
                return Err("Prepared release file drift".into());
            }
        }
        let mut plan = plan;
        plan.prepared = true;
        found = Some(plan);
    }
    Ok(found)
}

struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn apply(root: &Path, plan: &Plan) -> Result<()> {
    if plan.prepared || plan.next.is_none() {
        return Ok(());
    }
    let control = destination(root, ".craftguild")?;
    fs::create_dir_all(&control)?;
    let lock = control.join("release.lock");
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .map_err(|e| format!("Cannot lock release preparation: {e}"))?;
    let _guard = Lock(lock);
    if history::git(root, &["rev-parse", "HEAD"])? != plan.head || !history::dirty(root)?.is_empty()
    {
        return Err("HEAD or working tree changed since preview".into());
    }
    let receipt = plan.receipt_path.clone().ok_or("Missing release receipt")?;
    let mut changes = plan.changes.clone();
    changes.push(Change::new(
        receipt,
        format!("{}\n", serde_json::to_string_pretty(plan)?),
    ));
    let stage = tempfile::tempdir_in(&control)?;
    let mut originals = Vec::new();
    for (i, c) in changes.iter().enumerate() {
        let target = destination(root, &c.path)?;
        let before = if target.exists() {
            Some(fs::read(&target)?)
        } else {
            None
        };
        if before.as_ref().map(|b| paths::digest(b)) != c.before {
            return Err("Release input changed or output already exists".into());
        }
        if paths::digest(c.contents.as_bytes()) != c.after {
            return Err("Invalid release replacement digest".into());
        }
        fs::write(stage.path().join(i.to_string()), &c.contents)?;
        originals.push((target, before));
    }
    for (i, (target, _)) in originals.iter().enumerate() {
        let replace = || -> std::io::Result<()> {
            fs::create_dir_all(target.parent().unwrap())?;
            fs::rename(stage.path().join(i.to_string()), target)
        };
        if let Err(error) = replace() {
            for (path, contents) in originals.iter().take(i + 1).rev() {
                match contents {
                    Some(bytes) => fs::write(path, bytes)?,
                    None => {
                        if path.exists() {
                            fs::remove_file(path)?;
                        }
                    }
                }
            }
            return Err(format!("Release replacement failed and was rolled back: {error}").into());
        }
    }
    Ok(())
}
