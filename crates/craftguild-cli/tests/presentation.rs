use craftguild_cli::{help, sbom, version, Brand, BuildInfo};
use std::io::{self, Write};

const BRAND: Brand<'_> = Brand {
    banner: "     xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n",
    project: "demo",
};

fn info(dirty: Option<bool>) -> BuildInfo<'static> {
    BuildInfo {
        command: "demo",
        version: "2.3.4",
        commit: "1234567890123456789012345678901234567890",
        dirty,
        compiler: "rustc 1.85.0",
        target: "x86_64-unknown-linux-gnu",
        profile: "release",
    }
}

#[test]
fn plain_centered_output_and_known_dirty_states() {
    let mut bytes = vec![];
    help(&mut bytes, BRAND, "Usage: demo\n").unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert_eq!(
        text,
        format!("{}          craftguild/demo\n\nUsage: demo\n", BRAND.banner)
    );
    assert!(!text.contains('\x1b'));
    for (dirty, suffix) in [
        (Some(false), ""),
        (Some(true), " (dirty)"),
        (None, " (dirty: unknown)"),
    ] {
        let mut bytes = vec![];
        version(&mut bytes, BRAND, info(dirty)).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains(&format!("  commit   {}{suffix}\n", info(dirty).commit)));
        assert_eq!(text.matches(BRAND.banner).count(), 1);
        assert!(text.contains("demo 2.3.4\n"));
    }
}

#[test]
fn sbom_is_byte_exact() {
    let bytes = b"{\n  \"bomFormat\":\"CycloneDX\"\n}\n";
    let mut out = vec![];
    sbom(&mut out, bytes).unwrap();
    assert_eq!(out, bytes);
}

struct Broken;
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn output_failure_is_never_success() {
    assert!(help(&mut Broken, BRAND, "Usage\n").is_err());
    assert!(version(&mut Broken, BRAND, info(Some(false))).is_err());
    assert!(sbom(&mut Broken, b"{}").is_err());
}

struct FlushFailure;
impl Write for FlushFailure {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
}

#[test]
fn buffered_flush_failure_is_never_success() {
    assert!(help(&mut FlushFailure, BRAND, "Usage\n").is_err());
    assert!(version(&mut FlushFailure, BRAND, info(Some(false))).is_err());
    assert!(sbom(&mut FlushFailure, b"{}").is_err());
}

#[test]
fn reject_control_sequences_in_brand_and_build_values() {
    let mut bytes = vec![];
    assert!(help(
        &mut bytes,
        Brand {
            project: "a\x1b",
            ..BRAND
        },
        "help"
    )
    .is_err());
    assert!(help(
        &mut bytes,
        Brand {
            banner: "bad \n",
            ..BRAND
        },
        "help"
    )
    .is_err());
    let mut bad = info(None);
    bad.compiler = "compiler\nforged";
    assert!(version(&mut bytes, BRAND, bad).is_err());
}
