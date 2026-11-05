//! Schema 2 build declarations and dependency ordering, independent of execution.

use crate::config::{Artifact, Config, Kind};
use craftguild_core::{paths, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    pub library: Option<crate::rust_artifact::Library>,
    pub toolchain: Option<String>,
    pub profile: Option<Profile>,
    pub target_dir: Option<String>,
    pub output: Option<String>,
    pub output_root: Option<crate::build_settings::OutputRoot>,
    pub compiler: Option<Vec<String>>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub variants: Vec<crate::build_settings::Variant>,
    #[serde(default)]
    pub targets: Vec<crate::build_settings::TargetSettings>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Debug,
    #[default]
    Release,
}

impl Profile {
    pub fn name(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "debug" => Ok(Self::Debug),
            "release" => Ok(Self::Release),
            _ => Err("Profile must be debug or release".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Prerequisite,
    Prepare,
    Evidence,
    Verify,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hook {
    pub id: String,
    pub stage: Stage,
    pub argv: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub input_dirs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub output_dirs: Vec<String>,
    pub environment_output: Option<String>,
    #[serde(default)]
    pub unset_env: Vec<String>,
}

pub fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

pub fn output_path(value: &str, command: &str) -> Result<()> {
    paths::relative(value)?;
    let first = value.split('/').next().unwrap_or_default();
    if matches!(first, ".git" | ".private" | ".beads")
        || [
            format!(".craftguild/{command}"),
            ".craftguild/cache".into(),
            ".craftguild/engine".into(),
        ]
        .iter()
        .any(|p| value == p || value.starts_with(&format!("{p}/")))
        || matches!(
            value,
            ".craftguild/execution.lock" | ".craftguild/recovery.lock"
        )
    {
        return Err("Outputs cannot own repository history, user state or engine records".into());
    }
    Ok(())
}

fn overlaps(a: &str, b: &str) -> bool {
    a == b || a.starts_with(&format!("{b}/")) || b.starts_with(&format!("{a}/"))
}

impl Build {
    pub fn validate(&self, artifact: &Artifact) -> Result<()> {
        crate::build_settings::validate(self)?;
        if let Some(library) = &self.library {
            if artifact.kind != Kind::Rust {
                return Err("Only Rust artifacts select Cargo library targets".into());
            }
            library.validate()?;
        }
        if self.toolchain.as_deref().is_some_and(|s| !identifier(s)) {
            return Err("Toolchain must be an installed named toolchain".into());
        }
        if artifact.kind == Kind::Frontend && self.toolchain.is_some() {
            return Err("Frontend artifacts do not select a Rust toolchain".into());
        }
        if artifact.kind == Kind::Frontend
            && (self.compiler.is_some()
                || self.output.is_some()
                || self.output_root.is_some()
                || !self.targets.is_empty()
                || self.variants.iter().any(|v| {
                    v.target.is_some()
                        || v.compiler.is_some()
                        || v.output.is_some()
                        || v.output_root.is_some()
                }))
        {
            return Err("Frontend output/target comes from bundle configuration; Rust compiler settings are inapplicable".into());
        }
        if let Some(path) = &self.target_dir {
            paths::relative(path)?;
        }
        let mut ids = BTreeSet::new();
        let mut outputs: Vec<&str> = vec![];
        for hook in &self.hooks {
            if !identifier(&hook.id) || !ids.insert(&hook.id) {
                return Err("Hooks require unique portable ids".into());
            }
            if hook.argv.is_empty() || hook.argv.iter().any(|s| s.is_empty() || s.contains('\0')) {
                return Err("Hook argv must be nonempty".into());
            }
            let owns_outputs = !hook.outputs.is_empty() || !hook.output_dirs.is_empty();
            let environment_prerequisite =
                hook.stage == Stage::Prerequisite && hook.environment_output.is_some();
            if owns_outputs
                != (matches!(hook.stage, Stage::Prepare | Stage::Evidence)
                    || environment_prerequisite)
            {
                return Err(
                    "Prepare/evidence hooks own outputs; prerequisite/verify hooks only verify"
                        .into(),
                );
            }
            if let Some(path) = &hook.environment_output {
                if !hook.outputs.contains(path) || hook.stage == Stage::Verify {
                    return Err(
                        "Environment evidence must be a declared output before verification".into(),
                    );
                }
            }
            for name in &hook.unset_env {
                if name.starts_with("CRAFTGUILD_")
                    || name == "RUSTUP_AUTO_INSTALL"
                    || name.is_empty()
                    || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    return Err("Hook unset_env cannot remove selected artifact context".into());
                }
            }
            for path in hook
                .inputs
                .iter()
                .chain(&hook.input_dirs)
                .chain(&hook.outputs)
                .chain(&hook.output_dirs)
            {
                paths::relative(path)?;
            }
            for output in hook.outputs.iter().chain(&hook.output_dirs) {
                output_path(output, &artifact.command)?;
                if outputs.iter().any(|p| overlaps(p, output))
                    || hook
                        .inputs
                        .iter()
                        .chain(&hook.input_dirs)
                        .any(|p| overlaps(p, output))
                    || ["craftguild.toml", &artifact.manifest, &artifact.lockfile]
                        .iter()
                        .any(|p| overlaps(p, output))
                {
                    return Err("Hook outputs overlap input or engine ownership".into());
                }
                outputs.push(output);
            }
        }
        self.hook_order()?;
        Ok(())
    }

    pub fn hook_order(&self) -> Result<Vec<&Hook>> {
        fn visit<'a>(
            build: &'a Build,
            hook: &'a Hook,
            active: &mut BTreeSet<&'a str>,
            result: &mut Vec<&'a Hook>,
        ) -> Result<()> {
            if result.iter().any(|h| h.id == hook.id) {
                return Ok(());
            }
            if !active.insert(&hook.id) {
                return Err("Cyclic hook dependencies".into());
            }
            for id in &hook.depends_on {
                let dependency = build
                    .hooks
                    .iter()
                    .find(|h| &h.id == id)
                    .ok_or("Unknown hook dependency")?;
                if dependency.stage > hook.stage {
                    return Err("Hook depends on a later stage".into());
                }
                visit(build, dependency, active, result)?;
            }
            active.remove(hook.id.as_str());
            result.push(hook);
            Ok(())
        }
        let mut result = vec![];
        let mut hooks: Vec<_> = self.hooks.iter().collect();
        hooks.sort_by_key(|h| h.stage);
        for hook in hooks {
            visit(self, hook, &mut BTreeSet::new(), &mut result)?;
        }
        Ok(result)
    }
}

pub fn artifact_order<'a>(config: &'a Config, selected: Option<&str>) -> Result<Vec<&'a Artifact>> {
    fn visit<'a>(
        config: &'a Config,
        artifact: &'a Artifact,
        active: &mut BTreeSet<&'a str>,
        result: &mut Vec<&'a Artifact>,
    ) -> Result<()> {
        if result.iter().any(|a| a.id == artifact.id) {
            return Ok(());
        }
        if !active.insert(&artifact.id) {
            return Err("Cyclic artifact preparation dependencies".into());
        }
        if let Some(build) = &artifact.build {
            for id in &build.dependencies {
                visit(config, config.artifact(id)?, active, result)?;
            }
        }
        active.remove(artifact.id.as_str());
        result.push(artifact);
        Ok(())
    }
    let mut result = vec![];
    let roots = match selected {
        Some(id) => vec![config.artifact(id)?],
        None => config.artifacts.iter().collect(),
    };
    for artifact in roots {
        visit(config, artifact, &mut BTreeSet::new(), &mut result)?;
    }
    Ok(result)
}
