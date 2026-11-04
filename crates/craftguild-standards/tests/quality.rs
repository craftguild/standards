use craftguild_standards::{
    config::{self, Role},
    inventory, measure, quality, rust_regions, sea_orm,
};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn config_text() -> &'static str {
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
[[sources]]
pattern="src/**"
role="implementation"
reason="Application implementation"
[[sources]]
pattern="tests/**"
role="test"
reason="Test runner only"
"#
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .current_dir(temp.path())
        .status()
        .unwrap()
        .success());
    write(temp.path(), "craftguild.toml", config_text());
    write(temp.path(),"AGENTS.md","<!-- craftguild:common:start -->\nUse standards 0.1.0 and its skill. Run make check.\n<!-- craftguild:common:end -->\nRepository contracts.\n");
    temp
}

fn lines(count: usize) -> String {
    (0..count)
        .map(|i| format!("const VALUE_{i}: usize = {i};\n"))
        .collect()
}

#[test]
fn exact_400_401_exclusions_and_untracked_sources() {
    let temp = fixture();
    let root = temp.path();
    write(root, "src/good.rs", &lines(400));
    write(root, "src/big.rs", &lines(401));
    write(root, "tests/large.rs", &lines(800));
    let report = quality::assess(root, &config::load(root).unwrap(), None).unwrap();
    assert_eq!(report.measured["src/good.rs"], 400);
    assert_eq!(report.measured["src/big.rs"], 401);
    assert!(report.excluded.contains("tests/large.rs"));
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].rule, "CG-SIZE-001");
    assert_eq!(report.findings[0].path.as_deref(), Some("src/big.rs"));
}

#[test]
fn mixed_rust_removes_only_syntax_confirmed_test_items() {
    let temp = fixture();
    let root = temp.path();
    let text = format!(
        "{}\n#[cfg(test)]\nmod tests {{\n{}\n}}\n",
        lines(400),
        lines(600)
    );
    write(root, "src/mixed.rs", &text);
    assert_eq!(measure::file(root, "src/mixed.rs").unwrap(), 400);
    assert!(
        rust_regions::implementation("#[cfg(any(test, feature=\"x\"))] fn mixed() {}\n").is_err()
    );
    assert!(rust_regions::implementation("fn malformed(").is_err());
    assert!(rust_regions::implementation("#[custom::test] fn ambiguous() {}\n").is_err());
    let imp =
        rust_regions::implementation("fn normal() { let text = \"#[test] fn fake() {}\"; }\n")
            .unwrap();
    assert!(imp.contains("fake"));
}

#[test]
fn unsupported_counter_source_unknown_role_and_invalid_configuration_fail() {
    let temp = fixture();
    let root = temp.path();
    write(root, "src/unknown.unrecognized", "something executable");
    assert!(measure::file(root, "src/unknown.unrecognized").is_err());
    write(root, "unowned.rs", "fn hidden() {}\n");
    let files = inventory::files(root, None).unwrap();
    assert!(inventory::roles(&files, &config::load(root).unwrap().sources).is_err());
    write(
        root,
        "craftguild.toml",
        &(config_text().to_owned() + "\nunsupported=true\n"),
    );
    assert!(config::load(root).is_err());
    write(
        root,
        "craftguild.toml",
        &config_text().replace("kind=\"rust\"", "kind=\"unknown\""),
    );
    assert!(config::load(root).is_err());
    let mut cfg: config::Config = toml::from_str(config_text()).unwrap();
    cfg.sources.push(cfg.sources[0].clone());
    let files = BTreeSet::from(["src/one.rs".into()]);
    assert!(inventory::roles(&files, &cfg.sources).is_err());
    assert_eq!(cfg.sources[0].role, Role::Implementation);
}

