use craftguild_standards::{measure, rust_regions};

#[test]
fn field_variant_arm_and_statement_exclusions_preserve_valid_production_syntax() {
    let source = r#"
struct Data { keep: u8, #[cfg(test)] probe: usize, }
struct Tuple(u8, #[cfg(all(test, unix))] usize,);
enum State { Ready, #[cfg(test)] Probe { value: usize }, }
fn make() -> Data { Data { keep: 1, #[cfg(test)] probe: 2, } }
#[cfg(any(test, feature = "worker"))]
fn conditional() { #[cfg(test)] observe(); }
#[cfg(not(test))] fn production() {}
fn select(x: u8) { match x { #[cfg(test)] 0 => observe(), _ => (), } }
fn statements() {
    let retained = "日本語";
    #[cfg(all(unix, test))] let probe = 3;
    #[cfg(test)] observe();
    println!("{retained}");
}
"#;
    let result = rust_regions::implementation(source).unwrap();
    syn::parse_file(&result).unwrap();
    assert_eq!(source.lines().count(), result.lines().count());
    assert!(result.contains("keep: u8") && result.contains("fn conditional"));
    assert!(result.contains("fn production") && result.contains("_ => ()"));
    assert!(result.contains("#[cfg(any(test, feature = \"worker\"))]"));
    assert!(result.contains("#[cfg(not(test))]"));
    assert!(result.contains("日本語"));
    assert!(!result.contains("probe") && !result.contains("observe") && !result.contains("Probe"));
}

#[test]
fn production_attributes_retain_their_original_code_lines() {
    let source = "#[cfg(any(test, feature = \"worker\"))]\nfn conditional() {}\n#[cfg(not(test))]\nfn production() {}\nstruct Data {\n    keep: u8,\n    #[cfg(test)] probe: usize,\n}\n";
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("mixed.rs"), source).unwrap();
    assert_eq!(measure::file(root.path(), "mixed.rs").unwrap(), 7);
    assert_eq!(
        rust_regions::implementation("#[cfg(feature=\"test\")] fn ordinary_feature() {}\n")
            .unwrap(),
        "#[cfg(feature=\"test\")] fn ordinary_feature() {}\n"
    );
}

#[test]
fn unsupported_contexts_and_attributes_remain_unmeasured() {
    for source in [
        "#[cfg_attr(test, inline)] fn work() {}",
        "#[cfg(mystery(test))] fn work() {}",
        "#[custom::test] fn work() {}",
        "fn work() { let values = [#[cfg(test)] 1, 2]; }",
    ] {
        assert!(rust_regions::implementation(source).is_err(), "{source}");
    }
}
