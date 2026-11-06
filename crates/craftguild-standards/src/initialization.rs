//! Initialize only the explicit cwd, preserving product-owned configuration.

use crate::{
    init_transaction::{self, Edit},
    repository_pin::{self as pin, MachineIdentity, Reference, Release},
};
use craftguild_core::Result;
use std::{io::Write, path::Path};

const START: &str = "<!-- craftguild:common:start -->";
const END: &str = "<!-- craftguild:common:end -->";
const SKILL: &str = include_str!("../../../templates/local-skill.md");
const STARTER: &str = include_str!("../../../templates/craftguild.toml");
const MAKEFILE: &str = "include standards/make/common.mk\n";

fn agents(existing: &str) -> Result<String> {
    let baseline = pin::BASELINE;
    let block = format!("{START}\nUse Craftguild standards {baseline} with the fixed engine in standards/engine.json.\nRun `make prime` (or `python3 standards/launcher.py prime`) before adoption.\nRead relevant topics with `make prime PRIME_ARGS=\"--topic NAME\"`.\nFollow the repo-local craftguild-standardize Skill. Run `make check`.\n{END}");
    if !existing.contains(START) && !existing.contains(END) {
        return Ok(format!(
            "{block}\n{}{}",
            if existing.is_empty() { "" } else { "\n" },
            existing
        ));
    }
    if existing.matches(START).count() != 1 || existing.matches(END).count() != 1 {
        return Err("Conflicting AGENTS common markers; explicit adaptation required".into());
    }
    let start = existing.find(START).unwrap();
    let end = existing.find(END).unwrap();
    if end < start {
        return Err("Invalid AGENTS common block".into());
    }
    Ok(format!(
        "{}{block}{}",
        &existing[..start],
        &existing[end + END.len()..]
    ))
}

fn refuse_legacy_references(root: &Path) -> Result<()> {
    for name in ["standards/manifest.json", "standards/execution/engine.json"] {
        if init_transaction::read(root, name)?.is_some() {
            return Err(format!("Legacy reference {name} requires explicit migration; init will not create a second pin authority").into());
        }
    }
    Ok(())
}

pub fn plan(root: &Path, reference: Option<Vec<u8>>) -> Result<Vec<Edit>> {
    refuse_legacy_references(root)?;
    let cache = init_transaction::destination(root, ".craftguild")?;
    if cache.exists() && !cache.is_dir() {
        return Err("Repository cache must be a directory".into());
    }
    let mut edits = Vec::new();
    for (name, contents) in [
        ("standards/launcher.py", pin::LAUNCHER),
        ("standards/make/common.mk", pin::MAKE),
        (".agents/skills/craftguild-standardize/SKILL.md", SKILL),
    ] {
        edits.push(Edit::new(root, name, contents.as_bytes().to_vec(), true)?);
    }
    let before = init_transaction::read(root, "AGENTS.md")?;
    let existing = String::from_utf8(before.clone().unwrap_or_default())?;
    edits.push(Edit {
        name: "AGENTS.md".into(),
        before,
        after: agents(&existing)?.into_bytes(),
    });
    for (name, default) in [("craftguild.toml", STARTER), ("Makefile", MAKEFILE)] {
        let contents =
            init_transaction::read(root, name)?.unwrap_or_else(|| default.as_bytes().to_vec());
        edits.push(Edit::new(root, name, contents, true)?);
    }
    let before = init_transaction::read(root, ".gitignore")?;
    let mut ignore = String::from_utf8(before.clone().unwrap_or_default())?;
    if !ignore.lines().any(|line| line == "/.craftguild/") {
        if !ignore.is_empty() && !ignore.ends_with('\n') {
            ignore.push('\n');
        }
        ignore.push_str("/.craftguild/\n");
    }
    edits.push(Edit {
        name: ".gitignore".into(),
        before,
        after: ignore.into_bytes(),
    });
    if let Some(bytes) = reference {
        edits.push(Edit::new(root, pin::REFERENCE, bytes, true)?);
    }
    Ok(edits)
}

