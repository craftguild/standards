use craftguild_core::{paths, Dirty};
use craftguild_standards::{
    config::{self, Artifact},
    distribution,
};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

fn write(root: &Path, name: &str, text: &str) {
    let p = root.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        r#"[workspace]
members=["runtime","build-only","test-only","unrelated","extra","unix-only"]
resolver="2"
[package]
name="demo"
version="1.2.3"
edition="2021"
[dependencies]
runtime={path="runtime"}
extra={path="extra",optional=true}
[features]
optional-part=["dep:extra"]
[target.'cfg(unix)'.dependencies]
unix-only={path="unix-only"}
[build-dependencies]
build-only={path="build-only"}
[dev-dependencies]
test-only={path="test-only"}
"#,
    );
    write(root, "src/main.rs", "fn main() {}\n");
    for name in [
        "runtime",
        "build-only",
        "test-only",
        "unrelated",
        "extra",
        "unix-only",
    ] {
        write(
            root,
            &format!("{name}/Cargo.toml"),
            &format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"),
        );
        write(
            root,
            &format!("{name}/src/lib.rs"),
            "pub fn exported() {}\n",
        );
    }
    write(
        root,
        "craftguild.toml",
        r#"schema_version=1
standards_version="0.1.0"
[[artifacts]]
id="demo"
command="demo"
project="demo"
kind="rust"
manifest="Cargo.toml"
lockfile="Cargo.lock"
package="demo"
"#,
    );
    assert!(Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    temp
}

fn names(root: &Path, prepared: &craftguild_core::seal::Prepared) -> Vec<String> {
    let bom: Value =
        serde_json::from_slice(&std::fs::read(root.join(&prepared.sbom)).unwrap()).unwrap();
    bom["components"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().into())
        .collect()
}

fn artifact(root: &Path) -> Artifact {
    config::load(root).unwrap().artifacts.remove(0)
}

#[test]
fn selected_target_features_and_runtime_edges_only() {
    let temp = fixture();
    let root = temp.path();
    let mut a = artifact(root);
    let first = distribution::build(root, &a, "x86_64-unknown-linux-gnu", false).unwrap();
    assert_eq!(names(root, &first), vec!["runtime", "unix-only"]);
    assert_eq!(first.build.dirty, Dirty::Unknown);
    let first_hash = first.sbom_sha256;
    a.features.push("optional-part".into());
    let second = distribution::build(root, &a, "x86_64-pc-windows-msvc", false).unwrap();
    assert_eq!(names(root, &second), vec!["extra", "runtime"]);
    assert_ne!(first_hash, second.sbom_sha256);
    assert!(second.features.contains(&"optional-part".into()));
    second
        .verify(root, &second.build, &second.features)
        .unwrap();
    write(root, "Cargo.lock", "# changed locked inputs\n");
    assert!(second
        .verify(root, &second.build, &second.features)
        .is_err());
}

#[test]
fn embedded_npm_and_font_assets_but_no_unused_dev_dependency() {
    let temp = fixture();
    let root = temp.path();
    let mut a = artifact(root);
    write(
        root,
        "web/package-lock.json",
        &json!({"lockfileVersion":3,"packages":{
        "node_modules/shipped":{"version":"2.0.0","dev":true},
        "node_modules/unused-dev":{"version":"9.0.0","dev":true}}})
        .to_string(),
    );
    write(
        root,
        "web/node_modules/shipped/package.json",
        r#"{"name":"shipped","version":"2.0.0"}"#,
    );
    write(
        root,
        "web/node_modules/shipped/index.js",
        "export const value = 42;\n",
    );
    write(root, "web/dist/app.js", "console.log(42);\n");
    write(root,"web/metafile.json",&json!({"inputs":{"node_modules/shipped/index.js":{"bytes":25}},
        "outputs":{"dist/app.js":{"bytes":17,"inputs":{"node_modules/shipped/index.js":{"bytesInOutput":2}}}}}).to_string());
    write(root, "font.woff2", "sample-font");
    write(
        root,
        "supplemental.json",
        r#"[{"name":"sample-font","version":"1.0","type":"file","reference":"font:sample","files":["font.woff2"],"evidence":"Explicitly bundled font fixture"}]"#,
    );
    a.supplemental = Some("supplemental.json".into());
    // The fixture command validates prewritten deterministic bundler evidence.
    // A release acceptance test must also run an actual npm/esbuild build.
    a.bundle = Some(config::Bundle {
        root: "web".into(),
        lockfile: "web/package-lock.json".into(),
        metafile: "web/metafile.json".into(),
        directory: "web/dist".into(),
        command: vec!["python3".into(), "-c".into(), "pass".into()],
    });
    let p = distribution::build(root, &a, "x86_64-unknown-linux-gnu", false).unwrap();
    let included = names(root, &p);
    assert!(included.contains(&"shipped".into()));
    assert!(included.contains(&"sample-font".into()));
    assert!(!included.contains(&"unused-dev".into()));
    write(root, "web/dist/app.js", "changed asset\n");
    assert!(p.verify(root, &p.build, &p.features).is_err());
    assert!(distribution::build(root, &a, "x86_64-unknown-linux-gnu", false).is_err());
    assert!(!root.join(".craftguild/demo/prepared.json").exists());
}

#[test]
fn independent_frontend_is_not_in_backend_distribution() {
    let temp = fixture();
    let root = temp.path();
    write(
        root,
        "frontend/package.json",
        r#"{"name":"separate-ui","version":"4.5.6"}"#,
    );
    write(
        root,
        "frontend/package-lock.json",
        r#"{"lockfileVersion":3,"packages":{}}"#,
    );
    write(root, "frontend/src/main.js", "console.log('standalone');\n");
    write(root, "frontend/dist/app.js", "console.log('standalone');\n");
    write(
        root,
        "frontend/metafile.json",
        &json!({"inputs":{"src/main.js":{"bytes":27}},
        "outputs":{"dist/app.js":{"bytes":27,"inputs":{"src/main.js":{"bytesInOutput":27}}}}})
        .to_string(),
    );
    let mut frontend = artifact(root);
    frontend.id = "frontend".into();
    frontend.command = "separate-ui".into();
    frontend.package = "separate-ui".into();
    frontend.kind = config::Kind::Frontend;
    frontend.manifest = "frontend/package.json".into();
    frontend.lockfile = "frontend/package-lock.json".into();
    frontend.bundle = Some(config::Bundle {
        root: "frontend".into(),
        lockfile: frontend.lockfile.clone(),
        metafile: "frontend/metafile.json".into(),
        directory: "frontend/dist".into(),
        command: vec!["python3".into(), "-c".into(), "pass".into()],
    });
    let backend =
        distribution::build(root, &artifact(root), "x86_64-unknown-linux-gnu", false).unwrap();
    assert!(!backend.inputs.keys().any(|p| p.starts_with("frontend/")));
    assert!(!names(root, &backend).contains(&"separate-ui".into()));
    let ui = distribution::build(root, &frontend, "browser", false).unwrap();
    assert_eq!(ui.build.version, "4.5.6");
    assert!(ui.inputs.contains_key("frontend/dist/app.js"));
    assert_eq!(paths::hash_file(root, &ui.sbom).unwrap(), ui.sbom_sha256);
}
