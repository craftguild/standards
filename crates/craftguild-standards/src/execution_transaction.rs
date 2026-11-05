//! Recoverable file replacement and exclusive repository mutation ownership.

use crate::distribution::atomic_write;
use craftguild_core::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Lock(PathBuf);

impl Lock {
    pub fn acquire(root: &Path) -> Result<Self> {
        crate::distribution::owned_directory(root, ".craftguild")?;
        let path = root.join(".craftguild/execution.lock");
        if path.exists() {
            safe(&path)?;
            let pid: u32 = std::fs::read_to_string(&path)?.trim().parse()?;
            #[cfg(unix)]
            {
                let observed = std::process::Command::new("ps")
                    .args(["-p", &pid.to_string(), "-o", "pid="])
                    .output()?;
                if observed.status.code() != Some(1) {
                    return Err("Execution owner is live or cannot be verified terminal".into());
                }
            }
            #[cfg(not(unix))]
            return Err(format!(
                "Verify interrupted execution owner {pid} before removing its lock"
            )
            .into());
            let recovery_path = root.join(".craftguild/recovery.lock");
            let _recovery_file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&recovery_path)?;
            let recovery_lock = Self(recovery_path);
            recover(root)?;
            std::fs::remove_file(&path)?;
            drop(recovery_lock);
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| {
                format!(
                    "Cannot acquire execution lock (another invocation or interrupted run): {e}"
                )
            })?;
        writeln!(file, "{}", std::process::id())?;
        file.sync_all()?;
        let lock = Self(path);
        recover(root)?;
        Ok(lock)
    }
}

