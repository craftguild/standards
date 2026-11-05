//! Declarative compiler variants and bounded environment ownership.

use crate::build_config::{identifier, Build};
use craftguild_core::{paths, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum OutputRoot {
    #[default]
    TargetDirectory,
    Repository,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub id: String,
    pub target: Option<String>,
    pub output: Option<String>,
    pub output_root: Option<OutputRoot>,
    pub compiler: Option<Vec<String>>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSettings {
    pub target: String,
    #[serde(default)]
    pub cross_only: bool,
    pub output: Option<String>,
    pub output_root: Option<OutputRoot>,
    pub compiler: Option<Vec<String>>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

pub struct Settings {
    pub target: Option<String>,
    pub output: String,
    pub output_root: OutputRoot,
    pub compiler: Vec<String>,
    pub environment: BTreeMap<String, String>,
}

impl Build {
    pub fn settings(&self, variant: Option<&str>) -> Result<Settings> {
        self.settings_for(variant, None, None)
    }

    pub fn settings_for(
        &self,
        variant: Option<&str>,
        target: Option<&str>,
        host: Option<&str>,
    ) -> Result<Settings> {
        let selected = variant
            .map(|id| {
                self.variants
                    .iter()
                    .find(|v| v.id == id)
                    .ok_or_else(|| format!("Unknown build variant: {id}"))
            })
            .transpose()?;
        let mut environment = self.environment.clone();
        let target_settings = self.targets.iter().find(|settings| {
            Some(settings.target.as_str()) == target && (!settings.cross_only || target != host)
        });
        if let Some(settings) = target_settings {
            environment.extend(settings.environment.clone());
        }
        if let Some(variant) = selected {
            environment.extend(variant.environment.clone());
        }
        Ok(Settings {
            target: selected.and_then(|v| v.target.clone()),
            output: selected
                .and_then(|v| v.output.clone())
                .or_else(|| target_settings.and_then(|s| s.output.clone()))
                .or_else(|| self.output.clone())
                .unwrap_or_else(|| "{target}/{profile}/{file}".into()),
            output_root: selected
                .and_then(|v| v.output_root)
                .or_else(|| target_settings.and_then(|s| s.output_root))
                .or(self.output_root)
                .unwrap_or_default(),
            compiler: selected
                .and_then(|v| v.compiler.clone())
                .or_else(|| target_settings.and_then(|s| s.compiler.clone()))
                .or_else(|| self.compiler.clone())
                .unwrap_or_else(|| vec!["cargo".into()]),
            environment,
        })
    }
}

pub fn environment_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.starts_with(|c: char| c.is_ascii_digit())
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || name.starts_with("CRAFTGUILD_")
        || matches!(
            name,
            "CARGO_TARGET_DIR" | "RUSTUP_TOOLCHAIN" | "RUSTUP_AUTO_INSTALL"
        )
    {
        return Err("Environment cannot override engine context or compiler cache routing".into());
    }
    Ok(())
}

pub fn environment(values: &BTreeMap<String, String>) -> Result<()> {
    for (name, value) in values {
        environment_name(name)?;
        if value.contains('\0') {
            return Err("NUL in compiler environment value".into());
        }
    }
    Ok(())
}

fn compiler(argv: &[String]) -> Result<()> {
    if argv.is_empty() || argv.iter().any(|s| s.is_empty() || s.contains('\0')) {
        return Err("Compiler argv prefix must be nonempty".into());
    }
    Ok(())
}

pub fn validate(build: &Build) -> Result<()> {
    environment(&build.environment)?;
    if let Some(argv) = &build.compiler {
        compiler(argv)?;
    }
    let mut ids = BTreeSet::new();
    for settings in &build.targets {
        if !identifier(&settings.target) || !ids.insert(&settings.target) {
            return Err("Target settings require unique named targets".into());
        }
        environment(&settings.environment)?;
        if let Some(argv) = &settings.compiler {
            compiler(argv)?;
        }
    }
    ids.clear();
    for variant in &build.variants {
        if !identifier(&variant.id)
            || !ids.insert(&variant.id)
            || variant.target.as_deref().is_some_and(|s| !identifier(s))
        {
            return Err("Variants require unique portable names and named targets".into());
        }
        environment(&variant.environment)?;
        if let Some(argv) = &variant.compiler {
            compiler(argv)?;
        }
    }
    for output in build
        .output
        .iter()
        .chain(build.variants.iter().filter_map(|v| v.output.as_ref()))
        .chain(build.targets.iter().filter_map(|s| s.output.as_ref()))
    {
        let rendered = ["target", "target_path", "profile", "command", "file", "exe"]
            .into_iter()
            .fold(output.clone(), |value, name| {
                value.replace(
                    &format!("{{{name}}}"),
                    if name == "target_path" {
                        "target/"
                    } else {
                        "value"
                    },
                )
            });
        if rendered.contains(['{', '}']) {
            return Err("Unknown output substitution".into());
        }
        paths::relative(&rendered)?;
    }
    Ok(())
}
