//! Common quality engine: inventory, roles, policies, adapter gates and stable results.

use crate::{
    config::{self, Config, Language, Phase, Role},
    generated, inventory, measure, policies,
};
use craftguild_core::Result;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct Finding {
    pub rule: String,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub scope: String,
    pub counter: String,
    pub measured: BTreeMap<String, usize>,
    pub exempt: BTreeSet<String>,
    pub excluded: BTreeSet<String>,
    pub executed: Vec<String>,
    pub findings: Vec<Finding>,
}

impl Report {
    fn finding(&mut self, rule: &str, path: Option<&str>, message: impl ToString) {
        self.findings.push(Finding {
            rule: rule.into(),
            path: path.map(str::to_owned),
            message: message.to_string(),
        });
    }
}

pub fn assess(root: &Path, config: &Config, changed: Option<&str>) -> Result<Report> {
    let all = inventory::files(root, None)?;
    let selected = if changed.is_some() {
        inventory::files(root, changed)?
    } else {
        all.clone()
    };
    let roles = inventory::roles(&all, &config.sources)?;
    let mut report = Report {
        schema_version: 1,
        scope: changed.map_or("all".into(), |s| format!("changes from {s}")),
        counter: format!(
            "{} {}",
            crate::contract::measurement().counter,
            crate::contract::measurement().version
        ),
        ..Report::default()
    };
    measure::counter(root)?;
    report.exempt = generated::verify(root, &config.generated)?;
    for (name, role) in &roles {
        if !selected.contains(name) {
            continue;
        }
        if matches!(
            role,
            Role::Test | Role::TestHelper | Role::Fixture | Role::Documentation | Role::Vendored
        ) {
            report.excluded.insert(name.clone());
            continue;
        }
        if *role == Role::Generated
            && !config
                .generated
                .iter()
                .any(|g| name.starts_with(&(g.output_root.clone() + "/")))
        {
            report.finding(
                "CG-GEN-001",
                Some(name),
                "Generated role lacks verified ownership",
            );
        }
        match measure::file(root, name) {
            Ok(code) => {
                report.measured.insert(name.clone(), code);
                let limit = crate::contract::measurement().implementation_limit;
                if code > limit && !report.exempt.contains(name) {
                    report.finding(
                        "CG-SIZE-001",
                        Some(name),
                        format!(
                            "{code} implementation Code lines exceeds {limit}; assess responsibilities"
                        ),
                    );
                }
            }
            Err(error) => report.finding("CG-SIZE-002", Some(name), format!("Unmeasured: {error}")),
        }
    }
    if report.measured.is_empty() {
        report.finding(
            "CG-SCOPE-001",
            None,
            "No implementation files measured in requested scope",
        );
    }
    for (id, message) in policies::check(root, &config.policies, &all)? {
        report.finding(&id, None, message);
    }
    agent_instructions(root, &config.standards_version, &mut report)?;
    Ok(report)
}

fn agent_instructions(root: &Path, version: &str, report: &mut Report) -> Result<()> {
    let instructions = std::fs::read_to_string(root.join("AGENTS.md"))?;
    let block = instructions
        .split_once("<!-- craftguild:common:start -->")
        .and_then(|(_, rest)| rest.split_once("<!-- craftguild:common:end -->"));
    match block {
        Some((common, specific))
            if common.contains(version)
                && common.contains("make check")
                && common.to_lowercase().contains("skill")
                && !specific.trim().is_empty()
                && !common.contains("400")
                && !common.contains("tokei") => {}
        _ => report.finding(
            "CG-AGENT-001",
            Some("AGENTS.md"),
            "Expected versioned common references and separate repository contracts",
        ),
    }
    for file in ["CLAUDE.md", "GEMINI.md"] {
        if root.join(file).exists()
            && !std::fs::read_to_string(root.join(file))?.contains("AGENTS.md")
        {
            report.finding(
                "CG-AGENT-002",
                Some(file),
                "Other agent instructions must reference AGENTS.md",
            );
        }
    }
    Ok(())
}

pub fn gates(root: &Path, config: &Config, report: &mut Report) -> Result<()> {
    gates_with(
        root,
        config,
        report,
        &crate::execution_process::Runner::default(),
    )
}

