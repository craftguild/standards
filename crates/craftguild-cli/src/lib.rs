//! Plain, fallible output functions. Applications retain their argument grammar.

#![forbid(unsafe_code)]

use std::io::{self, Write};

/// Committed ASCII artwork and a public organization project name.
#[derive(Clone, Copy, Debug)]
pub struct Brand<'a> {
    pub banner: &'a str,
    pub project: &'a str,
}

/// Borrowed build facts; adapters may transform one shared structured identity.
#[derive(Clone, Copy, Debug)]
pub struct BuildInfo<'a> {
    pub command: &'a str,
    pub version: &'a str,
    pub commit: &'a str,
    /// Some(false) is clean; None is an unavailable observation.
    pub dirty: Option<bool>,
    pub compiler: &'a str,
    pub target: &'a str,
    pub profile: &'a str,
}

fn printable(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| (32..=126).contains(&b))
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn brand(output: &mut impl Write, brand: Brand<'_>) -> io::Result<()> {
    if !printable(brand.project)
        || !brand
            .project
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        || !brand.banner.ends_with('\n')
        || brand
            .banner
            .lines()
            .any(|s| !printable(s) || s.ends_with(' '))
    {
        return Err(invalid(
            "Expected normalized ASCII banner and public project name",
        ));
    }
    let width = brand.banner.lines().map(str::len).max().unwrap_or(0);
    let affiliation = format!("craftguild/{}", brand.project);
    let padding = width.saturating_sub(affiliation.len()) / 2;
    output.write_all(brand.banner.as_bytes())?;
    writeln!(output, "{:padding$}{affiliation}\n", "")
}

/// Write the same root presentation for no args, -h and --help.
pub fn help(output: &mut impl Write, branding: Brand<'_>, root_help: &str) -> io::Result<()> {
    brand(output, branding)?;
    output.write_all(root_help.as_bytes())?;
    output.flush()
}

/// Write the application identity without discovering files or invoking tools.
pub fn version(
    output: &mut impl Write,
    branding: Brand<'_>,
    info: BuildInfo<'_>,
) -> io::Result<()> {
    if [
        info.command,
        info.version,
        info.commit,
        info.compiler,
        info.target,
        info.profile,
    ]
    .iter()
    .any(|v| !printable(v))
    {
        return Err(invalid("Build values must be nonempty printable ASCII"));
    }
    brand(output, branding)?;
    writeln!(output, "{} {}", info.command, info.version)?;
    let suffix = match info.dirty {
        Some(true) => " (dirty)",
        Some(false) => "",
        None => " (dirty: unknown)",
    };
    writeln!(output, "  commit   {}{suffix}", info.commit)?;
    writeln!(output, "  rustc    {}", info.compiler)?;
    writeln!(output, "  target   {}", info.target)?;
    writeln!(output, "  profile  {}", info.profile)?;
    output.flush()
}

/// Write prepared JSON unchanged; validation belongs to the build, not runtime.
pub fn sbom(output: &mut impl Write, embedded: &[u8]) -> io::Result<()> {
    output.write_all(embedded)?;
    output.flush()
}
