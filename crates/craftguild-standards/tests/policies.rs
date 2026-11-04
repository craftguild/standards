use craftguild_standards::{config::Policy, policies};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

fn write(root: &Path, name: &str, text: &str) {
    let p = root.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

#[test]
fn dependency_allow_lists_restricted_layers_and_orm_scope() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers=[\"domain\",\"app\"]\nresolver=\"2\"\n",
    );
    write(
        root,
        "domain/Cargo.toml",
        "[package]\nname=\"domain\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    );
    write(root, "domain/src/lib.rs", "pub struct Domain;\n");
    write(root,"app/Cargo.toml","[package]\nname=\"app\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\ndomain={path=\"../domain\"}\n");
    write(root, "app/src/lib.rs", "pub fn operation() {}\n");
    assert!(Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    let layers = BTreeMap::from([
        ("domain".into(), vec![]),
        ("app".into(), vec!["domain".into()]),
    ]);
    let sources = BTreeSet::from(["domain/src/lib.rs".into(), "app/src/lib.rs".into()]);
    let good = Policy::Dependencies {
        manifest: "Cargo.toml".into(),
        layers: layers.clone(),
        restricted: BTreeMap::new(),
    };
    assert!(policies::check(root, &[good], &sources).unwrap().is_empty());
    let bad = Policy::Dependencies {
        manifest: "Cargo.toml".into(),
        layers,
        restricted: BTreeMap::from([("domain".into(), vec!["domain".into()])]),
    };
    assert_eq!(
        policies::check(root, &[bad], &sources).unwrap()[0].0,
        "CG-DEP-001"
    );
    let orm = Policy::SeaOrm {
        manifest: "Cargo.toml".into(),
        sources: vec!["app/src/**".into()],
        allowed_packages: vec!["app".into()],
    };
    assert!(policies::check(root, std::slice::from_ref(&orm), &sources)
        .unwrap()
        .is_empty());
    write(
        root,
        "app/src/lib.rs",
        "pub fn operation() { db.execute_raw(query); }\n",
    );
    assert_eq!(
        policies::check(root, &[orm], &sources).unwrap()[0].0,
        "CG-ORM-004"
    );
    let publication = Policy::Publication {
        repository: "craftguild/product".into(),
        manifest: Some("Cargo.toml".into()),
        npm_lockfiles: vec![],
    };
    assert!(
        policies::check(root, std::slice::from_ref(&publication), &sources)
            .unwrap()
            .is_empty()
    );
    let outside = tempfile::tempdir().unwrap();
    write(
        outside.path(),
        "Cargo.toml",
        "[workspace]\n[package]\nname=\"craftguild-cli\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    );
    write(outside.path(), "src/lib.rs", "pub struct Fixture;\n");
    let mut app = std::fs::read_to_string(root.join("app/Cargo.toml")).unwrap();
    app.push_str(&format!(
        "common={{package=\"craftguild-cli\",path={:?}}}\n",
        outside.path().to_str().unwrap()
    ));
    write(root, "app/Cargo.toml", &app);
    assert!(Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    let findings = policies::check(root, &[publication], &sources).unwrap();
    assert!(findings.iter().any(|(rule, _)| rule == "CG-PUB-001"));
    assert!(findings.iter().any(|(rule, _)| rule == "CG-PUB-002"));
}
