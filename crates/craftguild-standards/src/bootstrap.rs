//! Non-distributed bootstrap; no version/SBOM impersonation of the final CLI.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((command, rest)) if command == "banner" => {
            craftguild_standards::banner::run(rest, &mut std::io::stdout().lock())
        }
        Some((command, rest)) if command == "prepare" || command == "build" => {
            craftguild_standards::distribution::run(rest, command == "build")
        }
        Some((command, rest)) if command == "check" => {
            craftguild_standards::quality::run(rest, &mut std::io::stdout().lock())
        }
        _ => Err("Bootstrap usage: banner | prepare | build".into()),
    }
}
