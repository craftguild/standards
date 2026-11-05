//! Observe an explicit application's build without inheriting parent repository data.

use crate::process::output;
use craftguild_core::{BuildInfo, Dirty};
use std::path::Path;
use std::process::Command;

const GIT_ROUTING: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
];

/// Capture compiler and source facts for preparation or a Cargo build script.
pub fn capture(
    root: &Path,
    command: &str,
    version: &str,
    target: &str,
    profile: &str,
) -> BuildInfo {
    let (commit, dirty) = repository(root);
    let compiler = output(
        Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .env("RUSTUP_AUTO_INSTALL", "0")
            .arg("--version")
            .current_dir(root),
    )
    .and_then(|b| String::from_utf8(b).ok())
    .map(|s| s.trim().to_owned())
    .filter(|s| !s.is_empty() && s.len() <= 512 && s.bytes().all(|b| (32..=126).contains(&b)))
    .unwrap_or_else(|| "unknown".into());
    BuildInfo {
        command: command.into(),
        version: version.into(),
        commit,
        dirty,
        compiler,
        target: target.into(),
        profile: profile.into(),
    }
}

/// Clean, dirty and unavailable are separate observations. Archives stay unknown.
pub fn repository(root: &Path) -> (String, Dirty) {
    let unknown = || ("unknown".into(), Dirty::Unknown);
    let Ok(root) = root.canonicalize() else {
        return unknown();
    };
    let Ok(entry) = std::fs::symlink_metadata(root.join(".git")) else {
        return unknown();
    };
    if !(entry.is_dir() || entry.is_file()) || entry.file_type().is_symlink() {
        return unknown();
    }
    let git = || {
        let mut cmd = Command::new("git");
        cmd.current_dir(&root)
            .args([
                "--no-optional-locks",
                "-c",
                "core.fsmonitor=false",
                "--git-dir",
            ])
            .arg(root.join(".git"))
            .arg("--work-tree")
            .arg(&root);
        for key in GIT_ROUTING {
            cmd.env_remove(key);
        }
        cmd
    };
    let commit = output(git().args(["rev-parse", "--verify", "HEAD^{commit}"]))
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| matches!(s.len(), 40 | 64) && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .unwrap_or_else(|| "unknown".into());
    let dirty = output(git().args([
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=normal",
        "--ignore-submodules=none",
    ]))
    .map_or(Dirty::Unknown, |b| {
        if b.is_empty() {
            Dirty::Clean
        } else {
            Dirty::Dirty
        }
    });
    (commit, dirty)
}
