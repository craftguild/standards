//! Receipt-backed release assets; no compiler, dependency resolver or bootstrap.

use crate::{
    config::{self, Config},
    execution_context::{absolute, Context, Options},
    execution_record::Receipt,
    release::versions,
};
use craftguild_cli_build::observe;
use craftguild_core::{paths, Dirty, Result};
use std::{collections::BTreeSet, path::Path};

pub fn run(args: &[String]) -> Result<()> {
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        println!("Usage: package [--group ID] --output DIR");
        return Ok(());
    }
    let mut group = None;
    let mut output = None;
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--group" if group.is_none() => {
                group = Some(args.next().ok_or("Missing group")?.as_str())
            }
            "--output" if output.is_none() => {
                output = Some(args.next().ok_or("Missing output")?.as_str())
            }
            _ => return Err("Invalid package arguments; use --help".into()),
        }
    }
    let root = std::env::current_dir()?.canonicalize()?;
    let output = absolute(&root, output.ok_or("Package requires --output DIR")?)?;
    package(&root, &config::load(&root)?, group, &output)
}

pub fn package(root: &Path, config: &Config, selected: Option<&str>, output: &Path) -> Result<()> {
    if output.exists() || output.is_symlink() {
        return Err("Package destination must be fresh".into());
    }
    let groups: BTreeSet<_> = config
        .installers
        .iter()
        .flatten()
        .map(|i| i.release_group.as_str())
        .collect();
    let id = match selected {
        Some(id) if groups.contains(id) => id,
        None if groups.len() == 1 => groups.iter().next().unwrap(),
        _ => return Err("Select an installer release group with --group ID".into()),
    };
    let group = config
        .releases
        .iter()
        .find(|g| g.id == id)
        .ok_or("Unknown group")?;
    let version = versions::current(root, group)?;
    let (revision, dirty) = observe::repository(root);
    if dirty != Dirty::Clean || !matches!(revision.len(), 40 | 64) {
        return Err("Package requires a clean explicit Git root and full HEAD revision".into());
    }
    let mut target = None;
    let mut assets = std::collections::BTreeMap::new();
    for installer in config
        .installers
        .iter()
        .flatten()
        .filter(|i| i.release_group == id)
    {
        let artifact = config.artifact(&installer.artifact)?;
        let receipt: Receipt = serde_json::from_slice(&std::fs::read(paths::owned(
            root,
            &format!(".craftguild/{}/receipt.json", artifact.command),
        )?)?)?;
        let build = &receipt.prepared.build;
        if build.dirty != Dirty::Clean
            || build.profile != "release"
            || build.version != version
            || build.commit != revision
            || target.as_ref().is_some_and(|t| t != &build.target)
        {
            return Err(
                "Package requires matching clean release version/target/revision receipts".into(),
            );
        }
        let mut options = Options {
            offline: true,
            standard_layout: config.rust_output_layout.is_some(),
            fallback_target: Some(receipt.context.target.clone()),
            fallback_explicit_target: receipt.context.explicit_target,
            known_host: receipt.context.host.clone(),
            known_compiler: Some(receipt.context.compiler_version.clone()),
            ..Options::default()
        };
        for (key, value) in [
            ("variant", receipt.context.variant.clone()),
            ("toolchain", receipt.context.toolchain.clone()),
            ("profile", Some("release".into())),
            ("features", Some(receipt.context.features.join(","))),
            (
                "no-default-features",
                Some(receipt.context.no_default_features.to_string()),
            ),
            (
                "target-dir",
                Some(
                    receipt
                        .context
                        .target_dir
                        .to_str()
                        .ok_or("Non-UTF8 target directory")?
                        .into(),
                ),
            ),
        ] {
            if let Some(value) = value {
                options.values.insert(key.into(), value);
            }
        }
        if Context::resolve(root, artifact, &options)? != receipt.context {
            return Err("Receipt context differs from configured artifact output".into());
        }
        receipt.verify(root)?;
        let binary = std::fs::read(&receipt.context.output)?;
        if receipt.outputs.get(&receipt.context.file) != Some(&paths::digest(&binary)) {
            return Err("Output changed during package verification".into());
        }
        let bom = receipt
            .prepared
            .verify(root, build, &receipt.prepared.features)?;
        target = Some(build.target.clone());
        let name = format!("{}-v{version}-{}", artifact.command, build.target);
        assets.insert(name.clone(), binary);
        assets.insert(format!("{name}.bom.json"), bom);
    }
    let checksums: String = assets
        .iter()
        .map(|(name, bytes)| format!("{}  {name}\n", paths::digest(bytes)))
        .collect();
    let parent = output.parent().ok_or("Package output has no parent")?;
    std::fs::create_dir_all(parent)?;
    let stage = tempfile::Builder::new()
        .prefix("package-")
        .tempdir_in(parent)?;
    for (name, bytes) in assets {
        let path = stage.path().join(name);
        std::fs::write(&path, bytes)?;
        #[cfg(unix)]
        if !path.to_string_lossy().ends_with(".bom.json") {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
        }
    }
    std::fs::write(
        stage.path().join(format!(
            "SHA256SUMS-{}.txt",
            target.ok_or("No package artifacts")?
        )),
        checksums,
    )?;
    // Recheck the explicit source root before publishing the complete directory.
    if !checkout_matches(root, &revision, stage.path())? || output.exists() || output.is_symlink() {
        return Err("Checkout or destination changed during package verification".into());
    }
    std::fs::rename(stage.path(), output)?;
    println!("{}", output.display());
    Ok(())
}

/// Ignore only this invocation's owned stage, never its parent or sibling stages.
fn checkout_matches(root: &Path, revision: &str, stage: &Path) -> Result<bool> {
    let (head, dirty) = observe::repository(root);
    if head != revision || dirty == Dirty::Unknown {
        return Ok(false);
    }
    if dirty == Dirty::Clean {
        return Ok(true);
    }
    let Ok(relative) = stage.strip_prefix(root) else {
        return Ok(false);
    };
    let relative = relative.to_str().ok_or("Non-UTF8 package stage")?;
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
    let mut argv: Vec<String> = ["git", "--no-optional-locks", "-c", "core.fsmonitor=false"]
        .map(str::to_owned)
        .into();
    for (flag, path) in [
        ("--git-dir", root.join(".git")),
        ("--work-tree", root.to_owned()),
    ] {
        argv.extend([
            flag.into(),
            path.to_str().ok_or("Non-UTF8 Git root")?.into(),
        ]);
    }
    argv.extend(
        [
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
            "--",
            ".",
        ]
        .map(str::to_owned),
    );
    argv.push(format!(":(top,literal,exclude){relative}"));
    Ok(runner.capture(root, &argv)?.is_empty())
}