fn recover(root: &Path) -> Result<()> {
    let config = crate::config::load(root)?;
    for artifact in &config.artifacts {
        let runs = root.join(format!(".craftguild/{}/runs", artifact.command));
        if !runs.exists() {
            continue;
        }
        safe(&runs)?;
        for run in std::fs::read_dir(&runs)? {
            let directory = run?.path();
            let manifest = directory.join("rollback/snapshot.json");
            if !manifest.exists() {
                continue;
            }
            safe(&manifest)?;
            let mut snapshot: Snapshot = serde_json::from_slice(&std::fs::read(&manifest)?)?;
            if !snapshot.active {
                continue;
            }
            verify_terminal_children(&directory)?;
            let result = directory.join("result.json");
            safe(&result)?;
            let mut record: crate::execution_record::Record =
                serde_json::from_slice(&std::fs::read(&result)?)?;
            let mut allowed = record.context.outputs.clone();
            for name in ["prepared.json", "bom.json", "receipt.json"] {
                allowed.push(root.join(format!(".craftguild/{}/{name}", artifact.command)));
            }
            if let Some(bundle) = &artifact.bundle {
                allowed.extend([root.join(&bundle.directory), root.join(&bundle.metafile)]);
            }
            if let Some(build) = &artifact.build {
                for hook in &build.hooks {
                    allowed.extend(
                        hook.outputs
                            .iter()
                            .chain(&hook.output_dirs)
                            .map(|p| root.join(p)),
                    );
                }
            }
            for hook in config
                .operations
                .iter()
                .flatten()
                .filter(|op| op.artifact == artifact.id)
                .flat_map(|op| &op.hooks)
            {
                allowed.extend(
                    hook.outputs
                        .iter()
                        .chain(&hook.output_dirs)
                        .map(|p| root.join(p)),
                );
            }
            if snapshot.entries.iter().any(|e| {
                !allowed.contains(&e.path)
                    || e.backup
                        .as_ref()
                        .is_some_and(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
            }) {
                return Err("Interrupted snapshot differs from declared output ownership".into());
            }
            snapshot.directory = directory.join("rollback");
            snapshot.restore()?;
            record.directory = directory;
            record.finish("interrupted")?;
        }
    }
    Ok(())
}

fn verify_terminal_children(directory: &Path) -> Result<()> {
    let active = directory.join("process.active");
    if active.exists() {
        let pid: u32 = std::fs::read_to_string(&active)?.trim().parse()
            .map_err(|_| "Subprocess scope was interrupted before PID recording; verify termination before recovery")?;
        #[cfg(unix)]
        if crate::execution_process::group_live(pid)? {
            return Err(
                "Interrupted subprocess group is still live; recovery cannot restore outputs yet"
                    .into(),
            );
        }
        #[cfg(not(unix))]
        return Err(
            format!("Verify interrupted process {pid} before recovery on this platform").into(),
        );
    }
    let log = directory.join("process.jsonl");
    if !log.exists() {
        return Ok(());
    }
    let text = std::fs::read_to_string(log)?;
    let mut events: Vec<serde_json::Value> = vec![];
    for line in text.split_inclusive('\n') {
        if !line.ends_with('\n') {
            // An interrupted append is not a committed event. Only the scoped
            // PID marker checked above can authorize recovery past that tail.
            if active.exists() {
                break;
            }
            return Err(
                "Incomplete process event without termination evidence; verify before recovery"
                    .into(),
            );
        }
        events.push(serde_json::from_str(line)?);
    }
    #[cfg(unix)]
    {
        for pid in events.iter().filter_map(|v| v["pid"].as_u64()) {
            if crate::execution_process::group_live(pid.try_into()?)? {
                return Err(
                "Interrupted subprocess group is still live; recovery cannot restore outputs yet"
                    .into(),
            );
            }
        }
    }
    #[cfg(not(unix))]
    if events.iter().any(|v| v["pid"].is_number()) {
        return Err("Verify interrupted subprocesses before recovery on this platform".into());
    }
    Ok(())
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: PathBuf,
    backup: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    entries: Vec<Entry>,
    active: bool,
    #[serde(skip)]
    directory: PathBuf,
}

impl Snapshot {
    pub fn create(directory: &Path) -> Result<Self> {
        let directory = directory.join("rollback");
        std::fs::create_dir(&directory)?;
        let snapshot = Self {
            entries: vec![],
            active: true,
            directory,
        };
        snapshot.save()?;
        Ok(snapshot)
    }

    pub fn capture(&mut self, path: &Path) -> Result<()> {
        safe(path)?;
        if self.entries.iter().any(|e| e.path == path) {
            return Ok(());
        }
        let backup = if path.exists() {
            let name = self.entries.len().to_string();
            copy(path, &self.directory.join(&name))?;
            Some(name)
        } else {
            None
        };
        self.entries.push(Entry {
            path: path.into(),
            backup,
        });
        self.save()
    }

    pub fn clear(&mut self, path: &Path) -> Result<()> {
        self.capture(path)?;
        remove(path)
    }

    pub fn write(&mut self, path: &Path, bytes: &[u8]) -> Result<()> {
        self.capture(path)?;
        std::fs::create_dir_all(path.parent().ok_or("Output has no parent")?)?;
        atomic_write(path, bytes)
    }

    pub fn publish(&mut self, source: &Path, destination: &Path) -> Result<()> {
        self.capture(destination)?;
        std::fs::create_dir_all(destination.parent().ok_or("Output has no parent")?)?;
        atomic_write(destination, &std::fs::read(source)?)?;
        std::fs::set_permissions(destination, std::fs::metadata(source)?.permissions())?;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
        self.active = false;
        self.save()
    }

    pub fn restore(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        verify_terminal_children(self.directory.parent().ok_or("Missing run directory")?)?;
        for entry in self.entries.iter().rev() {
            safe(&entry.path)?;
            remove(&entry.path)?;
            if let Some(backup) = &entry.backup {
                copy(&self.directory.join(backup), &entry.path)?;
            }
        }
        let active = self
            .directory
            .parent()
            .ok_or("Missing run directory")?
            .join("process.active");
        if active.exists() {
            std::fs::remove_file(active)?;
        }
        self.finish()
    }

    fn save(&self) -> Result<()> {
        atomic_write(
            &self.directory.join("snapshot.json"),
            &serde_json::to_vec_pretty(self)?,
        )
    }
}

fn safe(path: &Path) -> Result<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("Transaction paths must be absolute without traversal".into());
    }
    if path.ancestors().any(|p| p.is_symlink()) {
        return Err("Symlink in transactional output".into());
    }
    Ok(())
}

fn remove(path: &Path) -> Result<()> {
    safe(path)?;
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn copy(source: &Path, destination: &Path) -> Result<()> {
    safe(source)?;
    safe(destination)?;
    if source.is_dir() {
        std::fs::create_dir_all(destination)?;
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            copy(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if source.is_file() {
        std::fs::create_dir_all(destination.parent().ok_or("Copy has no parent")?)?;
        std::fs::copy(source, destination)?;
    } else {
        return Err("Transaction owns only regular files/directories".into());
    }
    Ok(())
}
