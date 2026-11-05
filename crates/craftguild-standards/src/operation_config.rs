//! Explicit preparation dependencies around existing repository operation gates.

use crate::{
    build_config::{Build, Hook},
    config::Config,
};
use craftguild_core::Result;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub name: Name,
    pub artifact: String,
    #[serde(default)]
    pub prepare: Vec<String>,
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Name {
    Test,
    Check,
    Acceptance,
    Release,
}

impl Name {
    pub fn text(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Check => "check",
            Self::Acceptance => "acceptance",
            Self::Release => "release",
        }
    }
}

pub fn validate(config: &Config) -> Result<()> {
    if config.schema_version == 1 && config.operations.is_some() {
        return Err("Repository operation preparation requires configuration schema 2".into());
    }
    let mut names = BTreeSet::new();
    for operation in config.operations.iter().flatten() {
        if !names.insert(operation.name) {
            return Err("Duplicate repository operation".into());
        }
        let artifact = config.artifact(&operation.artifact)?;
        Build {
            hooks: operation.hooks.clone(),
            ..Build::default()
        }
        .validate(artifact)?;
        let mut artifacts = BTreeSet::new();
        for id in operation.prepare.iter().chain(&operation.build) {
            config.artifact(id)?;
            if !artifacts.insert(id) {
                return Err("Duplicate operation preparation artifact".into());
            }
        }
    }
    Ok(())
}
