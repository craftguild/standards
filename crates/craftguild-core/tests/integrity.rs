use craftguild_core::{paths, sbom, seal::Prepared, BuildInfo, Dirty};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn info() -> BuildInfo {
    BuildInfo {
        command: "demo".into(),
        version: "1.2.3".into(),
        commit: "unknown".into(),
        dirty: Dirty::Unknown,
        compiler: "rustc example".into(),
        target: "example-target".into(),
        profile: "release".into(),
    }
}

fn prepared(root: &std::path::Path) -> Prepared {
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    let mut p = Prepared {
        schema_version: 1,
        build: info(),
        features: vec![],
        inputs: BTreeMap::from([(
            "src/main.rs".into(),
            paths::hash_file(root, "src/main.rs").unwrap(),
        )]),
        inventories: BTreeMap::from([("src".into(), BTreeSet::from(["src/main.rs".into()]))]),
        sbom: "bom.json".into(),
        sbom_sha256: String::new(),
    };
    let bom = json!({"bomFormat":"CycloneDX","specVersion":"1.5","version":1,
        "metadata":{"component":{"type":"application","bom-ref":"demo","name":"demo","version":"1.2.3"},
        "properties":p.properties().unwrap()},
        "components":[],"dependencies":[{"ref":"demo","dependsOn":[]}]});
    let bytes = serde_json::to_vec_pretty(&bom).unwrap();
    p.sbom_sha256 = paths::digest(&bytes);
    std::fs::write(root.join("bom.json"), bytes).unwrap();
    p
}

#[test]
fn official_schema_and_graph_validation() {
    let root = tempfile::tempdir().unwrap();
    let p = prepared(root.path());
    p.verify(root.path(), &info(), &[]).unwrap();
    let bytes = std::fs::read(root.path().join("bom.json")).unwrap();
    let mut bom = sbom::validate(&bytes).unwrap();
    bom["components"] = json!([{"type":"invented","bom-ref":"x","name":"x"}]);
    assert!(sbom::validate(&serde_json::to_vec(&bom).unwrap()).is_err());
    bom["components"] = json!([]);
    bom["dependencies"][0]["dependsOn"] = json!(["missing"]);
    assert!(sbom::validate(&serde_json::to_vec(&bom).unwrap()).is_err());
    bom["dependencies"] = json!([]);
    assert!(sbom::validate(&serde_json::to_vec(&bom).unwrap()).is_err());
    assert!(sbom::validate(b"{}").is_err());
}

#[test]
fn changed_missing_inputs_new_files_features_target_and_bytes_fail() {
    let root = tempfile::tempdir().unwrap();
    let p = prepared(root.path());
    let mut changed = info();
    changed.target = "other-target".into();
    assert!(p.verify(root.path(), &changed, &[]).is_err());
    assert!(p
        .verify(root.path(), &info(), &["different-feature".into()])
        .is_err());
    std::fs::write(root.path().join("src/new.rs"), "new input").unwrap();
    assert!(p.verify(root.path(), &info(), &[]).is_err());
    std::fs::remove_file(root.path().join("src/new.rs")).unwrap();
    std::fs::write(root.path().join("src/main.rs"), "changed").unwrap();
    assert!(p.verify(root.path(), &info(), &[]).is_err());
    std::fs::remove_file(root.path().join("src/main.rs")).unwrap();
    assert!(p.verify(root.path(), &info(), &[]).is_err());
    std::fs::write(root.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(root.path().join("bom.json"), "{}").unwrap();
    assert!(p.verify(root.path(), &info(), &[]).is_err());
}

#[test]
fn unsafe_paths_and_symlinks_never_become_owned() {
    for bad in [
        "", "../x", "/abs", "x//y", "./x", "x/./y", "x\\y", "C:/x", "x/",
    ] {
        assert!(paths::relative(bad).is_err(), "{bad}");
    }
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("good"), "good").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.path().join("good"), root.path().join("link")).unwrap();
        assert!(paths::hash_file(root.path(), "link").is_err());
    }
}