fn launch(root: &Path, entry: &Path, operation: &str) -> Result<Vec<u8>> {
    crate::command::capture(
        root,
        &[
            "python3".into(),
            entry.to_string_lossy().into(),
            operation.into(),
        ],
    )
}

pub fn run(args: &[String], identity: &MachineIdentity, output: &mut impl Write) -> Result<()> {
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        writeln!(output, "Usage: craftguild-standards init\nInitialize cwd using a clean published engine; existing pins are retained.")?;
        return Ok(());
    }
    if !args.is_empty() {
        return Err("Usage: craftguild-standards init".into());
    }
    let root = std::env::current_dir()?.canonicalize()?;
    refuse_legacy_references(&root)?;
    crate::execution_process::initialize_signals()?;
    let existing = init_transaction::read(&root, pin::REFERENCE)?;
    if existing.is_some() {
        let reference = pin::load(&root).map_err(|e| format!("Existing fixed reference is incompatible: {e}. Explicit migration/adaptation required"))?;
        reference.verify_entries(&root)?;
        if identity.matches(&reference).is_err() {
            let launcher = root.join("standards/launcher.py");
            launch(&root, &launcher, "setup")?;
            output.write_all(&launch(&root, &launcher, "init")?)?;
            return Ok(());
        }
    }
    // Discover all local conflicts before any release lookup or acquisition.
    plan(&root, None)?;
    let bytes = match existing {
        Some(bytes) => {
            let reference: Reference = serde_json::from_slice(&bytes)?;
            if reference.entries != Reference::entry_hashes() {
                return Err(
                    "Fixed entry belongs to another implementation; explicit migration required"
                        .into(),
                );
            }
            bytes
        }
        None => {
            if !identity.clean {
                return Err("New init requires a clean published engine; unpublished source cannot create a public pin".into());
            }
            crate::release::versions::stable(&identity.engine.version)?;
            let url = format!("https://github.com/craftguild/standards/releases/download/v{0}/craftguild-release-v{0}.json", identity.engine.version);
            let bytes = crate::command::capture(
                &root,
                &[
                    "curl".into(),
                    "-fsSL".into(),
                    "--proto".into(),
                    "=https".into(),
                    "--proto-redir".into(),
                    "=https".into(),
                    url,
                ],
            )?;
            let release: Release = serde_json::from_slice(&bytes)?;
            release.validate()?;
            let reference = Reference::new(release);
            identity.matches(&reference)?;
            format!("{}\n", serde_json::to_string_pretty(&reference)?).into_bytes()
        }
    };
    let edits = plan(&root, Some(bytes))?;
    init_transaction::publish_with_finish(
        &root,
        &edits,
        |stage| {
            launch(&root, &stage.join("standards/launcher.py"), "setup")?;
            Ok(())
        },
        |_| crate::execution_process::Runner::default().check(),
        || {
            writeln!(output, "Initialized fixed Craftguild entry in cwd. Read make prime; finish product configuration, then make check and artifact acceptance.\nExisting Makefile recipes are preserved: include standards/make/common.mk and wire its targets where needed.\nCargo/npm declarations and product source remain your adoption work.")?;
            output.flush()?;
            Ok(())
        },
    )?;

    Ok(())
}

pub fn prime(args: &[String], output: &mut impl Write) -> Result<()> {
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        writeln!(output, "Usage: craftguild-standards prime [--topic NAME]")?;
        return Ok(());
    }
    let root = std::env::current_dir()?.canonicalize()?;
    let reference = pin::load(&root).map_err(|e| {
        format!("Fixed reference missing/invalid: {e}. Run init or explicitly adapt the reference")
    })?;
    reference.verify_entries(&root)?;
    let mut argv = vec![
        "python3".into(),
        root.join("standards/launcher.py").to_string_lossy().into(),
        "prime".into(),
    ];
    argv.extend_from_slice(args);
    output.write_all(&crate::command::capture(&root, &argv)?)?;
    Ok(())
}
