use craftguild_standards::release::{conventional, versions};

#[test]
fn language_neutral_release_cases_match_parser() {
    let contract: serde_json::Value =
        serde_json::from_str(include_str!("../../../conformance/contracts.json")).unwrap();
    for case in contract["release"]["commit_cases"].as_array().unwrap() {
        let (bump, _) = conventional::parse(case["message"].as_str().unwrap()).unwrap();
        assert_eq!(serde_json::to_value(bump).unwrap(), case["bump"]);
    }
    for message in [
        "plain message",
        "feat(): no scope",
        "feat: ",
        "feat: ok\nbody",
        "feat: ok\n\nBREAKING CHANGE:",
    ] {
        assert!(conventional::parse(message).is_err(), "{message}");
    }
    for version in ["01.0.0", "1.0", "1.0.0-beta.1", "1.0.0+build"] {
        assert!(versions::stable(version).is_err(), "{version}");
    }
}
