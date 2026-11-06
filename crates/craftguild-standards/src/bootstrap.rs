//! Non-distributed bootstrap; no version/SBOM impersonation of the final CLI.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (command, rest) = args.split_first().ok_or("Bootstrap usage: build | prepare | install | installer | package | test | check | acceptance | release | banner | protocol")?;
    craftguild_standards::dispatch::run(command, rest, &mut std::io::stdout().lock())
}