pub fn gates_with(
    root: &Path,
    config: &Config,
    report: &mut Report,
    runner: &crate::execution_process::Runner,
) -> Result<()> {
    let all = inventory::roles(&inventory::files(root, None)?, &config.sources)?;
    for (language, extensions) in [
        (Language::Rust, vec!["rs"]),
        (
            Language::Npm,
            vec!["ts", "tsx", "js", "jsx", "mjs", "cjs", "mts", "cts"],
        ),
    ] {
        let used = all.keys().any(|p| {
            Path::new(p)
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|s| extensions.contains(&s))
        });
        if used && !config.adapters.iter().any(|a| a.language == language) {
            report.finding(
                "CG-ADAPTER-001",
                None,
                format!("Missing required {language:?} language adapter"),
            );
        }
    }
    if config.adapters.is_empty() && !config.commands.iter().any(|c| c.phase == Phase::Test) {
        report.finding("CG-GATE-001", None, "No ordinary test gate configured");
    }
    for (index, adapter) in config.adapters.iter().enumerate() {
        for (kind, argv) in [
            ("test", Some(&adapter.test)),
            ("format", adapter.format.as_ref()),
            ("lint", Some(&adapter.lint)),
            ("typecheck", adapter.typecheck.as_ref()),
        ] {
            if let Some(argv) = argv {
                gate(
                    root,
                    &format!("adapter-{index}-{kind}"),
                    argv,
                    report,
                    runner,
                );
            }
        }
    }
    for check in config
        .commands
        .iter()
        .filter(|c| c.phase != Phase::Acceptance)
    {
        gate(root, &check.id, &check.argv, report, runner);
    }
    Ok(())
}

fn gate(
    root: &Path,
    id: &str,
    argv: &[String],
    report: &mut Report,
    runner: &crate::execution_process::Runner,
) {
    match runner.capture(root, argv) {
        Ok(_) => report.executed.push(id.into()),
        Err(error) => report.finding("CG-GATE-002", None, format!("{id}: {error}")),
    }
}

pub fn run(args: &[String], output: &mut impl Write) -> Result<()> {
    run_with(args, output, &crate::execution_process::Runner::default())
}

pub fn run_with(
    args: &[String],
    output: &mut impl Write,
    runner: &crate::execution_process::Runner,
) -> Result<()> {
    if matches!(args,[flag] if flag=="-h" || flag=="--help") {
        writeln!(output, "Usage: check [--changed | --base REV] [--json]")?;
        output.flush()?;
        return Ok(());
    }
    let mut changed = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" if !json => json = true,
            "--changed" if changed.is_none() => changed = Some("HEAD".to_owned()),
            "--base" if index + 1 < args.len() => {
                index += 1;
                changed = Some(args[index].clone());
            }
            _ => return Err("Usage: check [--changed | --base REV] [--json]".into()),
        }
        index += 1;
    }
    let root = std::env::current_dir()?;
    let result = (|| {
        let config = config::load(&root)?;
        let mut report = assess(&root, &config, changed.as_deref())?;
        gates_with(&root, &config, &mut report, runner)?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(report)
    })();
    let report = result.unwrap_or_else(|error| {
        let mut report = Report {
            schema_version: 1,
            ..Report::default()
        };
        report.finding("CG-CHECK-001", None, error);
        report
    });
    if json {
        serde_json::to_writer_pretty(&mut *output, &report)?;
        writeln!(output)?;
    } else {
        for finding in &report.findings {
            writeln!(
                output,
                "{} {}: {}",
                finding.rule,
                finding.path.as_deref().unwrap_or("repository"),
                finding.message
            )?;
        }
        writeln!(output,"{}: {} measured files, maximum {} Code lines, {} verified size exceptions, {} gates executed",
            if report.findings.is_empty(){"PASS"}else{"FAIL"},report.measured.len(),report.measured.values().max().unwrap_or(&0),report.exempt.len(),report.executed.len())?;
    }
    output.flush()?;
    if report.findings.is_empty() {
        Ok(())
    } else {
        Err("Quality checks failed; see rule findings".into())
    }
}
