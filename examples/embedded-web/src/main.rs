//! The product owns argument grammar; common functions own offline presentation.
use craftguild_cli::{help, sbom, version, Brand};
use std::io::Write;
include!(concat!(env!("OUT_DIR"), "/craftguild.rs"));
const HELP: &str = "Usage: embedded-demo [--help | --version | --machine-version | sbom]\n";

fn main() {
    if let Err(error) = run() {
        eprintln!("embedded-demo: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let brand = Brand {
        banner: include_str!("../assets/banner.txt"),
        project: "embedded-demo",
    };
    let mut out = std::io::stdout().lock();
    match args.as_slice() {
        [] => help(&mut out, brand, HELP)?,
        [flag] if flag == "-h" || flag == "--help" => help(&mut out, brand, HELP)?,
        [flag] if flag == "-V" || flag == "--version" => version(&mut out, brand, BUILD_INFO)?,
        [flag] if flag == "--machine-version" => out.write_all(BUILD_INFO_JSON.as_bytes())?,
        [command] if command == "sbom" => sbom(&mut out, SBOM)?,
        [command] if command == "asset" => out.write_all(include_bytes!("../web/dist/app.js"))?,
        _ => return Err("Unknown operation; use --help".into()),
    }
    out.flush()?;
    Ok(())
}
