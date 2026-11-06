//! Real locked Cargo outputs, disposable signed history and adversarial receipts.

use serde_json::Value;
use std::{
    path::Path,
    process::{Command, Output},
};
#[path = "../../../conformance/git_fixture.rs"]
mod signing;

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
        .args(args)
        .current_dir(root)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("TARGET")
        .env_remove("PROFILE")
        .output()
        .unwrap()
}
fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}
fn fixture(standard: bool) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let canonical = temp.path().canonicalize().unwrap();
    let root = canonical.as_path();
    write(root,"Cargo.toml", "[package]\nname='demo'\nversion='1.2.3'\nedition='2021'\n[[bin]]\nname='demo'\npath='src/main.rs'\n[[bin]]\nname='other'\npath='src/main.rs'\n");
    write(
        root,
        "src/main.rs",
        r#"use std::io::Write;
fn main(){std::io::stdout().write_all(include_bytes!(env!("FIXTURE_SBOM"))).unwrap();}
"#,
    );
    write(
        root,
        "build.rs",
        r#"fn main(){
let record=std::env::var("CRAFTGUILD_PREPARED").unwrap();
let bom=std::path::Path::new(&record).parent().unwrap().join("bom.json");
println!("cargo:rustc-env=FIXTURE_SBOM={}/{}",std::env::var("CARGO_MANIFEST_DIR").unwrap(),bom.display());
println!("cargo:rerun-if-env-changed=CRAFTGUILD_PREPARED");}
"#,
    );
    let mut config = format!(
        "schema_version=2\nstandards_version=\"0.1.0\"\n{}",
        if standard {
            "rust_output_layout=\"standard\"\n"
        } else {
            ""
        }
    );
    for command in ["demo", "other"] {
        config.push_str(&format!(
            r#"
[[artifacts]]
id="{command}"
command="{command}"
project="demo"
kind="rust"
manifest="Cargo.toml"
lockfile="Cargo.lock"
package="demo"
[artifacts.build]
target_dir="configured"
[[installers]]
artifact="{command}"
repository="owner/repo"
release_group="demo"
output="scripts/{command}.sh"
"#
        ));
    }
    config.push_str(
        r#"
[[releases]]
id="demo"
initial_version="1.2.3"
tag_prefix="v"
notes="docs/releases"
paths=["src"]
[[releases.cargo]]
manifest="Cargo.toml"
lockfile="Cargo.lock"
packages=["demo"]
"#,
    );
    write(root, "craftguild.toml", config);
    write(
        root,
        ".gitignore",
        ".craftguild/\nconfigured/\ntarget/\nstage/\n/dist/\nsrc/ignored-*.rs\n",
    );
    assert!(Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    success(cli(root, &["installer"]));
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    signing::commit(root, "feat: isolated package fixture");
    temp
}
fn receipt(root: &Path, command: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(root.join(format!(".craftguild/{command}/receipt.json"))).unwrap(),
    )
    .unwrap()
}
fn rejected(root: &Path, output: &Path) {
    let result = cli(root, &["package", "--output", output.to_str().unwrap()]);
    assert!(!result.status.success(), "unexpected package success");
    assert!(!output.exists());
}

