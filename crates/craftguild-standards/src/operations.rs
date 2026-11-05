//! Run declared preparation around ordinary gates with technical attempt history.

use crate::{
    build_config::{Build, Stage},
    config::{self, Artifact},
    execution,
    execution_context::{Context, Options},
    execution_hooks as hooks,
    execution_process::Runner,
    execution_record::Record,
    execution_transaction::{Lock, Snapshot},
    operation_config::Operation,
};
use craftguild_core::Result;
use std::{collections::BTreeMap, io::Write};

pub fn run<W: Write>(
    name: &str,
    args: &[String],
    output: &mut W,
    gate: impl FnOnce(&Runner, &mut W) -> Result<()>,
) -> Result<()> {
    if matches!(args, [flag] if flag == "--help" || flag == "-h") {
        return gate(&Runner::default(), output);
    }
    if name == "assess" && args.iter().any(|arg| arg == "--root") {
        return gate(&Runner::default(), output);
    }
    let root = std::env::current_dir()?.canonicalize()?;
    let config = config::load(&root)?;
    let Some(operation) = config
        .operations
        .iter()
        .flatten()
        .find(|op| op.name.text() == if name == "assess" { "check" } else { name })
    else {
        return gate(&Runner::default(), output);
    };
    crate::execution_process::initialize_signals()?;
    let _lock = Lock::acquire(&root)?;
    let options = Options::default();
    let scope = config.artifact(&operation.artifact)?;
    let context = Context::resolve(&root, scope, &options)?;
    let mut scoped = scope.clone();
    scoped.build = Some(Build {
        hooks: operation.hooks.clone(),
        ..Build::default()
    });
    let preparations = preparation(&config, operation)?;
    let ordered = scoped.build.as_ref().unwrap().hook_order()?;
    let mut steps = vec![];
    for stage in [Stage::Prerequisite, Stage::Prepare] {
        steps.extend(
            ordered
                .iter()
                .filter(|h| h.stage == stage)
                .map(|h| format!("hook:{}", h.id)),
        );
    }
    steps.extend(
        preparations
            .iter()
            .map(|(a, _)| format!("prepare:{}", a.id)),
    );
    steps.extend(
        ordered
            .iter()
            .filter(|h| h.stage == Stage::Evidence)
            .map(|h| format!("hook:{}", h.id)),
    );
    steps.push(name.into());
    steps.extend(
        ordered
            .iter()
            .filter(|h| h.stage == Stage::Verify)
            .map(|h| format!("hook:{}", h.id)),
    );
    let mut record = Record::create(&root, context.clone(), steps)?;
    let mut runner = Runner {
        toolchain: context.toolchain.clone(),
        log: Some(record.directory.join("process.jsonl")),
        env: context.env(
            &format!(".craftguild/{}/prepared.json", scope.command),
            &record.run,
        )?,
        ..Runner::default()
    };
    runner.env.extend(context.environment.clone());
    runner
        .env
        .insert("CRAFTGUILD_OPERATION".into(), name.into());
    runner.env.insert(
        "CARGO_TARGET_DIR".into(),
        context.target_dir.to_string_lossy().into(),
    );
    let mut snapshot = Snapshot::create(&record.directory)?;
    let result = (|| -> Result<()> {
        hooks::stage(
            &root,
            &scoped,
            Stage::Prerequisite,
            &mut record,
            &mut runner,
            &mut snapshot,
        )?;
        hooks::stage(
            &root,
            &scoped,
            Stage::Prepare,
            &mut record,
            &mut runner,
            &mut snapshot,
        )?;
        for (artifact, compile) in preparations {
            record.step(&format!("prepare:{}", artifact.id), &runner, || {
                let context = Context::resolve(&root, artifact, &options)?;
                let mut selected = artifact.clone();
                selected.inputs.extend(
                    operation
                        .hooks
                        .iter()
                        .filter(|h| h.stage < Stage::Evidence)
                        .filter_map(|h| h.environment_output.clone()),
                );
                let mut child_options = options.clone();
                child_options.environment = runner
                    .env
                    .iter()
                    .filter(|(name, _)| {
                        !name.starts_with("CRAFTGUILD_")
                            && !matches!(name.as_str(), "CARGO_TARGET_DIR" | "RUSTUP_TOOLCHAIN")
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                execution::execute(&root, &selected, context, &child_options, compile, true)?;
                Ok(())
            })?;
        }
        hooks::stage(
            &root,
            &scoped,
            Stage::Evidence,
            &mut record,
            &mut runner,
            &mut snapshot,
        )?;
        record.environment(&runner.env)?;
        let evidence = record.evidence.clone();
        record.step(name, &runner, || {
            hooks::verify_evidence(&root, &evidence)?;
            gate(&runner, output)?;
            if name != "release" {
                hooks::verify_evidence(&root, &evidence)?;
            }
            runner.check()
        })?;
        hooks::stage(
            &root,
            &scoped,
            Stage::Verify,
            &mut record,
            &mut runner,
            &mut snapshot,
        )?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            snapshot.finish()?;
            record.finish("succeeded")
        }
        Err(error) => {
            if let Err(cleanup) = runner.rollback_safe() {
                record.finish("failed")?;
                return Err(format!("{error}; {cleanup}").into());
            }
            snapshot.restore()?;
            record.finish(if runner.cancelled() {
                "cancelled"
            } else {
                "failed"
            })?;
            Err(error)
        }
    }
}

fn preparation<'a>(
    config: &'a config::Config,
    operation: &Operation,
) -> Result<Vec<(&'a Artifact, bool)>> {
    let mut ordered = vec![];
    let mut compile = BTreeMap::new();
    for (ids, build) in [(&operation.prepare, false), (&operation.build, true)] {
        for id in ids {
            for artifact in crate::build_config::artifact_order(config, Some(id))? {
                if !compile.contains_key(&artifact.id) {
                    ordered.push(artifact);
                }
                let mode = compile.entry(artifact.id.clone()).or_insert(false);
                *mode |= build;
            }
        }
    }
    Ok(ordered
        .into_iter()
        .map(|a| {
            let mode = compile[&a.id];
            (a, mode)
        })
        .collect())
}
