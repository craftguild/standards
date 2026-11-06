//! Local release planning and application; never commits, tags or publishes.

pub mod conventional;
pub mod files;
pub mod history;
pub mod versions;

use conventional::Bump;
use craftguild_core::{paths, Result};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Normal,
    ExplicitMajor,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema_version: u32,
    pub group: String,
    pub config_hash: String,
    pub head: String,
    pub base: Option<String>,
    pub current: String,
    pub next: Option<String>,
    pub tag: String,
    pub bump: Bump,
    #[serde(default)]
    pub mode: Mode,
    pub prepared: bool,
    pub receipt_path: Option<String>,
    pub notes: String,
    pub changes: Vec<files::Change>,
}

pub fn plan(root: &Path, group: &versions::Group) -> Result<Plan> {
    plan_with_mode(root, group, Mode::Normal)
}

pub fn plan_with_mode(root: &Path, group: &versions::Group, mode: Mode) -> Result<Plan> {
    if !root.join(".git").exists() {
        return Err("Release requires an explicit Git repository root".into());
    }
    if let Some(pending) = files::pending(root, group, mode)? {
        return Ok(pending);
    }
    if !history::dirty(root)?.is_empty() {
        return Err("Release preparation requires a clean working tree".into());
    }
    let head = history::git(root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let baseline = history::baseline(root, group, &head)?;
    let base = baseline.as_ref().map(|(tag, _)| tag.clone());
    let current = versions::current(root, group)?;
    if baseline.as_ref().is_some_and(|(_, v)| v != &current)
        || (baseline.is_none() && current != group.initial_version)
    {
        return Err("Version authority disagrees with release baseline; resolve pending/manual version edits".into());
    }
    let (inferred, _) = history::notes(root, group, base.as_deref(), &head, &current)?;
    let (bump, next) = versions::select(&current, inferred, base.is_none(), mode)?;
    let release = base.is_none() || bump != Bump::None;
    let tag = format!("{}{next}", group.tag_prefix);
    let mut changes = Vec::new();
    let mut notes = String::new();
    let mut receipt_path = None;
    if release {
        if !history::git(root, &["tag", "--list", &tag])?.is_empty() {
            return Err("Target tag already exists; never overwrite releases".into());
        }
        notes = history::notes(root, group, base.as_deref(), &head, &next)?.1;
        changes = versions::changes(root, group, &current, &next)?;
        changes.retain(|c| c.before.as_ref() != Some(&c.after));
        changes.push(files::Change::new(
            format!("{}/{tag}.md", group.notes),
            notes.clone(),
        ));
        receipt_path = Some(format!("{}/{tag}.json", group.notes));
    }
    Ok(Plan {
        schema_version: 2,
        group: group.id.clone(),
        config_hash: paths::digest(&serde_json::to_vec(group)?),
        head,
        base,
        current,
        next: release.then_some(next),
        tag,
        bump,
        mode,
        prepared: false,
        receipt_path,
        notes,
        changes,
    })
}

fn options(args: &[String]) -> Result<(Option<&str>, bool, bool, Mode)> {
    let mut group = None;
    let mut apply = false;
    let mut json = false;
    let mut mode = Mode::Normal;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--group" if group.is_none() => {
                i += 1;
                group = Some(args.get(i).ok_or("Missing release group")?.as_str());
            }
            "--apply" if !apply => apply = true,
            "--json" if !json => json = true,
            "--major" if mode == Mode::Normal => mode = Mode::ExplicitMajor,
            _ => return Err("Unknown/duplicate release option; use release --help".into()),
        }
        i += 1;
    }
    Ok((group, apply, json, mode))
}

fn select<'a>(
    config: &'a crate::config::Config,
    group: Option<&str>,
) -> Result<&'a versions::Group> {
    Ok(match group {
        Some(id) => config
            .releases
            .iter()
            .find(|g| g.id == id)
            .ok_or("Unknown release group")?,
        None if config.releases.len() == 1 => &config.releases[0],
        _ => return Err("Select an explicit --group from configured releases".into()),
    })
}

/// Reject mode conflicts before operation hooks, records or preparation write.
pub fn preflight(root: &Path, config: &crate::config::Config, args: &[String]) -> Result<()> {
    let (group, _, _, mode) = options(args)?;
    files::pending(root, select(config, group)?, mode)?;
    Ok(())
}

pub fn run(args: &[String], output: &mut impl Write) -> Result<()> {
    if args == ["--help"] || args == ["-h"] {
        writeln!(output,"Usage: craftguild-standards-engine release [--group ID] [--major] [--apply] [--json]\nDefault: preview only. --major requests one explicit MAJOR increment.\nApply prepares local files; commit/tag/push are separate.")?;
        return Ok(());
    }
    let (group, apply, json, mode) = options(args)?;
    let root = std::env::current_dir()?;
    let config = crate::config::load(&root)?;
    let selected = select(&config, group)?;
    let plan = plan_with_mode(&root, selected, mode)?;
    // Publish the preview before mutation; a broken output stream prevents writes.
    if json {
        serde_json::to_writer_pretty(&mut *output, &plan)?;
        writeln!(output)?;
    } else if plan.prepared {
        writeln!(output, "Already prepared {} at {}", plan.tag, plan.head)?;
    } else if plan.next.is_none() {
        writeln!(
            output,
            "No release: no version-incrementing changes since {}",
            plan.base.as_deref().unwrap_or("root")
        )?;
    } else {
        writeln!(
            output,
            "{}: {} -> {}\nIncrement: {:?}; mode: {:?}\nRange: {}..{}\nFiles: {}\n\n{}",
            plan.group,
            plan.current,
            plan.next.as_deref().unwrap(),
            plan.bump,
            plan.mode,
            plan.base.as_deref().unwrap_or("root"),
            plan.head,
            plan.changes
                .iter()
                .map(|c| c.path.as_str())
                .chain(plan.receipt_path.as_deref())
                .collect::<Vec<_>>()
                .join(", "),
            plan.notes
        )?;
    }
    output.flush()?;
    if apply {
        files::apply(&root, &plan)?;
    }
    Ok(())
}