#[test]
fn sea_orm_checks_real_syntax_and_skips_comments_and_strings() {
    let drivers = BTreeSet::from(["sqlx".into(), "rusqlite".into()]);
    for source in [
        "// sqlx::query, Statement::from_sql_and_values\nfn safe() {}",
        "fn safe() { let text = \"execute_raw() and rusqlite::Connection\"; }",
        "fn safe() { println!(\"Statement::from_sql_and_values\"); }",
    ] {
        assert!(
            sea_orm::check(source, &drivers).unwrap().is_empty(),
            "{source}"
        );
    }
    for source in [
        "fn bad() { db.execute_raw(value); }",
        "use sea_orm::Statement as S;",
        "fn bad() { sqlx::query(\"select\"); }",
        "fn bad() { Expr::cust(\"select\"); }",
        "fn bad() { invoke!(db.execute_unprepared(query)); }",
    ] {
        assert!(
            !sea_orm::check(source, &drivers).unwrap().is_empty(),
            "{source}"
        );
    }
}

#[test]
fn mixed_html_javascript_and_css_are_counted_once() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();
    std::fs::create_dir_all(root.join("target")).unwrap();
    let temp = tempfile::tempdir_in(root.join("target")).unwrap();
    write(temp.path(),"page.html","<html>\n<style>\nbody { color: red; }\n</style>\n<script>\nconst x = 1;\nconst y = 2;\n</script>\n</html>\n");
    let relative = temp
        .path()
        .join("page.html")
        .strip_prefix(&root)
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    assert_eq!(measure::file(&root, &relative).unwrap(), 9);
    write(
        temp.path(),
        "page.html",
        "<html><script>test('case', () => {});</script></html>\n",
    );
    assert!(measure::file(&root, &relative).is_err());
}

#[test]
fn missing_counter_and_repository_gate_failure_never_pass() {
    let temp = fixture();
    write(temp.path(), "src/main.rs", "fn main() {}\n");
    let result = Command::new(env!("CARGO_BIN_EXE_craftguild-standards-bootstrap"))
        .args(["check", "--json"])
        .current_dir(temp.path())
        .env("CRAFTGUILD_TOKEI", temp.path().join("missing-counter"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(report["findings"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Cannot execute required tool"));
    let mut cfg = config::load(temp.path()).unwrap();
    cfg.commands.push(config::CheckCommand {
        id: "must-run".into(),
        purpose: "Product invariant fixture".into(),
        argv: vec!["__craftguild_missing_product_check__".into()],
        phase: config::Phase::Check,
    });
    let mut report = quality::Report::default();
    quality::gates(temp.path(), &cfg, &mut report).unwrap();
    assert!(report.findings.iter().any(|f| f.rule == "CG-GATE-002"));
    assert!(report.executed.is_empty());
}

#[test]
fn changed_scope_includes_staged_unstaged_and_untracked_implementation() {
    let temp = fixture();
    let root = temp.path();
    write(root, "src/main.rs", "fn main() {}\n");
    write(root, "src/other.rs", "pub fn original() {}\n");
    assert!(Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    git_fixture::commit(root, "test: establish quality baseline");
    write(
        root,
        "src/main.rs",
        "fn main() { println!(\"changed\"); }\n",
    );
    assert!(Command::new("git")
        .args(["add", "src/main.rs"])
        .current_dir(root)
        .status()
        .unwrap()
        .success());
    write(root, "src/other.rs", "pub fn changed() {}\n");
    write(root, "src/new.rs", "pub fn untracked() {}\n");
    let files = inventory::files(root, Some("HEAD")).unwrap();
    assert_eq!(
        files,
        BTreeSet::from([
            "src/main.rs".into(),
            "src/other.rs".into(),
            "src/new.rs".into()
        ])
    );
    let report = quality::assess(root, &config::load(root).unwrap(), Some("HEAD")).unwrap();
    assert_eq!(report.measured.len(), 3);
    assert!(report.findings.is_empty());
    assert!(inventory::files(root, Some("--output=unexpected")).is_err());
}
#[path = "../../../conformance/git_fixture.rs"]
mod git_fixture;
