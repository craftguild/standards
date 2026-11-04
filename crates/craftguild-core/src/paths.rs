//! Repository-relative paths and exact, non-symlink file inventories.

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

/// A portable relative path with no traversal, alias spelling or empty component.
pub fn relative(name: &str) -> crate::Result<&Path> {
    let path = Path::new(name);
    if name.is_empty()
        || name.contains('\\')
        || name.contains(':')
        || name
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
        || !path.components().all(|p| matches!(p, Component::Normal(_)))
    {
        return Err(format!("Unsafe repository-relative path: {name}").into());
    }
    Ok(path)
}

/// Resolve an existing regular file/directory without traversing any symlink.
pub fn owned(root: &Path, name: &str) -> crate::Result<PathBuf> {
    let mut path = root.canonicalize()?;
    for part in relative(name)?.components() {
        path.push(part);
        if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(format!("Symlink in owned path: {name}").into());
        }
    }
    Ok(path)
}

/// Hash bytes using the common input/asset digest algorithm.
pub fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

/// Hash a required, regular, repository-owned file.
pub fn hash_file(root: &Path, name: &str) -> crate::Result<String> {
    let path = owned(root, name)?;
    if !path.is_file() {
        return Err(format!("Required input is not a regular file: {name}").into());
    }
    Ok(digest(&std::fs::read(path)?))
}

/// Inventory every regular file in a declared ownership tree, including ignored files.
pub fn inventory(root: &Path, directory: &str) -> crate::Result<BTreeSet<String>> {
    fn visit(root: &Path, path: &Path, files: &mut BTreeSet<String>) -> crate::Result<()> {
        let kind = std::fs::symlink_metadata(path)?.file_type();
        if kind.is_symlink() {
            return Err(format!("Symlink in owned inventory: {}", path.display()).into());
        }
        if kind.is_dir() {
            for entry in std::fs::read_dir(path)? {
                visit(root, &entry?.path(), files)?;
            }
        } else if kind.is_file() {
            let name = path.strip_prefix(root)?.to_str().ok_or("Non-UTF8 path")?;
            relative(name)?;
            files.insert(name.to_owned());
        } else {
            return Err("Nonregular entry in owned inventory".into());
        }
        Ok(())
    }
    let root = root.canonicalize()?;
    let path = owned(&root, directory)?;
    if !path.is_dir() {
        return Err(format!("Inventory root must be a directory: {directory}").into());
    }
    let mut files = BTreeSet::new();
    visit(&root, &path, &mut files)?;
    Ok(files)
}
