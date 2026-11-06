use craftguild_standards::release::{conventional, versions, Mode};

#[test]
fn version_policy_uses_each_groups_major_and_checks_overflow() {
    use conventional::Bump::*;
    for current in ["0.0.9", "0.99.42", "1.2.3", "2.5.1"] {
        let v = versions::stable(current).unwrap();
        for inferred in [None, Patch, Minor, Major] {
            let (effective, next) =
                versions::select(current, inferred, false, Mode::Normal).unwrap();
            assert_eq!(
                effective,
                if v.major == 0 && inferred == Major {
                    Minor
                } else {
                    inferred
                }
            );
            if v.major == 0 {
                assert_eq!(versions::stable(&next).unwrap().major, 0);
            }
            let (bump, explicit) =
                versions::select(current, inferred, false, Mode::ExplicitMajor).unwrap();
            assert_eq!(bump, Major);
            assert_eq!(explicit, format!("{}.0.0", v.major + 1));
            assert_eq!(
                versions::select(current, inferred, true, Mode::Normal).unwrap(),
                (None, current.into())
            );
            assert_eq!(
                versions::select(current, inferred, true, Mode::ExplicitMajor)
                    .unwrap()
                    .1,
                explicit
            );
        }
    }
    for (version, bump, mode) in [
        ("0.18446744073709551615.0", Major, Mode::Normal),
        ("0.0.18446744073709551615", Patch, Mode::Normal),
        ("18446744073709551615.0.0", Patch, Mode::ExplicitMajor),
    ] {
        assert!(versions::select(version, bump, false, mode).is_err());
    }
}

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
