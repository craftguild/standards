//! Conventional Commits grammar and stable SemVer increment precedence.

use craftguild_core::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bump {
    None,
    Patch,
    Minor,
    Major,
}

pub fn parse(message: &str) -> Result<(Bump, String)> {
    let lines: Vec<_> = message.trim_end().lines().collect();
    let subject = *lines.first().ok_or("Empty commit message")?;
    let (prefix, description) = subject
        .split_once(": ")
        .ok_or("Expected Conventional Commit type: description")?;
    if description.trim().is_empty() || (lines.len() > 1 && !lines[1].is_empty()) {
        return Err("Commit body must follow a blank line; description must be nonempty".into());
    }
    let bang = prefix.ends_with('!');
    let prefix = prefix.strip_suffix('!').unwrap_or(prefix);
    let kind = if let Some((kind, scope)) = prefix.split_once('(') {
        let scope = scope.strip_suffix(')').ok_or("Unclosed commit scope")?;
        if scope.is_empty() || scope.chars().any(|c| c.is_control() || "()!".contains(c)) {
            return Err("Invalid commit scope".into());
        }
        kind
    } else {
        prefix
    };
    if kind.is_empty() || !kind.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("Invalid Conventional Commit type".into());
    }
    let mut breaking = Vec::new();
    let mut footer = false;
    let mut breaking_value = false;
    for (i, line) in lines.iter().enumerate().skip(2) {
        let is_breaking =
            line.starts_with("BREAKING CHANGE:") || line.starts_with("BREAKING-CHANGE:");
        if is_breaking {
            let value = line.split_once(':').unwrap().1;
            if !value.starts_with(' ') || value.trim().is_empty() {
                return Err("Breaking footer requires a description".into());
            }
        }
        if let Some((token, value)) = trailer(line) {
            if !footer && !lines[i - 1].is_empty() {
                if is_breaking {
                    return Err("Breaking footer needs a blank separator".into());
                }
                continue;
            }
            footer = true;
            breaking_value = token == "BREAKING CHANGE" || token == "BREAKING-CHANGE";
            if breaking_value && !is_breaking {
                return Err("Breaking footer requires colon-space separation".into());
            }
            if breaking_value && value.trim().is_empty() {
                return Err("Breaking footer requires a description".into());
            }
            if breaking_value {
                breaking.push(value.trim().to_owned());
            }
        } else if breaking_value && !line.is_empty() {
            if let Some(last) = breaking.last_mut() {
                last.push(' ');
                last.push_str(line.trim());
            }
        }
    }
    let bump = if bang || !breaking.is_empty() {
        Bump::Major
    } else if kind.eq_ignore_ascii_case("feat") {
        Bump::Minor
    } else if kind.eq_ignore_ascii_case("fix") {
        Bump::Patch
    } else {
        Bump::None
    };
    if bang && breaking.is_empty() {
        breaking.push(description.to_owned());
    }
    Ok((bump, breaking.join("; ")))
}

/// A new footer ends the preceding value, including its multiline continuation.
fn trailer(line: &str) -> Option<(&str, &str)> {
    let (token, value) = line.split_once(": ").or_else(|| line.split_once(" #"))?;
    (token == "BREAKING CHANGE"
        || (!token.is_empty()
            && token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')))
    .then_some((token, value))
}
