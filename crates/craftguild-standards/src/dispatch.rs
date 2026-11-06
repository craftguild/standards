//! One operational entry for final CLI, bootstrap, Make and CI.

use crate::{
    config::{self, Phase},
    execution_context::PROTOCOL,
    execution_process::Runner,
};
use craftguild_core::Result;
use std::io::Write;

pub fn run(name: &str, args: &[String], output: &mut impl Write) -> Result<()> {
    match name {
        "protocol" if args.is_empty() => {
            writeln!(output, "{PROTOCOL}")?;
            Ok(())
        }
        "banner" => crate::banner::run(args, output),
        "build" | "prepare" => crate::execution::run(args, name == "build"),
        "install" => crate::execution_install::run(args),
        "installer" => crate::installer::run(args),
        "package" => crate::package::run(args),
        "check" | "release" | "test" | "acceptance" | "assess" => {
            crate::operations::run(name, args, output, |runner, output| match name {
                "check" => crate::quality::run_with(args, output, runner),
                "release" => crate::release::run(args, output),
                "assess" => crate::verification::run(name, args, output),
                _ => gates(name, args, runner),
            })
        }
        "check-release-config"
        | "validate-sbom"
        | "verify"
        | "verify-frontend"
        | "verify-receipt"
        | "measure"
        | "measure-conditional" => crate::verification::run(name, args, output),
        _ => Err("Unknown command; use --help".into()),
    }
}

fn gates(name: &str, args: &[String], runner: &Runner) -> Result<()> {
    if matches!(args, [flag] if flag == "-h" || flag == "--help") {
        println!(
            "Usage: {name}{}",
            if name == "acceptance" {
                " [--gate ID]"
            } else {
                ""
            }
        );
        return Ok(());
    }
    let selected = match (name, args) {
        (_, []) => None,
        ("acceptance", [flag, id]) if flag == "--gate" => Some(id.as_str()),
        _ => return Err(format!("Invalid {name} arguments; use --help").into()),
    };
    let root = std::env::current_dir()?;
    let config = config::load(&root)?;
    let phase = if name == "test" {
        Phase::Test
    } else {
        Phase::Acceptance
    };
    let mut executed = 0;
    if phase == Phase::Test {
        for adapter in &config.adapters {
            runner.run(&root, &adapter.test)?;
            executed += 1;
        }
    }
    for check in config
        .commands
        .iter()
        .filter(|c| c.phase == phase && selected.is_none_or(|id| c.id == id))
    {
        runner.run(&root, &check.argv)?;
        executed += 1;
    }
    if executed == 0 {
        return Err(format!("No {name} gates configured").into());
    }
    Ok(())
}
