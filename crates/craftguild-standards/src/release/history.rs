//! Fixed Git comparison ranges, OpenPGP verification and deterministic notes.

use super::{
    conventional::{self, Bump},
    versions::{self, Group},
};
use craftguild_core::Result;
use std::path::Path;

pub fn git(root: &Path, args: &[&str]) -> Result<String> {
    let argv: Vec<_> = ["git", "--literal-pathspecs"]
        .into_iter()
        .chain(args.iter().copied())
        .map(String::from)
        .collect();
    Ok(String::from_utf8(crate::command::capture(root, &argv)?)?
        .trim_end()
        .into())
}

pub fn ancestor(root: &Path, before: &str, after: &str) -> Result<bool> {
    let status = std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", before, after])
        .current_dir(root)
        .status()?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err("Cannot determine Git ancestry".into()),
    }
}

pub fn baseline(root: &Path, group: &Group, head: &str) -> Result<Option<(String, String)>> {
    if git(root, &["rev-parse", "--is-shallow-repository"])? != "false" {
        return Err(
            "Release requires full history and all release tags; fetch --unshallow --tags".into(),
        );
    }
    let mut best = None;
    for tag in git(root, &["tag", "--list"])?.lines() {
        let Some(text) = tag.strip_prefix(&group.tag_prefix) else {
            continue;
        };
        let Ok(version) = semver::Version::parse(text) else {
            continue;
        };
        versions::stable(text)?;
        if ancestor(root, tag, head)? && best.as_ref().is_none_or(|(v, _)| &version > v) {
            best = Some((version, tag.to_owned()));
        }
    }
    Ok(best.map(|(v, tag)| (tag, v.to_string())))
}

pub fn notes(
    root: &Path,
    group: &Group,
    base: Option<&str>,
    head: &str,
    version: &str,
) -> Result<(Bump, String)> {
    let range = base
        .map(|b| format!("{b}..{head}"))
        .unwrap_or_else(|| head.into());
    let mut args = vec![
        "rev-list",
        "--reverse",
        "--topo-order",
        "--full-history",
        &range,
        "--",
    ];
    args.extend(group.paths.iter().map(String::as_str));
    let commits = git(root, &args)?;
    let mut bump = Bump::None;
    let mut rows = Vec::new();
    let mut breaking = Vec::new();
    for sha in commits.lines() {
        let raw = git(root, &["cat-file", "commit", sha])?;
        if !openpgp(&raw) {
            return Err(
                format!("Commit {sha} lacks an OpenPGP signature; stop before release").into(),
            );
        }
        git(root, &["-c", "gpg.format=openpgp", "verify-commit", sha])?;
        let message = git(root, &["show", "-s", "--format=%B", sha])?;
        let (change, detail) =
            conventional::parse(&message).map_err(|e| format!("Commit {sha}: {e}"))?;
        bump = bump.max(change);
        rows.push(format!(
            "- {} ({sha})",
            message.lines().next().unwrap_or("")
        ));
        if !detail.is_empty() {
            breaking.push(format!("- {detail} ({sha})"));
        }
    }
    let baseline = base.unwrap_or("repository root (initial release)");
    let mut text = format!(
        "# {}{version}\n\nGroup: `{}`\n\nCompare: `{baseline}` → `{head}`\n\n## Changes\n\n{}\n",
        group.tag_prefix,
        group.id,
        rows.join("\n")
    );
    if !breaking.is_empty() {
        text.push_str(&format!(
            "\n## Breaking changes\n\n{}\n",
            breaking.join("\n")
        ));
    }
    Ok((bump, text))
}

fn openpgp(raw: &str) -> bool {
    raw.split("\n\n")
        .next()
        .unwrap_or("")
        .lines()
        .any(|line| line == "gpgsig -----BEGIN PGP SIGNATURE-----")
}

pub fn dirty(root: &Path) -> Result<Vec<String>> {
    let raw = git(
        root,
        &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    )?;
    raw.split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.len() < 4 || s[..2].contains(['R', 'C']) {
                return Err("Renames/copies are not release preparation edits".into());
            }
            Ok(s[3..].to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::openpgp;
    #[test]
    fn missing_ssh_and_body_only_signatures_are_rejected_without_creating_commits() {
        assert!(!openpgp("tree example\n\nfeat: unsigned mock"));
        assert!(!openpgp(
            "gpgsig -----BEGIN SSH SIGNATURE-----\n data\n\nfeat: mock"
        ));
        assert!(!openpgp(
            "tree example\n\nfeat: mock\n\ngpgsig -----BEGIN PGP SIGNATURE-----"
        ));
        assert!(openpgp(
            "gpgsig -----BEGIN PGP SIGNATURE-----\n data\n\nfeat: mock"
        ));
    }
}
