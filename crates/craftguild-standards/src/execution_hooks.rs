//! Bounded argv hooks with fresh output ownership and sealed input evidence.

use crate::{
    build_config::{Hook, Stage},
    config::Artifact,
    execution_process::Runner,
    execution_record::Record,
    execution_transaction::Snapshot,
};
use craftguild_core::{paths, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn hashes(root: &Path, files: &[String], dirs: &[String]) -> Result<BTreeMap<String, String>> {
    let mut names = files.to_vec();
    for dir in dirs {
        names.extend(paths::inventory(root, dir)?);
    }
    names
        .into_iter()
        .map(|name| Ok((name.clone(), paths::hash_file(root, &name)?)))
        .collect()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub inputs: BTreeMap<String, String>,
    pub outputs: BTreeMap<String, String>,
    pub inventories: BTreeMap<String, BTreeSet<String>>,
}

impl Evidence {
    pub fn inputs(root: &Path, files: &[String]) -> Result<Self> {
        Ok(Self {
            inputs: hashes(root, files, &[])?,
            ..Self::default()
        })
    }

    pub fn verify(&self, root: &Path) -> Result<()> {
        for (name, expected) in self.inputs.iter().chain(&self.outputs) {
            if paths::hash_file(root, name)? != *expected {
                return Err(format!("Stale execution evidence: {name}").into());
            }
        }
        for (name, expected) in &self.inventories {
            if paths::inventory(root, name)? != *expected {
                return Err(format!("Changed execution evidence inventory: {name}").into());
            }
        }
        Ok(())
    }
}

pub fn verify_evidence(root: &Path, evidence: &BTreeMap<String, Evidence>) -> Result<()> {
    for value in evidence.values() {
        value.verify(root)?;
    }
    Ok(())
}

pub fn stage(
    root: &Path,
    artifact: &Artifact,
    stage: Stage,
    record: &mut Record,
    runner: &mut Runner,
    snapshot: &mut Snapshot,
) -> Result<()> {
    if let Some(build) = &artifact.build {
        for hook in build.hook_order()?.into_iter().filter(|h| h.stage == stage) {
            let mut hook_runner = runner.clone();
            hook_runner.unset_env.extend(hook.unset_env.iter().cloned());
            let (environment, evidence) =
                record.step(&format!("hook:{}", hook.id), &hook_runner, || {
                    let evidence = run(root, hook, &hook_runner, snapshot)?;
                    let environment = if let Some(path) = &hook.environment_output {
                        serde_json::from_slice(&std::fs::read(paths::owned(root, path)?)?)?
                    } else {
                        BTreeMap::new()
                    };
                    crate::build_settings::environment(&environment)?;
                    Ok((environment, evidence))
                })?;
            record.evidence(&hook.id, evidence)?;
            runner.env.extend(environment);
        }
    }
    Ok(())
}

fn run(root: &Path, hook: &Hook, runner: &Runner, snapshot: &mut Snapshot) -> Result<Evidence> {
    let before = hashes(root, &hook.inputs, &hook.input_dirs)?;
    for name in hook.outputs.iter().chain(&hook.output_dirs) {
        snapshot.clear(&root.join(name))?;
        crate::distribution::owned_directory(
            root,
            Path::new(name)
                .parent()
                .and_then(Path::to_str)
                .filter(|p| !p.is_empty())
                .unwrap_or(".craftguild"),
        )?;
    }
    runner.run(root, &hook.argv)?;
    if before != hashes(root, &hook.inputs, &hook.input_dirs)? {
        return Err("Hook changed its declared inputs".into());
    }
    let outputs = hashes(root, &hook.outputs, &hook.output_dirs)?;
    if matches!(hook.stage, Stage::Prepare | Stage::Evidence) && outputs.is_empty() {
        return Err("Hook did not produce declared outputs".into());
    }
    let inventories = hook
        .input_dirs
        .iter()
        .chain(&hook.output_dirs)
        .map(|directory| Ok((directory.clone(), paths::inventory(root, directory)?)))
        .collect::<Result<_>>()?;
    Ok(Evidence {
        inputs: before,
        outputs,
        inventories,
    })
}

pub fn seal_declarations(artifact: &mut Artifact) {
    if let Some(build) = &artifact.build {
        for hook in &build.hooks {
            artifact
                .inputs
                .extend(hook.inputs.iter().chain(&hook.outputs).cloned());
            artifact
                .input_dirs
                .extend(hook.input_dirs.iter().chain(&hook.output_dirs).cloned());
        }
    }
    artifact.inputs.sort();
    artifact.inputs.dedup();
    artifact.input_dirs.sort();
    artifact.input_dirs.dedup();
}
