//! Pinned tokei Code accounting, including embedded language blobs exactly once.

use crate::{command, rust_regions};
use craftguild_core::Result;
use serde_json::Value;
use std::path::Path;

pub fn counter(root: &Path) -> Result<()> {
    let version = String::from_utf8(command::capture(
        root,
        &[counter_program(), "--version".into()],
    )?)?;
    let contract = crate::contract::measurement();
    if version.split_whitespace().take(2).collect::<Vec<_>>()
        != [contract.counter.as_str(), contract.version.as_str()]
    {
        return Err("Required counter is tokei 15.0.0 with JSON serialization support".into());
    }
    Ok(())
}

pub fn file(root: &Path, name: &str) -> Result<usize> {
    let source = craftguild_core::paths::owned(root, name)?;
    let temp = tempfile::tempdir()?;
    let path = temp
        .path()
        .join(source.file_name().ok_or("Missing source name")?);
    if source.extension().is_some_and(|e| e == "rs") {
        let text = std::fs::read_to_string(&source)?;
        std::fs::write(&path, rust_regions::implementation(&text)?)?;
    } else {
        if source
            .extension()
            .is_some_and(|e| e == "html" || e == "svg")
        {
            crate::html_regions::verify(root, &source)?;
        }
        if source
            .extension()
            .is_some_and(|e| e == "vue" || e == "svelte")
        {
            return Err("No reliable mixed component test-region adapter; unmeasured".into());
        }
        if source
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| {
                matches!(
                    e,
                    "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts"
                )
            })
        {
            crate::npm_regions::verify(root, &source)?;
        }
        std::fs::copy(&source, &path)?;
    }
    let output = command::capture(
        root,
        &[
            counter_program(),
            "--files".into(),
            "--output".into(),
            "json".into(),
            "--".into(),
            path.to_str().ok_or("Non-UTF8 counting path")?.into(),
        ],
    )?;
    let data: Value = serde_json::from_slice(&output)?;
    let mut found = vec![];
    for (language, value) in data.as_object().ok_or("Invalid counter JSON")? {
        if language == "Total" {
            continue;
        }
        for report in value["reports"]
            .as_array()
            .ok_or("Missing tokei file reports")?
        {
            if report["name"].as_str() == path.to_str() {
                found.push(stats(&report["stats"])?);
            }
        }
    }
    if found.len() != 1 {
        return Err(format!("Unmeasured/ambiguous source: {name}").into());
    }
    Ok(found[0])
}

fn counter_program() -> String {
    std::env::var("CRAFTGUILD_TOKEI").unwrap_or_else(|_| "tokei".into())
}

fn stats(value: &Value) -> Result<usize> {
    let mut code = value["code"].as_u64().ok_or("Missing tokei Code value")? as usize;
    if let Some(blobs) = value["blobs"].as_object() {
        for child in blobs.values() {
            code += stats(child)?;
        }
    }
    Ok(code)
}
