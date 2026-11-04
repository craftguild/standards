use craftguild_standards::release::{conventional, versions};

#[test]
fn breaking_description_ends_at_the_next_footer_and_preserves_its_own_continuation() {
    let (bump, description) = conventional::parse(
        "fix: adjust contract\n\nBREAKING CHANGE: callers must migrate\n  Migration: rename the old method\nReviewed-by: Example Reviewer\n  review-only continuation\nRefs #123\n  issue-only continuation"
    ).unwrap();
    assert_eq!(bump, conventional::Bump::Major);
    assert_eq!(
        description,
        "callers must migrate Migration: rename the old method"
    );
    let (_, description) = conventional::parse(
        "fix: adjust contract\n\nReviewed-by: Example Reviewer\n  review-only continuation\nBREAKING-CHANGE: use the new API\n  including its new argument"
    ).unwrap();
    assert_eq!(description, "use the new API including its new argument");
    assert!(
        conventional::parse("fix: adjust\n\nBREAKING CHANGE:missing separator: more detail")
            .is_err()
    );
}

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
