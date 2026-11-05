//! Offline root operations precede configuration and operational tool dispatch.

use craftguild_cli::{help, sbom, version, Brand};

include!(concat!(env!("OUT_DIR"), "/craftguild.rs"));

const HELP: &str = "Usage: craftguild-standards <COMMAND>\n\nCommands:\n  banner TEXT    Generate Small Slant artwork\n  build          Build locked artifacts (debug or release)\n  prepare        Prepare and validate artifact SBOMs\n  install        Copy a matching verified executable without building\n  test           Run ordinary language and repository tests\n  acceptance     Run explicit configured acceptance gates\n  protocol       Print the entry compatibility identifier\n  assess         Report structural quality without running gates\n  verify         Verify a prepared seal and SBOM\n  verify-receipt Verify local sealed inputs, inventories and outputs\n  validate-sbom  Validate CycloneDX JSON\n  check-release-config  Validate version ownership without edits\n  measure        Measure implementation Code with pinned tokei\n  measure-conditional  Separate Rust test regions and measure\n  check          Run common quality, language and repository gates\n  release        Preview or apply a Conventional Commits release\n  sbom           Print the embedded CycloneDX JSON\n";

fn main() {
    if let Err(error) = run() {
        eprintln!("craftguild-standards: {error}");
        std::process::exit(1);
    }
}

fn run() -> craftguild_core::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let brand = Brand {
        banner: include_str!("../../../assets/banner.txt"),
        project: "standards",
    };
    let mut output = std::io::stdout().lock();
    match args.as_slice() {
        [] => help(&mut output, brand, HELP)?,
        [flag] if flag == "-h" || flag == "--help" => help(&mut output, brand, HELP)?,
        [flag] if flag == "-V" || flag == "--version" => version(&mut output, brand, BUILD_INFO)?,
        [command] if command == "sbom" => sbom(&mut output, SBOM)?,
        [command, rest @ ..] => craftguild_standards::dispatch::run(command, rest, &mut output)?,
    }
    Ok(())
}
