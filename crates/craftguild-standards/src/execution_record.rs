//! Per-attempt technical transition history and immutable success provenance.

use crate::{
    distribution::{atomic_write, owned_directory},
    execution_context::{Context, PROTOCOL},
    execution_process::{now, Runner},
};
use craftguild_core::{paths, seal::Prepared, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub state: String,
    pub started_at: Option<u128>,
    pub finished_at: Option<u128>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub schema_version: u32,
    pub protocol: String,
    pub run: String,
    pub context: Context,
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub evidence: BTreeMap<String, crate::execution_hooks::Evidence>,
    pub state: String,
    pub steps: Vec<Step>,
    #[serde(skip)]
    pub directory: PathBuf,
}

impl Record {
    pub fn create(root: &Path, context: Context, steps: Vec<String>) -> Result<Self> {
        let run = format!(
            "{}-{}-{}",
            now(),
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let directory = format!(".craftguild/{}/runs/{run}", context.command);
        owned_directory(root, &directory)?;
        let record = Self {
            schema_version: 2,
            protocol: PROTOCOL.into(),
            run,
            context,
            environment: BTreeMap::new(),
            evidence: BTreeMap::new(),
            state: "running".into(),
            directory: root.join(directory),
            steps: steps
                .into_iter()
                .map(|id| Step {
                    id,
                    state: "pending".into(),
                    started_at: None,
                    finished_at: None,
                    error: None,
                })
                .collect(),
        };
        atomic_write(
            &record.directory.join("plan.json"),
            &serde_json::to_vec_pretty(&record)?,
        )?;
        record.save()?;
        Ok(record)
    }

    pub fn step<T>(
        &mut self,
        id: &str,
        runner: &Runner,
        action: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        runner.check()?;
        let index = self
            .steps
            .iter()
            .position(|s| s.id == id)
            .ok_or("Step missing from plan")?;
        self.steps[index].state = "running".into();
        self.steps[index].started_at = Some(now());
        self.save()?;
        let result = action();
        self.steps[index].finished_at = Some(now());
        self.steps[index].state = if result.is_ok() {
            "succeeded"
        } else if runner.cancelled() {
            "cancelled"
        } else {
            "failed"
        }
        .into();
        self.steps[index].error = result.as_ref().err().map(ToString::to_string);
        self.save()?;
        result
    }

    pub fn finish(&mut self, state: &str) -> Result<()> {
        if state == "interrupted" {
            for step in self.steps.iter_mut().filter(|s| s.state == "running") {
                step.state = "interrupted".into();
                step.finished_at = Some(now());
                step.error = Some("Execution stopped before completion; outputs restored".into());
            }
        }
        self.state = state.into();
        self.save()
    }

    pub fn environment(&mut self, environment: &BTreeMap<String, String>) -> Result<()> {
        self.environment = environment.clone();
        self.save()
    }

    pub fn evidence(&mut self, id: &str, evidence: crate::execution_hooks::Evidence) -> Result<()> {
        self.evidence.insert(id.into(), evidence);
        self.save()
    }

    fn save(&self) -> Result<()> {
        atomic_write(
            &self.directory.join("result.json"),
            &serde_json::to_vec_pretty(self)?,
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema_version: u32,
    pub protocol: String,
    pub run: String,
    pub context: Context,
    pub environment: BTreeMap<String, String>,
    pub prepared: Prepared,
    pub outputs: BTreeMap<String, String>,
}

impl Receipt {
    pub fn verify(&self, root: &Path) -> Result<()> {
        if self.schema_version != 2
            || self.protocol != PROTOCOL
            || self.outputs.is_empty()
            || self.context.command != self.prepared.build.command
            || self.context.target != self.prepared.build.target
            || self.context.profile.name() != self.prepared.build.profile
            || !self.context.compiler_version.is_empty()
                && self.context.compiler_version != self.prepared.build.compiler
        {
            return Err("Invalid artifact success receipt".into());
        }
        self.prepared
            .verify(root, &self.prepared.build, &self.prepared.features)?;
        if artifact_hashes(&self.context)? != self.outputs {
            return Err("Built output differs from success receipt".into());
        }
        Ok(())
    }
}

pub fn output_hashes(path: &Path) -> Result<BTreeMap<String, String>> {
    if path.ancestors().any(|p| p.is_symlink()) {
        return Err("Symlink in built output".into());
    }
    if path.is_file() {
        return Ok([(
            path.file_name()
                .ok_or("Output has no name")?
                .to_str()
                .ok_or("Non-UTF8 output")?
                .into(),
            paths::digest(&std::fs::read(path)?),
        )]
        .into());
    }
    let files = paths::inventory(
        path.parent().ok_or("Output has no parent")?,
        path.file_name()
            .ok_or("Output has no name")?
            .to_str()
            .ok_or("Non-UTF8 output")?,
    )?;
    if files.is_empty() {
        return Err("Built output directory is empty".into());
    }
    files
        .into_iter()
        .map(|name| {
            Ok((
                name.clone(),
                paths::hash_file(path.parent().unwrap(), &name)?,
            ))
        })
        .collect()
}

pub fn artifact_hashes(context: &Context) -> Result<BTreeMap<String, String>> {
    if context.outputs.is_empty() || context.outputs.first() != Some(&context.output) {
        return Err("Invalid resolved artifact output inventory".into());
    }
    let mut result = BTreeMap::new();
    for output in &context.outputs {
        for (name, hash) in output_hashes(output)? {
            if result.insert(name, hash).is_some() {
                return Err("Colliding artifact output names".into());
            }
        }
    }
    Ok(result)
}
