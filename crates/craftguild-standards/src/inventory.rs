//! Git inventories include untracked sources and refuse unclassified executable code.

use crate::config::{Role, Source};
use craftguild_core::{paths, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn files(root: &Path, changed: Option<&str>) -> Result<BTreeSet<String>> {
    let root = root.canonicalize()?;
    let entry = std::fs::symlink_metadata(root.join(".git"))
        .map_err(|_| "Quality inventory requires Git at the explicit repository root")?;
    if entry.file_type().is_symlink() || !(entry.is_file() || entry.is_dir()) {
        return Err("Invalid explicit-root Git entry".into());
    }
    let mut args = vec![
        "git".into(),
        "ls-files".into(),
        "--cached".into(),
        "--others".into(),
        "--exclude-standard".into(),
        "-z".into(),
    ];
    let mut names = decode(git(&root, &args)?)?;
    if let Some(base) = changed {
        if base.is_empty() || base.starts_with('-') {
            return Err("Expected a Git revision, not an option".into());
        }
        args = vec![
            "git".into(),
            "diff".into(),
            "--name-only".into(),
            "-z".into(),
            base.into(),
            "--".into(),
        ];
        let mut delta = decode(git(&root, &args)?)?;
        delta.extend(decode(git(
            &root,
            &[
                "git".into(),
                "ls-files".into(),
                "--others".into(),
                "--exclude-standard".into(),
                "-z".into(),
            ],
        )?)?);
        names = names.intersection(&delta).cloned().collect();
    }
    names.retain(|name| root.join(name).exists());
    for name in &names {
        paths::owned(&root, name)?;
    }
    Ok(names)
}

fn git(root: &Path, argv: &[String]) -> Result<Vec<u8>> {
    let mut runner = crate::execution_process::Runner::default();
    runner.unset_env.extend(
        [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
            "GIT_INDEX_FILE",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        ]
        .map(str::to_owned),
    );
    let mut command = vec![
        "git".into(),
        "--no-optional-locks".into(),
        "-c".into(),
        "core.fsmonitor=false".into(),
        "--git-dir".into(),
        root.join(".git").to_string_lossy().into(),
        "--work-tree".into(),
        root.to_string_lossy().into(),
    ];
    command.extend_from_slice(&argv[1..]);
    runner.capture(root, &command)
}

fn decode(bytes: Vec<u8>) -> Result<BTreeSet<String>> {
    bytes
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .map(|b| {
            let name = std::str::from_utf8(b)?;
            paths::relative(name)?;
            Ok(name.to_owned())
        })
        .collect()
}

pub fn roles(files: &BTreeSet<String>, sources: &[Source]) -> Result<BTreeMap<String, Role>> {
    let patterns = sources
        .iter()
        .map(|s| Ok((crate::config::glob(&s.pattern)?.compile_matcher(), s.role)))
        .collect::<Result<Vec<_>>>()?;
    let mut roles = BTreeMap::new();
    for name in files {
        let matched: Vec<_> = patterns
            .iter()
            .filter(|(pattern, _)| pattern.is_match(name))
            .collect();
        if matched.len() > 1 {
            return Err(format!("Overlapping source roles: {name}").into());
        }
        if let Some((_, role)) = matched.first() {
            roles.insert(name.clone(), *role);
        } else if executable(name) {
            return Err(format!("Executable source has no declared role: {name}").into());
        }
    }
    Ok(roles)
}

fn executable(name: &str) -> bool {
    Path::new(name)
        .file_name()
        .is_some_and(|p| p == "Makefile" || p == "Dockerfile")
        || matches!(
            Path::new(name).extension().and_then(|p| p.to_str()),
            Some(
                "rs" | "py"
                    | "sh"
                    | "bash"
                    | "zsh"
                    | "js"
                    | "jsx"
                    | "mjs"
                    | "cjs"
                    | "ts"
                    | "tsx"
                    | "mts"
                    | "cts"
                    | "css"
                    | "html"
                    | "svg"
                    | "sql"
                    | "graphql"
                    | "yml"
                    | "yaml"
                    | "go"
                    | "c"
                    | "h"
                    | "cpp"
                    | "hpp"
                    | "java"
                    | "kt"
                    | "swift"
                    | "rb"
                    | "php"
                    | "vue"
                    | "svelte"
            )
        )
}
