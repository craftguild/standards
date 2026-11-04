//! External command execution reports missing tools and failed checks explicitly.

use craftguild_core::Result;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn capture(root: &Path, argv: &[String]) -> Result<Vec<u8>> {
    let (program, args) = argv.split_first().ok_or("Empty required command")?;
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Cannot execute required tool {program}: {e}"))?;
    if !output.status.success() {
        use std::io::Write;
        std::io::stderr().write_all(&output.stdout)?;
        return Err(format!(
            "Required command failed ({}) : {}",
            output.status,
            argv.join(" ")
        )
        .into());
    }
    Ok(output.stdout)
}

pub fn run(root: &Path, argv: &[String]) -> Result<()> {
    let (program, args) = argv.split_first().ok_or("Empty required command")?;
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .status()
        .map_err(|e| format!("Cannot execute required tool {program}: {e}"))?;
    if !status.success() {
        return Err(format!("Required command failed ({status}): {}", argv.join(" ")).into());
    }
    Ok(())
}
