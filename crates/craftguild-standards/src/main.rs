//! Offline root operations precede configuration and operational tool dispatch.

use craftguild_cli::{help, sbom, version, Brand};

include!(concat!(env!("OUT_DIR"), "/craftguild.rs"));

const HELP: &str = "Usage: craftguild-standards <COMMAND>\n\nCommands:\n  banner TEXT    Generate Small Slant artwork\n  build          Prepare SBOM and build locked release artifacts\n  prepare        Prepare and validate artifact SBOMs\n  check          Run common quality, language and repository gates\n  sbom           Print the embedded CycloneDX JSON\n";

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
        [command, rest @ ..] if command == "banner" => {
            craftguild_standards::banner::run(rest, &mut output)?
        }
        [command, rest @ ..] if command == "build" || command == "prepare" => {
            craftguild_standards::distribution::run(rest, command == "build")?
        }
        [command, rest @ ..] if command == "check" => {
            craftguild_standards::quality::run(rest, &mut output)?
        }
        _ => return Err("Unknown command; use --help".into()),
    }
    Ok(())
}
