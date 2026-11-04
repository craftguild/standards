use craftguild_core::paths;
use craftguild_standards::{config::Generated, generated};
use serde_json::json;
use std::path::Path;

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