#[test]
fn verified_group_assets_and_full_failure_boundaries() {
    for standard in [false, true] {
        let temp = fixture(standard);
        let canonical = temp.path().canonicalize().unwrap();
        let root = canonical.as_path();
        let distribution = tempfile::tempdir().unwrap();
        let output = distribution.path().join("delivery");
        rejected(root, &output); // Missing receipt.
        success(cli(root, &["build"]));
        let first = receipt(root, "demo");
        let target = first["context"]["target"].as_str().unwrap();
        for command in ["demo", "other"] {
            let current = receipt(root, command);
            assert_eq!(
                current["context"]["output"].as_str().unwrap(),
                root.join(format!("configured/{target}/release/{command}"))
                    .to_str()
                    .unwrap()
            );
        }
        success(cli(root, &["install", "--destdir", "stage"]));
        assert_eq!(
            std::fs::read(root.join("stage/usr/local/bin/demo")).unwrap(),
            std::fs::read(first["context"]["output"].as_str().unwrap()).unwrap()
        );
        // A PATH containing only Git proves packaging needs no Cargo or rustc.
        let tools = distribution.path().join("tools");
        std::fs::create_dir(&tools).unwrap();
        #[cfg(unix)]
        {
            let git = Command::new("which").arg("git").output().unwrap();
            std::os::unix::fs::symlink(
                String::from_utf8(git.stdout).unwrap().trim(),
                tools.join("git"),
            )
            .unwrap();
        }
        success(
            Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
                .args(["package", "--output", output.to_str().unwrap()])
                .env("PATH", &tools)
                .env("RUSTC", "absent")
                .current_dir(root)
                .output()
                .unwrap(),
        );
        let mut expected = vec![format!("SHA256SUMS-{target}.txt")];
        for command in ["demo", "other"] {
            let name = format!("{command}-v1.2.3-{target}");
            expected.extend([name.clone(), format!("{name}.bom.json")]);
            let current = receipt(root, command);
            assert_eq!(
                std::fs::read(output.join(&name)).unwrap(),
                std::fs::read(current["context"]["output"].as_str().unwrap()).unwrap()
            );
            assert_eq!(
                std::fs::read(output.join(format!("{name}.bom.json"))).unwrap(),
                std::fs::read(root.join(current["prepared"]["sbom"].as_str().unwrap())).unwrap()
            );
        }
        let mut actual: Vec<_> = std::fs::read_dir(&output)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_str().unwrap().to_owned())
            .collect();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
        for line in std::fs::read_to_string(output.join(format!("SHA256SUMS-{target}.txt")))
            .unwrap()
            .lines()
        {
            let (hash, name) = line.split_once("  ").unwrap();
            assert_eq!(
                hash,
                craftguild_core::paths::digest(&std::fs::read(output.join(name)).unwrap())
            );
        }
        // The ignored destination's parent is tracked; its sibling stage is not ignored.
        let ignored = root.join("dist");
        success(cli(root, &["package", "--output", "dist"]));
        for entry in std::fs::read_dir(&output).unwrap() {
            let entry = entry.unwrap();
            assert_eq!(
                std::fs::read(entry.path()).unwrap(),
                std::fs::read(ignored.join(entry.file_name())).unwrap()
            );
        }
        assert_eq!(
            craftguild_cli_build::observe::repository(root).1,
            craftguild_core::Dirty::Clean
        );
        assert!(!cli(root, &["package", "--output", "dist"]).status.success());
        std::fs::remove_dir_all(&ignored).unwrap();
        #[cfg(unix)]
        concurrent_changes_refuse_publication(root, &tools, &ignored);
        assert!(
            !cli(root, &["package", "--output", output.to_str().unwrap()])
                .status
                .success()
        );
        std::fs::remove_dir_all(&output).unwrap();
        // Every rejected package leaves no final destination.
        let source = root.join("src/main.rs");
        let before = std::fs::read(&source).unwrap();
        std::fs::write(&source, b"changed").unwrap();
        rejected(root, &output);
        std::fs::remove_file(&source).unwrap();
        rejected(root, &output);
        std::fs::write(&source, &before).unwrap();
        // Ignored additions still invalidate exact sealed directory inventories.
        write(root, "src/ignored-extra.rs", "// extra");
        assert!(
            !cli(root, &["package", "--output", output.to_str().unwrap()])
                .status
                .success()
        );
        std::fs::remove_file(root.join("src/ignored-extra.rs")).unwrap();
        for path in [
            root.join(first["prepared"]["sbom"].as_str().unwrap()),
            Path::new(first["context"]["output"].as_str().unwrap()).to_owned(),
        ] {
            let bytes = std::fs::read(&path).unwrap();
            std::fs::write(&path, b"tampered").unwrap();
            rejected(root, &output);
            std::fs::write(&path, &bytes).unwrap();
        }
        let record = root.join(".craftguild/other/receipt.json");
        let original = receipt(root, "other");
        for (field, value) in [
            ("version", "9.9.9"),
            ("target", "other-target"),
            ("commit", "1111111111111111111111111111111111111111"),
            ("dirty", "dirty"),
            ("profile", "debug"),
        ] {
            let mut changed = original.clone();
            changed["prepared"]["build"][field] = value.into();
            std::fs::write(&record, serde_json::to_vec(&changed).unwrap()).unwrap();
            rejected(root, &output);
        }
        std::fs::write(&record, serde_json::to_vec(&original).unwrap()).unwrap();
        let held = record.with_extension("held");
        std::fs::rename(&record, &held).unwrap();
        rejected(root, &output);
        std::fs::rename(&held, &record).unwrap();
        assert!(!cli(
            root,
            &[
                "package",
                "--group",
                "missing",
                "--output",
                output.to_str().unwrap()
            ]
        )
        .status
        .success());
        success(cli(
            root,
            &[
                "package",
                "--group",
                "demo",
                "--output",
                output.to_str().unwrap(),
            ],
        ));
        std::fs::remove_dir_all(&output).unwrap();
        // A clean, new signed HEAD still cannot package an older revision.
        write(root, "unrelated.txt", "new source revision");
        assert!(Command::new("git")
            .args(["add", "unrelated.txt"])
            .current_dir(root)
            .status()
            .unwrap()
            .success());
        signing::commit(root, "docs: next fixture revision");
        rejected(root, &output);
    }
}

#[cfg(unix)]
fn concurrent_changes_refuse_publication(root: &Path, tools: &Path, output: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let git = tools.join("git");
    let real_git = git.canonicalize().unwrap();
    let python = success(Command::new("which").arg("python3").output().unwrap());
    let python = String::from_utf8(python.stdout).unwrap();
    let source = root.join("src/main.rs");
    let before = std::fs::read(&source).unwrap();
    std::fs::remove_file(&git).unwrap();
    // The Git boundary introduces a change only once package has written its stage.
    for name in ["src/main.rs", "package-unowned/evidence"] {
        let script = format!(
            r#"#!{}
import pathlib, subprocess, sys
root = pathlib.Path.cwd()
if 'status' in sys.argv and any(root.glob('package-*')):
    changed = root / {name:?}
    changed.parent.mkdir(parents=True, exist_ok=True)
    changed.write_text('concurrent change')
sys.exit(subprocess.call([{}, *sys.argv[1:]]))
"#,
            python.trim(),
            serde_json::to_string(real_git.to_str().unwrap()).unwrap(),
        );
        std::fs::write(&git, script).unwrap();
        std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
            .args(["package", "--output", "dist"])
            .env("PATH", tools)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr)
            .contains("Checkout or destination changed during package verification"));
        assert_eq!(
            std::fs::read_to_string(root.join(name)).unwrap(),
            "concurrent change"
        );
        assert!(!output.exists());
        std::fs::write(&source, &before).unwrap();
        if name.starts_with("package-unowned/") {
            std::fs::remove_dir_all(root.join("package-unowned")).unwrap();
        }
        assert!(!std::fs::read_dir(root).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .starts_with("package-")
        }));
    }
    std::fs::remove_file(&git).unwrap();
    std::os::unix::fs::symlink(real_git, git).unwrap();
}
