//! Portable Cargo library target declarations and platform output filenames.

use craftguild_core::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
    pub name: String,
    pub formats: Vec<Format>,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Cdylib,
    Staticlib,
    Rlib,
}

impl Library {
    pub fn validate(&self) -> Result<()> {
        if !crate::build_config::identifier(&self.name)
            || self.name.contains('-')
            || self.formats.is_empty()
            || self.formats.iter().copied().collect::<BTreeSet<_>>().len() != self.formats.len()
        {
            return Err(
                "Library artifacts require a Rust target name and unique output formats".into(),
            );
        }
        if self
            .properties
            .iter()
            .any(|(name, value)| name.trim().is_empty() || value.trim().is_empty())
        {
            return Err("Library component properties require a name and evidence value".into());
        }
        Ok(())
    }

    pub fn filenames(&self, target: &str) -> Vec<String> {
        self.formats
            .iter()
            .map(|format| match format {
                Format::Cdylib if target.starts_with("wasm") => format!("{}.wasm", self.name),
                Format::Cdylib if target.contains("windows") => format!("{}.dll", self.name),
                Format::Cdylib if target.contains("apple") => format!("lib{}.dylib", self.name),
                Format::Cdylib => format!("lib{}.so", self.name),
                Format::Staticlib if target.contains("windows-msvc") => {
                    format!("{}.lib", self.name)
                }
                Format::Staticlib => format!("lib{}.a", self.name),
                Format::Rlib => format!("lib{}.rlib", self.name),
            })
            .collect()
    }
}
