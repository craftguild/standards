//! Optimistic, exclusive initialization writes with ownership-aware rollback.

use craftguild_core::{paths, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn destination(root: &Path, name: &str) -> Result<PathBuf> {
    let mut path = root.canonicalize()?;
    for component in paths::relative(name)?.components() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("Symlink in initialization path".into())
            }
            Ok(meta) if !meta.is_file() && !meta.is_dir() => {
                return Err("Nonregular initialization path".into())
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(path)
}

pub fn read(root: &Path, name: &str) -> Result<Option<Vec<u8>>> {
    let path = destination(root, name)?;
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[derive(Clone)]
pub struct Edit {
    pub name: String,
    pub before: Option<Vec<u8>>,
    pub after: Vec<u8>,
}

impl Edit {
    pub fn new(root: &Path, name: &str, after: Vec<u8>, managed: bool) -> Result<Self> {
        let before = read(root, name)?;
        if managed && before.as_ref().is_some_and(|bytes| bytes != &after) {
            return Err(
                format!("Conflicting managed file {name}; explicit adaptation required").into(),
            );
        }
        Ok(Self {
            name: name.into(),
            before,
            after,
        })
    }

    fn check(&self, root: &Path) -> Result<()> {
        if read(root, &self.name)? != self.before {
            return Err(format!("Initialization input changed concurrently: {}", self.name).into());
        }
        Ok(())
    }
}

struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn parents(root: &Path, name: &str, created: &mut Vec<PathBuf>) -> Result<()> {
    let parent = Path::new(name).parent().unwrap();
    let mut relative = PathBuf::new();
    for component in parent.components() {
        relative.push(component);
        let path = destination(
            root,
            relative.to_str().ok_or("Non-UTF8 initialization path")?,
        )?;
        if !path.exists() {
            fs::create_dir(&path)?;
            created.push(path);
        }
    }
    Ok(())
}

/// The preparation callback acquires/verifies the engine from staged entry files.
/// The observation callback supplies deterministic interruption/race tests.
pub fn publish(
    root: &Path,
    edits: &[Edit],
    prepare: impl FnOnce(&Path) -> Result<()>,
    observe: impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    publish_with_finish(root, edits, prepare, observe, || Ok(()))
}

pub fn publish_with_finish(
    root: &Path,
    edits: &[Edit],
    prepare: impl FnOnce(&Path) -> Result<()>,
    mut observe: impl FnMut(usize) -> Result<()>,
    finish: impl FnOnce() -> Result<()>,
) -> Result<()> {
    for edit in edits {
        edit.check(root)?;
    }
    let control = destination(root, ".craftguild")?;
    fs::create_dir_all(&control)?;
    let lock = destination(root, ".craftguild/init.lock")?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .map_err(|e| format!("Cannot lock initialization: {e}"))?;
    let _lock = Lock(lock);
    let stage = tempfile::Builder::new()
        .prefix("init-")
        .tempdir_in(control)?;
    for edit in edits {
        let path = stage.path().join(&edit.name);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, &edit.after)?;
        if edit.before.is_some() {
            fs::set_permissions(
                path,
                fs::metadata(destination(root, &edit.name)?)?.permissions(),
            )?;
        }
    }
    prepare(stage.path())?;
    let mut written: Vec<&Edit> = Vec::new();
    let mut created = Vec::new();
    let result = (|| -> Result<()> {
        for (index, edit) in edits.iter().enumerate() {
            observe(index)?;
            // Recheck the entire observed set, accounting for our own writes.
            for observed in edits {
                let expected = if written.iter().any(|e| e.name == observed.name) {
                    Some(&observed.after)
                } else {
                    observed.before.as_ref()
                };
                if read(root, &observed.name)?.as_ref() != expected {
                    return Err(format!(
                        "Initialization input changed concurrently: {}",
                        observed.name
                    )
                    .into());
                }
            }
            if edit.before.as_ref() == Some(&edit.after) {
                continue;
            }
            let candidate = stage.path().join(&edit.name);
            if fs::read(&candidate)? != edit.after {
                return Err("Staged initialization bytes changed".into());
            }
            parents(root, &edit.name, &mut created)?;
            let target = destination(root, &edit.name)?;
            if edit.before.is_none() {
                // Create-only publication must not replace a concurrent new file.
                fs::hard_link(candidate, target)?;
            } else {
                fs::rename(candidate, target)?;
            }
            written.push(edit);
        }
        finish()
    })();
    if let Err(error) = result {
        let mut conflicts = Vec::new();
        for edit in written.into_iter().rev() {
            if read(root, &edit.name).ok().flatten().as_ref() != Some(&edit.after) {
                conflicts.push(edit.name.clone());
                continue;
            }
            let path = destination(root, &edit.name)?;
            match &edit.before {
                Some(before) => fs::write(path, before)?,
                None => fs::remove_file(path)?,
            }
        }
        for directory in created.into_iter().rev() {
            let _ = fs::remove_dir(directory);
        }
        return Err(format!(
            "{error}; initialization rolled back owned writes; preserved concurrent files: {}",
            conflicts.join(", ")
        )
        .into());
    }
    Ok(())
}
