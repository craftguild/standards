//! The installed initializer never dispatches repository operations globally.
use craftguild_cli::{help, sbom, version, Brand};

include!(concat!(env!("OUT_DIR"), "/craftguild.rs"));

const HELP: &str = "Usage: craftguild-standards <COMMAND>\n\nCommands:\n  init [--force] Initialize or update cwd to this published release\n  sbom           Print this initializer's embedded CycloneDX JSON\n\nRepository operations: make setup, make prime, make build, make check.\n";

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
        [command, rest @ ..] if command == "init" => {
            craftguild_standards::initialization::run(
                rest,
                &craftguild_standards::repository_pin::MachineIdentity::from_build(BUILD_INFO),
                &mut output,
            )?;
        }
        _ => return Err(
            "Unknown command; use --help. Use the pinned repository entry for engine operations"
                .into(),
        ),
    }
    Ok(())
}
