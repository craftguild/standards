use craftguild_standards::banner::{generate, run};
use std::path::Path;

#[test]
fn matches_five_existing_normalized_assets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance/banners");
    for name in [
        "chigiri.rs",
        "musubi.rs",
        "pm9.rs",
        "agentchat.rs",
        "kicker.rs",
    ] {
        assert_eq!(
            generate(name).unwrap(),
            std::fs::read_to_string(root.join(format!("{name}.txt"))).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn all_seven_public_names_and_unsuffixed_products_are_supported() {
    for name in [
        "oriha.rs",
        "chigiri.rs",
        "musubi.rs",
        "pm9.rs",
        "agentchat.rs",
        "shiori.rs",
        "kicker.rs",
        "standards",
        "product",
    ] {
        let banner = generate(name).unwrap();
        assert!(banner.is_ascii());
        assert!(banner.ends_with('\n'));
        assert!(!banner.starts_with('\n'));
        assert!(!banner.ends_with("\n\n"));
        assert!(banner.lines().all(|s| !s.ends_with(' ')));
    }
    for bad in [
        "",
        "\n",
        "two\nlines",
        "a\t",
        "é",
        "日本語",
        "\x7f",
        "\x1b[0m",
        "   ",
    ] {
        assert!(generate(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn output_and_nonmutating_check() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("banner.txt");
    let path = path.to_str().unwrap().to_owned();
    run(
        &["pm9.rs".into(), "--output".into(), path.clone()],
        &mut vec![],
    )
    .unwrap();
    let expected = std::fs::read(&path).unwrap();
    run(
        &["pm9.rs".into(), "--check".into(), path.clone()],
        &mut vec![],
    )
    .unwrap();
    assert!(run(
        &["other".into(), "--check".into(), path.clone()],
        &mut vec![]
    )
    .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), expected);
    let mut stdout = vec![];
    run(&["pm9.rs".into()], &mut stdout).unwrap();
    assert_eq!(stdout, expected);
}
