use craftguild_core::paths;
use craftguild_standards::{config::Generated, generated};
use serde_json::json;
use std::path::Path;
use std::process::Command;

fn write(root: &Path, name: &str, text: &str) {
    let p = root.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn fixture() -> (tempfile::TempDir, Generated) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "schema.txt", "canonical source\n");
    write(root,"generate.py","from pathlib import Path\nimport sys\nPath(sys.argv[1], 'model.rs').write_text('pub struct Model;\\n')\n");
    write(root, "output/model.rs", "pub struct Model;\n");
    let manifest = json!({"schema_version":1,"owner":"canonical","generator_revision":"1234567890123456789012345678901234567890",
        "inputs":{"schema.txt":paths::hash_file(root,"schema.txt").unwrap(),"generate.py":paths::hash_file(root,"generate.py").unwrap()},
        "files":{"output/model.rs":paths::hash_file(root,"output/model.rs").unwrap()}});
    write(root, "ownership.json", &manifest.to_string());
    let group = Generated {
        owner: "canonical".into(),
        manifest: "ownership.json".into(),
        output_root: "output".into(),
        reproduce: vec!["python3".into(), "generate.py".into(), "{output}".into()],
        rust_size_exception: true,
    };
    (temp, group)
}

#[test]
fn verified_output_gets_only_the_explicit_rust_size_exception() {
    let (temp, mut group) = fixture();
    assert!(generated::verify(temp.path(), std::slice::from_ref(&group))
        .unwrap()
        .contains("output/model.rs"));
    group.rust_size_exception = false;
    assert!(generated::verify(temp.path(), &[group]).unwrap().is_empty());
}

#[test]
fn reject_false_ownership_hash_missing_extra_and_nonreproduction() {
    let (temp, mut group) = fixture();
    group.owner = "forged".into();
    assert!(generated::verify(temp.path(), &[group]).is_err());
    let (temp, group) = fixture();
    write(temp.path(), "output/new.rs", "pub struct Hidden;\n");
    assert!(generated::verify(temp.path(), &[group]).is_err());
    let (temp, group) = fixture();
    std::fs::remove_file(temp.path().join("output/model.rs")).unwrap();
    assert!(generated::verify(temp.path(), &[group]).is_err());
    let (temp, group) = fixture();
    write(temp.path(), "output/model.rs", "pub struct Forged;\n");
    assert!(generated::verify(temp.path(), &[group]).is_err());
    let (temp, mut group) = fixture();
    group.reproduce = vec![
        "python3".into(),
        "-c".into(),
        "pass".into(),
        "{output}".into(),
    ];
    assert!(generated::verify(temp.path(), &[group]).is_err());
}

#[test]
fn oversized_generated_implementation_is_measured_and_exempt_after_verification() {
    let (temp, group) = fixture();
    let root = temp.path();
    let code: String = (0..401)
        .map(|i| format!("pub const VALUE_{i}: usize = {i};\n"))
        .collect();
    write(root, "schema.txt", &code);
    write(root, "output/model.rs", &code);
    write(root, "generate.py", "from pathlib import Path\nimport sys\nPath(sys.argv[1], 'model.rs').write_bytes(Path('schema.txt').read_bytes())\n");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("ownership.json")).unwrap()).unwrap();
    for name in ["schema.txt", "generate.py"] {
        manifest["inputs"][name] = json!(paths::hash_file(root, name).unwrap());
    }
    manifest["files"]["output/model.rs"] =
        json!(paths::hash_file(root, "output/model.rs").unwrap());
    write(root, "ownership.json", &manifest.to_string());
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    write(root, "AGENTS.md", "<!-- craftguild:common:start -->\nUse standards 0.1.0 and skill; run make check.\n<!-- craftguild:common:end -->\nProduct invariants.\n");
    let mut config: craftguild_standards::config::Config = toml::from_str(
        r#"
schema_version=1
standards_version="0.1.0"
[[artifacts]]
id="demo"
command="demo"
project="demo"
kind="rust"
manifest="Cargo.toml"
lockfile="Cargo.lock"
package="demo"
[[sources]]
pattern="output/**"
role="generated"
reason="Canonical model outputs"
[[sources]]
pattern="generate.py"
role="implementation"
reason="Canonical generator"
"#,
    )
    .unwrap();
    config.generated.push(group);
    let report = craftguild_standards::quality::assess(root, &config, None).unwrap();
    assert_eq!(report.measured["output/model.rs"], 401);
    assert!(report.exempt.contains("output/model.rs"));
    assert!(report.findings.is_empty());
}
