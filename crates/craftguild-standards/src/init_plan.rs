//! Generated-file ownership, prior baselines and the complete optimistic write set.
use crate::{
    init_baseline as legacy,
    init_transaction::{self, Edit},
    repository_pin::{self as pin, Reference},
};
use craftguild_core::{paths, Result};
use std::{collections::BTreeMap, path::Path};

const START: &str = "<!-- craftguild:common:start -->";
const END: &str = "<!-- craftguild:common:end -->";
const SKILL: &str = include_str!("../../../templates/local-skill.md");
const STARTER: &str = include_str!("../../../templates/craftguild.toml");
const SKILL_PATH: &str = ".agents/skills/craftguild-standardize/SKILL.md";

fn block() -> String {
    format!("{START}\nUse Craftguild standards {} with the fixed engine in standards/engine.json.\nRun `make prime` (or `python3 standards/launcher.py prime`) before adoption.\nRead relevant topics with `make prime PRIME_ARGS=\"--topic NAME\"`.\nFollow the repo-local craftguild-standardize Skill. Run `make check`.\nUpdate explicitly with `craftguild-standards init [--force]`; follow its dependency revision guidance.\n{END}", pin::BASELINE)
}

pub fn managed_hashes() -> BTreeMap<String, String> {
    [
        (SKILL_PATH.into(), paths::digest(SKILL.as_bytes())),
        ("AGENTS.md".into(), paths::digest(block().as_bytes())),
    ]
    .into()
}

fn block_range(existing: &str) -> Result<Option<std::ops::Range<usize>>> {
    if !existing.contains(START) && !existing.contains(END) {
        return Ok(None);
    }
    if existing.matches(START).count() != 1 || existing.matches(END).count() != 1 {
        return Err("Conflicting AGENTS common markers; explicit adaptation required".into());
    }
    let start = existing.find(START).unwrap();
    let end = existing.find(END).unwrap();
    if end < start {
        return Err("Invalid AGENTS common block".into());
    }
    Ok(Some(start..end + END.len()))
}

pub fn refuse_legacy_references(root: &Path) -> Result<()> {
    for name in ["standards/manifest.json", "standards/execution/engine.json"] {
        if init_transaction::read(root, name)?.is_some() {
            return Err(format!("Legacy reference {name} requires explicit migration; init will not create a second pin authority").into());
        }
    }
    Ok(())
}

/// Validate the old baseline before transport, even when no content will change.
fn baselines(previous: Option<&Reference>) -> Result<BTreeMap<String, String>> {
    let Some(previous) = previous else {
        return Ok(BTreeMap::new());
    };
    previous.validate()?;
    if previous.schema_version == 2 {
        if previous.entries
            != BTreeMap::from([
                ("launcher.py".into(), legacy::LAUNCHER.into()),
                ("make/common.mk".into(), legacy::MAKE.into()),
            ])
        {
            return Err("Unknown previous managed baseline; explicit adaptation required".into());
        }
        Ok([
            (SKILL_PATH.into(), legacy::SKILL.into()),
            ("AGENTS.md".into(), legacy::AGENTS.into()),
        ]
        .into())
    } else {
        Ok(previous.managed.clone())
    }
}

pub fn plan(root: &Path, reference: Option<Vec<u8>>) -> Result<Vec<Edit>> {
    refuse_legacy_references(root)?;
    let cache = init_transaction::destination(root, ".craftguild")?;
    if cache.exists() && !cache.is_dir() {
        return Err("Repository cache must be a directory".into());
    }
    let previous_bytes = init_transaction::read(root, pin::REFERENCE)?;
    let previous: Option<Reference> = previous_bytes
        .as_deref()
        .map(serde_json::from_slice)
        .transpose()?;
    let baseline = baselines(previous.as_ref())?;
    if let Some(previous) = &previous {
        previous.verify_entries(root)?;
    }
    let mut edits = Vec::new();
    for (name, contents, expected) in [
        (
            "standards/launcher.py",
            pin::LAUNCHER,
            previous.as_ref().map(|p| p.entries["launcher.py"].as_str()),
        ),
        (
            "standards/make/common.mk",
            pin::MAKE,
            previous
                .as_ref()
                .map(|p| p.entries["make/common.mk"].as_str()),
        ),
        (
            SKILL_PATH,
            SKILL,
            baseline.get(SKILL_PATH).map(String::as_str),
        ),
    ] {
        let before = init_transaction::read(root, name)?;
        if let Some(expected) = expected {
            if before.as_deref().map(paths::digest).as_deref() != Some(expected) {
                return Err(format!(
                    "Conflicting managed file {name}; explicit adaptation required"
                )
                .into());
            }
        } else if before.as_deref().is_some_and(|b| b != contents.as_bytes()) {
            return Err(
                format!("Unowned managed file {name}; explicit adaptation required").into(),
            );
        }
        edits.push(Edit {
            name: name.into(),
            before,
            after: contents.as_bytes().to_vec(),
        });
    }
    let before = init_transaction::read(root, "AGENTS.md")?;
    let existing = String::from_utf8(before.clone().unwrap_or_default())?;
    let range = block_range(&existing)?;
    if let Some(expected) = baseline.get("AGENTS.md") {
        if range
            .as_ref()
            .map(|r| paths::digest(existing[r.clone()].as_bytes()))
            .as_ref()
            != Some(expected)
        {
            return Err(
                "Conflicting managed AGENTS common block; explicit adaptation required".into(),
            );
        }
    } else if range
        .as_ref()
        .is_some_and(|r| existing[r.clone()] != block())
    {
        return Err("Unowned AGENTS common block; explicit adaptation required".into());
    }
    let after = match range {
        Some(r) => format!("{}{}{}", &existing[..r.start], block(), &existing[r.end..]),
        None => format!(
            "{}\n{}{}",
            block(),
            if existing.is_empty() { "" } else { "\n" },
            existing
        ),
    };
    edits.push(Edit {
        name: "AGENTS.md".into(),
        before,
        after: after.into_bytes(),
    });
    for (name, default) in [
        ("craftguild.toml", STARTER),
        ("Makefile", "include standards/make/common.mk\n"),
    ] {
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
    // Include the previous pin in every observed set, and always publish it last.
    if let Some(after) = reference.or(previous_bytes.clone()) {
        edits.push(Edit {
            name: pin::REFERENCE.into(),
            before: previous_bytes,
            after,
        });
    }
    Ok(edits)
}
